use crate::core::scene::{
    Scene,
    Active
};
use crate::core::scene::system_dispatch::{MultiThreadedDispatcher, SystemDispatch};
use crate::construct_dispatcher;
use crate::core::systems::ui_systems::DebugUiSystem;
use crate::core::scene::scene_manager::SceneManager;

use std::cell::RefMut;


pub trait ApplicationState{
    fn run_dispatch(&mut self, scene: &mut Scene<Active>);
    fn init_dispatcher(&mut self);
    fn scene_interface_path(&self) -> &'static str;
}

pub struct ApplicationIdleState{
    pub dispatcher: Option<Box<dyn SystemDispatch + 'static>>,
    pub scene_interface_path: &'static str,
    idle_scene_id: i16,
}

impl ApplicationIdleState{
    pub fn create() -> Self{
        ApplicationIdleState{
            dispatcher: None,
            scene_interface_path: "./idle_state.ron",
            idle_scene_id: -1,
        }
    }

    fn create_idle_scene(&mut self, scene_manager: Option<RefMut<SceneManager>>){
        if self.idle_scene_id == -1 {  // if the scene doesn't exist, create one
            let scene_id = scene_manager.expect("Scene manager should exist here.").generate_and_register_scene();
            self.idle_scene_id = scene_id;
        }
    }
   
}

impl ApplicationState for ApplicationIdleState {
    fn run_dispatch(&mut self, scene: &mut Scene<Active>){
        log::info!("Running scene dispatch...");
        let mut dispatch = self.dispatcher.take().expect("No setup dispatch");
        dispatch.run_now(&mut *scene.get_world().unwrap());
        self.dispatcher = Some(dispatch);
    }

    fn init_dispatcher(&mut self){
        construct_dispatcher!(
            (DebugUiSystem, "debug_ui", &[])
        );
        self.dispatcher = Some(new_dispatch());
    }
    
    fn scene_interface_path(&self) -> &'static str{
        self.scene_interface_path
    }
}