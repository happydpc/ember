use crate::core::scene::{
    Scene,
    Active
};
use crate::core::managers::SceneManager;
use bevy_ecs::{
    schedule::Stage,
};
use bevy_ecs::prelude::Schedule;

use std::cell::RefMut;


pub trait ApplicationState{
    fn run_schedule(&mut self, scene: &mut Scene<Active>);
    fn init_schedule(&mut self);
    fn scene_interface_path(&self) -> &'static str;
}

pub struct ApplicationIdleState{
    pub schedule: Option<Box<dyn Stage>>,
    pub scene_interface_path: &'static str,
}

impl ApplicationIdleState{
    pub fn create() -> Self{
        ApplicationIdleState{
            schedule: None,
            scene_interface_path: "./idle_state.ron",
        }
    }

    fn create_idle_scene(&mut self, scene_manager: Option<RefMut<SceneManager>>){
        let _scene_id = scene_manager.expect("Scene manager should exist here.").generate_and_register_scene();
    }
   
}

impl ApplicationState for ApplicationIdleState {
    fn run_schedule(&mut self, scene: &mut Scene<Active>){
        log::info!("Running scene schedule...");
        let mut schedule = self.schedule.take().expect("No setup schedule");
        schedule.run(&mut *scene.get_world().unwrap());
        self.schedule = Some(schedule);
    }

    fn init_schedule(&mut self){
        let schedule = Schedule::default();
        self.schedule = Some(Box::new(schedule));
    }
    
    fn scene_interface_path(&self) -> &'static str{
        self.scene_interface_path
    }
}