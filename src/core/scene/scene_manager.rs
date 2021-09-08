use std::collections::HashMap;
use std::cell::RefCell;
use super::{super::managers::manager::Manager, scene::Scene};
use specs::{
    World,
};

pub struct SceneManager{
    active_scene: Option<i16>,  // Either the scene ID or None
    scenes: HashMap<i16, RefCell<Box<Scene>>>, // Scene ids and scenes
    scene_counter: i16,
}

impl Manager for SceneManager{
    fn startup(&mut self){
        log::info!("Starting SceneManager...");
    }
    fn shutdown(&mut self){
        self.scenes.clear();
    }
    fn update(&mut self){
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
            scenes: HashMap::new(),
            scene_counter: 0,
        }
    }

    // adds a scene and returns its scene id
    pub fn add_scene(&mut self, scene: Scene) -> i16 {
        self.scene_counter+=1;
        let key = self.scene_counter;
        self.scenes.insert(key, RefCell::new(Box::new(scene)));
        key
    }

    pub fn remove_scene(&mut self, scene_id: i16){
        self.scenes.remove(&scene_id);
    }

    // takes a scene id and, if that scene exists, sets that id to be the active scene.
    pub fn set_active_scene(&mut self, scene_id: i16){
        match self.scenes.get(&scene_id){
            Some(_x) => self.active_scene = Some(scene_id),
            None => log::info!("Scene {} is not in the SceneManager.", scene_id),
        }
    }

    // gets the active scene id
    pub fn get_active_scene(&self) -> Option<i16> {
        self.active_scene
    }

    pub fn switch_to(&mut self, scene_id: i16){
        match self.active_scene{
            Some(id) => self.scenes[&id].borrow_mut().deinitialize(),
            None => (),
        }
        self.scenes[&scene_id].borrow_mut().initialize();
        self.active_scene = Some(scene_id);
    }
}
