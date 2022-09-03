use std::collections::HashMap;
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
use bevy_reflect::TypeRegistryArc;

use crate::core::scene::SceneDeserializer;

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
    active_scene_id: Option<i16>,
    staged_scene: Option<RefCell<Scene<Staged>>>,
    staged_scene_id: Option<i16>,
    scenes: Mutex<HashMap<i16, Scene<Inactive>>>, // Scene ids and scenes
    scene_counter: i16,
}

// impl Manager for SceneManager{
//     fn startup(&mut self){
//         log::info!("Starting SceneManager...");
//     }
//     fn shutdown(&mut self){
//         self.scenes.lock().unwrap().clear();
//     }
//     fn update(&mut self, scene: &mut Scene<Active>){
//         log::info!("Finally doing something");
//     }
// }

impl SceneManager{
    //
    // public API
    //

    // creates a new SceneManager
    pub fn create_new() -> Self {
        log::info!("Creating SceneManager...");
        SceneManager{
            active_scene: None,
            active_scene_id: None,
            staged_scene: None,
            staged_scene_id: None,
            scenes: Mutex::new(HashMap::new()),
            scene_counter: 0,
        }
    }

    pub fn startup(&mut self){
        log::info!("Starting SceneManager...");
    }
    pub fn shutdown(&mut self){
        self.scenes.lock().unwrap().clear();
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
            let type_registry = world.get_resource::<TypeRegistryArc>().expect("Type registry not found.");

            let messages = (*pump.messages.lock().unwrap()).clone();
            pump.clear();

            (messages, type_registry.clone())
        };
        for m in messages.iter(){
            match m {
                SceneManagerMessage::OpenProject {path, scene_name} => {
                    // i think here is where i deserde and tell the application that it needs to re-prep
                    let mut scene_path = path.clone();
                    scene_path.push_str("scenes/");
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
                    let scene_bytes = ron_str.as_bytes();
                    let mut deserializer = ron::de::Deserializer::from_bytes(scene_bytes).expect("Error making deserializer.");
                    let scene_deserializer = SceneDeserializer {
                        type_registry: &*type_registry.read(),
                    };
                    let loaded_scene = match scene_deserializer.deserialize(&mut deserializer){
                        Ok(s) => s,
                        _ => return Err(SceneManagerUpdateError::DeserializationError)
                    };
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
    }

    pub fn does_save_exist(&self, save_name: String) -> bool {
        Path::new(&save_name.as_str()).exists()
    }

    // adds a scene and returns its scene id
    pub fn generate_and_register_scene(&mut self) -> i16 {
        self.scene_counter+=1;
        let key = self.scene_counter;
        self.scenes.lock().unwrap().insert(key, Scene::<Inactive>::new());
        log::info!("Registering scene {}.", key);
        key
    }

    pub fn unregister_scene(&mut self, scene_id: i16){
        self.scenes.lock().unwrap().remove(&scene_id);
        log::info!("Unregistered scene {}.", scene_id);
    }

    // gets the active scene id
    pub fn get_active_scene_id(&self) -> Option<i16> {
        self.active_scene_id.clone()
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
        self.staged_scene_id = Some(1);
    }

    pub fn stage_inactive_scene(&mut self, id: i16){
        if self.active_scene_id.is_some() {
            log::error!("Scene {:?} does not exist to be staged.", id);
            return;
        }
        log::info!("Staging scene {:?}...", id);
        let inactive_scene = self.scenes.lock().unwrap().remove(&id).expect("Scene does not exist");
        let staged_scene = Some(RefCell::new(Scene::<Staged>::from(inactive_scene)));
        self.staged_scene = staged_scene;
        self.staged_scene_id = Some(id);
    }

    pub fn activate_staged_scene(&mut self){
        log::info!("Activating staged scene...");
        let staged_scene_id = self.staged_scene_id.take().expect("Staged scene id not set.");
        let staged_scene = self.staged_scene.take().expect("Staged scene not set.");
        match &staged_scene.borrow().state.setup_schedule {
            Some(_s) => log::info!("Setup schedule exists..."),
            None => log::info!("wtf"),
        };
        let active_scene = Scene::<Active>::from(staged_scene.into_inner());
        self.active_scene = Some(RefCell::new(active_scene));
        self.active_scene_id = Some(staged_scene_id);
    }

    pub fn stage_active_scene(&mut self){
        log::info!("Staging active scene...");
        // take active scene
        let scene = self.active_scene.take().expect("No staged scene to take.");
        let id = self.active_scene_id.take().expect("No active scene id to take");

        // transition it down to staged
        let staged_scene = Some(RefCell::new(Scene::<Staged>::from(scene.into_inner())));
        
        // store it
        self.staged_scene = staged_scene;
        self.staged_scene_id = Some(id);
    }

    pub fn deactivate_staged_scene(&mut self){
        log::info!("Deactivating staged scene...");
        // get staged scene
        let _staged_scene_id = self.staged_scene_id.take().expect("Staged scene id not set.");
        let staged_scene = self.staged_scene.take().expect("Staged scene not set.");

        // transition it to inactive
        let _inactive_scene = Scene::<Inactive>::from(staged_scene.into_inner());
    }

    pub fn get_staged_scene(&mut self) -> Option<RefMut<Scene<Staged>>> {
        let scene = self.staged_scene.as_mut().expect("No staged scene to return.");
        Some(scene.borrow_mut())
    }

    pub fn does_scene_exist(&self, id: &i16) -> bool {
        self.scenes.lock().unwrap().contains_key(id)
    }

    pub fn load_scene_interface(&mut self, _interface_path: &'static str) {
        
        let scene_id = self.generate_and_register_scene();  // create scene
        self.stage_inactive_scene(scene_id);  // stage it
    //     {
    //         let mut scene = self.get_staged_scene().expect("should probably just not wrap this?. but anyways couldn't get scene during load.");

    //         {
    //             scene.register::<TransformComponent>();
    //             scene.register::<TransformUiComponent>();
    //             scene.register::<RenderableComponent>();
    //             scene.register::<CameraComponent>();
    //             scene.register::<InputComponent>();
    //             scene.register::<DebugUiComponent>();
    //             scene.register::<DirectionalLightComponent>();
    //             scene.register::<AmbientLightingComponent>();
    //             scene.register::<TerrainComponent>();
    //             scene.register::<TerrainUiComponent>();
    //             scene.register::<GeometryComponent>();
    //             scene.register::<SimpleMarker<SerializerFlag>>();
    //             scene.insert_resource(SimpleMarkerAllocator::<SerializerFlag>::new());
    //         }

    //         let mut world = scene.get_world().expect("couldn't get world out of scene in load from ron.");


    //         {
    //             // Ensure world is empty. Can probably remove this since I literally just made it?
    //             let mut to_delete = Vec::new();
    //             for e in world.entities().join() {
    //                 to_delete.push(e);
    //             }
    //             for del in to_delete.iter() {
    //                 world.delete_entity(*del).expect("Deletion failed");
    //             }
    //         }
        
    //         let data = fs::read_to_string(interface_path).unwrap();
    //         let mut de = ron::de::Deserializer::from_str(&data).expect("Couldn't create deserializer.");
        
    //         {
    //             let mut d = (&mut world.entities(), &mut world.write_storage::<SimpleMarker<SerializerFlag>>(), &mut world.insert_resource::<SimpleMarkerAllocator<SerializerFlag>>());
        
    //             deserialize_individually!(
    //                 world, de, d,
    //                 InputComponent,
    //                 CameraComponent,
    //                 TransformComponent,
    //                 TransformUiComponent,
    //                 DebugUiComponent,
    //                 RenderableComponent,
    //                 DirectionalLightComponent,
    //                 AmbientLightingComponent,
    //                 TerrainComponent,
    //                 TerrainUiComponent,
    //                 GeometryComponent
    //             );
    //         }
        }

    //     // finally, set that scene as active
    //     // self.activate_staged_scene();
    
    //     // let mut deleteme : Option<Entity> = None;
    //     // {
    //     //     let entities = ecs.entities();
    //     //     let helper = ecs.read_storage::<SerializationHelper>();
    //     //     let player = ecs.read_storage::<Player>();
    //     //     let position = ecs.read_storage::<Position>();
    //     //     for (e,h) in (&entities, &helper).join() {
    //     //         let mut worldmap = ecs.insert_resource::<super::map::Map>();
    //     //         *worldmap = h.map.clone();
    //     //         worldmap.tile_content = vec![Vec::new(); super::map::MAPCOUNT];
    //     //         deleteme = Some(e);
    //     //     }
    //     //     for (e,_p,pos) in (&entities, &player, &position).join() {
    //     //         let mut ppos = ecs.insert_resource::<rltk::Point>();
    //     //         *ppos = rltk::Point::new(pos.x, pos.y);
    //     //         let mut player_resource = ecs.insert_resource::<Entity>();
    //     //         *player_resource = e;
    //     //     }
    //     // }
    //     // ecs.delete_entity(deleteme.unwrap()).expect("Unable to delete helper");
    // }
}
