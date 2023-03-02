
use std::cell::{
    RefCell,
    RefMut
};
use std::sync::Mutex;
use std::path::Path;
use std::fs;
use thiserror::Error;
use crate::core::{
    scene::{
        Scene,
        Active,
        Inactive,
        Staged,
    },
};
use serde::de::DeserializeSeed;

use crate::core::events::scene_manager_messages::SceneManagerMessage;


use bevy_ecs::entity::EntityMap;
use crate::core::scene::TypeRegistryResource;
use crate::core::scene::SceneDeserializer;
use crate::core::scene::DynamicScene;
use bevy_ecs::prelude::Resource;


#[derive(Resource)]
pub struct SceneManagerMessagePump{
    messages: Mutex<Vec<SceneManagerMessage>>,
}

impl SceneManagerMessagePump{
    pub fn send(&mut self, message: SceneManagerMessage){
        self.messages.lock().unwrap().push(message);
    }
    pub fn clear(&self){
        self.messages.lock().unwrap().clear();
    }
}

impl Default for SceneManagerMessagePump{
    fn default() -> Self {
        SceneManagerMessagePump{
            messages: Mutex::new(vec!())
        }
    }
}

#[derive(Debug)]
pub enum SceneManagerUpdateResults{
    NoUpdate,
    NewSceneOpened    
}

#[derive(Debug, Error)]
pub enum SceneManagerUpdateError{
    #[error("No active scene")]
    NoActiveScene,
    #[error("No world on current scene")]
    NoWorldOnScene,
    #[error("Unknown scene manager message type")]
    UnknownMessageType,
    #[error("Error reading ron scene")]
    RonReadError,
    #[error("Error in constructing scene from ron file")]
    DeserializationError
}   

pub struct SceneManager{
    active_scene: Option<RefCell<Scene<Active>>>,
    staged_scene: Option<RefCell<Scene<Staged>>>,
    scene_counter: i16,
}


impl SceneManager{
    //
    // public API
    //

    // creates a new SceneManager
    pub fn new() -> Self {
        log::info!("Creating SceneManager...");
        SceneManager{
            active_scene: None,
            staged_scene: None,
            scene_counter: 0,
        }
    }

    pub fn startup(&mut self){
        log::info!("Starting SceneManager...");
    }
    pub fn shutdown(&mut self){
        log::info!("Shutting down scene manager...");
    }

    pub fn update(&mut self) -> Result<SceneManagerUpdateResults, SceneManagerUpdateError>{

        // let mut scene = self.get_active_scene().unwrap();
        let (messages, type_registry) = {
            let mut scene = match self.get_active_scene() {
                Some(s) => s, 
                None => return Err(SceneManagerUpdateError::NoActiveScene)
            };
            let world = match scene.get_world() {
                Some(w) => w,
                None => return Err(SceneManagerUpdateError::NoWorldOnScene)
            };
            
            let pump = world.get_resource::<SceneManagerMessagePump>().expect("Event channel not found");
            let type_registry = &world.get_resource::<TypeRegistryResource>().expect("Type registry not found.").0.clone();

            let messages = (*pump.messages.lock().unwrap()).clone();
            pump.clear();

            (messages, type_registry.clone())
        };
        for m in messages.iter(){
            match m {
                SceneManagerMessage::OpenProject {path, scene_name} => {
                    // i think here is where i deserde and tell the application that it needs to re-prep
                    let mut scene_path = path.clone();
                    scene_path.push_str("/scenes/");
                    scene_path.push_str(&scene_name.clone().to_owned());
                    log::info!(
                        "{}",
                        format!(
                            "Opening Project {} with scene {} - exists: {}",
                            path,
                            scene_path,
                            self.does_save_exist(scene_path.clone()),
                        )
                    );
                    let ron_str = match fs::read_to_string(scene_path) {
                        Ok(s) => s,
                        _ => return Err(SceneManagerUpdateError::RonReadError)
                    };
                    log::info!("Got ron string.");
                    let scene_bytes = ron_str.as_bytes();
                    let mut deserializer = ron::de::Deserializer::from_bytes(scene_bytes).expect("Error making deserializer.");
                    let scene_deserializer = SceneDeserializer {
                        type_registry: &*type_registry.read(),
                    };
                    log::info!("Created deserializer.");

                    // Get the metadata for the file
                    let metadata = match fs::metadata(path) {
                        Ok(metadata) => metadata,
                        Err(_e) => return Err(SceneManagerUpdateError::DeserializationError)
                    };
                    
                    let loaded_scene = DynamicScene::default();

                    if metadata.len() != 0 {
                        let _loaded_scene = match scene_deserializer.deserialize(&mut deserializer){
                            Ok(s) => s,
                            _ => return Err(SceneManagerUpdateError::DeserializationError)
                        };
                    }

                    log::info!("Down Syncing active scene...");
                    self.stage_active_scene();
                    self.deactivate_staged_scene();

                    log::info!("Creating and staging new scene from ron...");
                    let mut staged_scene = Scene::<Staged>::new(); 
                    let world = staged_scene.world.take().expect("No world on new scene");
                    loaded_scene.write_to_world(&mut world.borrow_mut(), &mut EntityMap::default());
                    staged_scene.world.replace(world);
                    self.set_staged_scene(staged_scene);

                    return Ok(SceneManagerUpdateResults::NewSceneOpened);
                },
                _ => return Err(SceneManagerUpdateError::UnknownMessageType)
            }
        }
        Ok(SceneManagerUpdateResults::NoUpdate)
    }

    pub fn prep_staged_scene(&mut self, _scene: &mut Scene<Staged>){
        // do ui here?
    }

    pub fn does_save_exist(&self, save_name: String) -> bool {
        Path::new(&save_name.as_str()).exists()
    }

    pub fn get_active_scene(&self) -> Option<RefMut<Scene<Active>>> {
        match &self.active_scene {
            Some(scene) => Some(scene.borrow_mut()),
            None => None,
        }
    }

    pub fn set_staged_scene(&mut self, scene: Scene<Staged>){
        log::info!("Setting new staged scene...");
        self.staged_scene = Some(RefCell::new(scene));
    }

    pub fn stage_inactive_scene(&mut self, inactive_scene: Scene::<Inactive>){
        let staged_scene = Some(RefCell::new(Scene::<Staged>::from(inactive_scene)));
        self.staged_scene = staged_scene;
    }

    pub fn activate_staged_scene(&mut self){
        log::info!("Activating staged scene...");
        let staged_scene = self.staged_scene.take().expect("Staged scene not set.");
        match &staged_scene.borrow().state.setup_schedule {
            Some(_s) => log::info!("Setup schedule exists..."),
            None => log::info!("wtf"),
        };
        let active_scene = Scene::<Active>::from(staged_scene.into_inner());
        self.active_scene = Some(RefCell::new(active_scene));
    }

    pub fn stage_active_scene(&mut self){
        log::info!("Staging active scene...");
        // take active scene
        let scene = self.active_scene.take().expect("No staged scene to take.");

        // transition it down to staged
        let staged_scene = Some(RefCell::new(Scene::<Staged>::from(scene.into_inner())));
        
        // store it
        self.staged_scene = staged_scene;
    }

    pub fn deactivate_staged_scene(&mut self){
        log::info!("Deactivating staged scene...");
        // get staged scene
        let staged_scene = self.staged_scene.take().expect("Staged scene not set.");

        // transition it to inactive
        let _inactive_scene = Scene::<Inactive>::from(staged_scene.into_inner());
    }

    pub fn get_staged_scene(&mut self) -> Option<RefMut<Scene<Staged>>> {
        let scene = self.staged_scene.as_mut().expect("No staged scene to return.");
        Some(scene.borrow_mut())
    }

    pub fn create_and_set_staged_scene(&mut self){
        let staged_scene = Scene::<Staged>::new();
        self.set_staged_scene(staged_scene);
    }

}
