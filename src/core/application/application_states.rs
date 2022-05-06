use crate::core::scene::{
    Scene,
    Active
};
use crate::core::systems::ui_systems::DebugUiSystem;
use crate::core::scene::scene_manager::SceneManager;
use bevy_ecs::{
    world::World,
    schedule::Stage,
};
use bevy_ecs::prelude::Schedule;
use bevy_ecs::prelude::SystemStage;
use std::cell::RefMut;


pub trait ApplicationState{
    fn run_schedule(&mut self, scene: &mut Scene<Active>);
    fn init_schedule(&mut self);
    fn scene_interface_path(&self) -> &'static str;
}

pub struct ApplicationIdleState{
    pub schedule: Option<Box<dyn Stage>>,
    pub scene_interface_path: &'static str,
    idle_scene_id: i16,
}

impl ApplicationIdleState{
    pub fn create() -> Self{
        ApplicationIdleState{
            schedule: None,
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
    fn run_schedule(&mut self, scene: &mut Scene<Active>){
        log::info!("Running scene schedule...");
        let mut schedule = self.schedule.take().expect("No setup schedule");
        schedule.run(&mut *scene.get_world().unwrap());
        self.schedule = Some(schedule);
    }

    fn init_schedule(&mut self){
        let mut schedule = Schedule::default();
        schedule.add_stage("draw_ui", SystemStage::parallel()
            .with_system(DebugUiSystem)
        );
        self.schedule = Some(Box::new(schedule));
    }
    
    fn scene_interface_path(&self) -> &'static str{
        self.scene_interface_path
    }
}