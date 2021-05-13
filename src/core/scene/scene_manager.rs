use super::{super::systems::system::System, scene::Scene};


pub struct SceneManager{
    pub active_scene: Option<Scene>,
    pub scenes: Vec<Scene>,  // maybe this should be a dictionary so set active scene can take a key and active scene can also be a key?
}

impl System for SceneManager{
    fn startup(&mut self){
        println!("Starting SceneManager...");
    }
    fn shutdown(&mut self){
        // should this iterate over a scene and save it? probably depends
        self.scenes.clear();
    }
    fn update(&self){

    }
}

impl SceneManager{
    pub fn create_new() -> Self {
        println!("Creating SceneManager...");
        SceneManager{
            active_scene: None,
            scenes: Vec::new(),
        }
    }
    pub fn set_active_scene(&mut self){

    }
}
