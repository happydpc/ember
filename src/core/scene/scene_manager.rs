use std::collections::HashMap;
use super::{super::systems::system::System, scene::Scene};


pub struct SceneManager{
    active_scene: Option<i16>,  // Either the scene ID or None
    scenes: HashMap<i16, Box<Scene>>, // Scene ids and scenes
    scene_counter: i16,
}

impl System for SceneManager{
    fn startup(&mut self){
        println!("Starting SceneManager...");
    }
    fn shutdown(&mut self){
        self.scenes.clear();
    }
    fn update(&self){
        if self.active_scene.is_some(){

        }
    }
}

impl SceneManager{
    //
    // public API
    //

    // creates a new SceneManager
    pub fn create_new() -> Self {
        println!("Creating SceneManager...");
        SceneManager{
            active_scene: None,
            scenes: HashMap::new(),
            scene_counter: 0,
        }
    }

    // adds a scene and returns its scene id
    pub fn add_scene<T: 'static + Scene>(&mut self, scene: T) -> i16 {
        self.scene_counter+=1;
        let key = self.scene_counter;
        self.scenes.insert(key, Box::new(scene));
        key
    }

    // takes a scene id and, if that scene exists, sets that id to be the active scene.
    pub fn set_active_scene(&mut self, scene_id: i16){
        match self.scenes.get(&scene_id){
            Some(x) => self.active_scene = Some(scene_id),
            None => println!("Scene {} is not in the SceneManager.", scene_id),
        }
    }

    // gets the active scene id
    pub fn get_active_scene_id(&self) -> Option<i16> {
        self.active_scene
    }

    //
    // internal
    //
    fn initialize_active_scene(&self){
        // TODO : Should probably make decisions about scene serialization, what should be serialized,
        // where things should be serialized etc. then this initialize active scene could actually
        // load a serialized state. That also makes me wonder if i'm going to want a "wipe scene" function
        // that clears the serialized state of a scene.

    }

    fn teardown_active_scene(&self){

    }
}
