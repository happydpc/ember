use std::collections::HashMap;
use std::cell::{
    RefCell,
    RefMut
};
use super::{
    super::managers::manager::Manager,
    scene::{
        Scene,
        Active,
        Inactive
    },
};

pub struct SceneManager{
    active_scene: Option<RefCell<Scene<Active>>>,
    active_scene_id: Option<i16>,
    scenes: HashMap<i16, Scene<Inactive>>, // Scene ids and scenes
    scene_counter: i16,
}

impl Manager for SceneManager{
    fn startup(&mut self){
        log::info!("Starting SceneManager...");
    }
    fn shutdown(&mut self){
        self.scenes.clear();
    }
    fn update(&mut self, _scene: &mut Scene<Active>){
    }
}

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
            scenes: HashMap::new(),
            scene_counter: 0,
        }
    }

    // adds a scene and returns its scene id
    pub fn generate_and_register_scene(&mut self) -> i16 {
        self.scene_counter+=1;
        let key = self.scene_counter;
        self.scenes.insert(key, Scene::<Inactive>::new());
        log::info!("Registering scene {}.", key);
        key
    }

    pub fn unregister_scene(&mut self, scene_id: i16){
        self.scenes.remove(&scene_id);
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

    pub fn borrow_mut_scene(&mut self, id: i16) -> Option<&mut Scene<Inactive>>{
        self.scenes.get_mut(&id)
    }

    pub fn set_active_scene(&mut self, scene_id: i16){
        log::info!("Attempting to activate scene {}.", scene_id);
        // if there is an active scene id, deactivate that scene and restore it in the hash map
        match self.active_scene_id{
            Some(_id) => {
                let deinit_scene = Scene::<Inactive>::from(
                    self.active_scene
                    .take()
                    .unwrap()
                    .into_inner()
                );
                log::info!("Deactivating scene {}.", self.active_scene_id.unwrap());
                self.scenes.insert(self.active_scene_id.take().unwrap(), deinit_scene);
            },
            None => (),
        }
        // now initialize the scene if it exists
        let scene = self.scenes.remove(&scene_id);
        match scene{
            Some(s) => {
                let Active_scene = Scene::<Active>::from(s);
                self.active_scene = Some(RefCell::new(Active_scene));
                self.active_scene_id = Some(scene_id);
                log::info!("Activated scene {}", scene_id);
            },
            None => {
                log::error!("Scene {} does not exist.", scene_id);
            }
        }

    }
}
