use super::{super::systems::system::System, scene::Scene};


pub struct SceneManager{
    pub active_scene: Option<Scene>,
    pub scenes: Vec<Scene>,
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
}
