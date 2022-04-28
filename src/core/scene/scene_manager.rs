use std::collections::HashMap;
use std::cell::{
    RefCell,
    RefMut
};
use std::sync::Mutex;
use super::{
    super::managers::manager::Manager,
    scene::{
        Scene,
        Active,
        Inactive,
        Staged,
    },
};

pub struct SceneManager{
    active_scene: Option<RefCell<Scene<Active>>>,
    active_scene_id: Option<i16>,
    staged_scene: Option<RefCell<Scene<Staged>>>,
    staged_scene_id: Option<i16>,
    scenes: Mutex<HashMap<i16, Scene<Inactive>>>, // Scene ids and scenes
    scene_counter: i16,
}

impl Manager for SceneManager{
    fn startup(&mut self){
        log::info!("Starting SceneManager...");
    }
    fn shutdown(&mut self){
        self.scenes.lock().unwrap().clear();
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
            staged_scene: None,
            staged_scene_id: None,
            scenes: Mutex::new(HashMap::new()),
            scene_counter: 0,
        }
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

    pub fn activate_staged_scene(&mut self){
        let staged_scene_id = self.staged_scene_id.take().expect("Staged scene id not set.");
        let staged_scene = self.staged_scene.take().expect("Staged scene not set.");
        log::info!("Activating staged scene {:?}...", staged_scene_id);
        let active_scene = Scene::<Active>::from(staged_scene.into_inner());
        self.active_scene = Some(RefCell::new(active_scene));
        self.active_scene_id = Some(staged_scene_id);
    }

    pub fn stage_scene(&mut self, id: i16){
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

    pub fn get_staged_scene(&mut self) -> Option<RefMut<Scene<Staged>>> {
        let scene = self.staged_scene.as_mut().expect("No staged scene to return.");
        Some(scene.borrow_mut())
    }

    pub fn stage_active_scene(&mut self){
        todo!();
    }
}
