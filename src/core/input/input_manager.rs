use winit::event::ModifiersState;
use winit::event::VirtualKeyCode;
use super::super::managers::manager::Manager;
use std::collections::VecDeque;
use crate::core::systems::CameraMoveSystem;
use crate::core::scene::{Scene, Initialized};
use crate::core::plugins::components::InputComponent;
use specs::System;

use log;


pub struct InputManager{
    modifier_state: Option<ModifiersState>,
    current_key_pressed: Option<VirtualKeyCode>,
    key_input_queue: VecDeque<VirtualKeyCode>,
    camera_move_system: CameraMoveSystem,
}

impl Manager for InputManager{
    fn startup(&mut self){
        log::info!("Starting input manager...");
    }

    fn shutdown(&mut self){
        log::info!("Shutting down input manager.");
    }

    fn update(&mut self, scene: &mut Scene<Initialized>){
        log::debug!("Updating input manager.");
        // let keys = self.key_input_queue.drain(..);
        // for key in keys{
        //     match key {
        //         VirtualKeyCode::W => log::info!("W pressed!"),
        //         VirtualKeyCode::A => log::info!("A pressed!"),
        //         VirtualKeyCode::S => log::info!("S pressed!"),
        //         VirtualKeyCode::D => log::info!("D pressed!"),
        //         VirtualKeyCode::Z => log::info!("Z pressed!"),
        //         VirtualKeyCode::Q => log::info!("Q pressed!"),
        //         VirtualKeyCode::E => log::info!("E pressed!"),
        //         _ => (),
        //     }
        // }
        self.current_key_pressed = None;
        self.camera_move_system.run(scene.get_world().unwrap().system_data());
    }
}

impl InputManager {

    // Creates a new input manager
    pub fn create_new() -> Self {
        log::info!("Creating input manager...");
        InputManager{
            modifier_state: None,
            current_key_pressed: None,
            key_input_queue: VecDeque::new(),
            camera_move_system: CameraMoveSystem{
                key_input_queue: VecDeque::new(),
            },
        }
    }

    // handle a change in modifiers
    pub fn handle_modifier_change(&mut self, new_state: Option<ModifiersState>) {
        log::debug!("Modifier changed: {:?}", new_state);
        self.modifier_state = new_state;
    }

    // handle key input
    pub fn handle_key_input(&mut self, key_pressed: Option<VirtualKeyCode>){
        log::debug!("Key input picked up by InputManager...");
        match key_pressed {
            Some(key) => self.camera_move_system.key_input_queue.push_back(key),
            _ => (),
        }
        self.current_key_pressed = key_pressed;
    }
}
