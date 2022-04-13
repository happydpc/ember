use winit::event::ModifiersState;
use winit::event::VirtualKeyCode;
use super::super::managers::manager::Manager;
use std::collections::VecDeque;

use crate::core::scene::{Scene, Initialized};



use log;

pub type KeyInputQueue = VecDeque<VirtualKeyCode>;

pub struct InputManager{
    modifier_state: ModifiersState,
    current_key_pressed: Option<VirtualKeyCode>,
    key_input_queue: VecDeque<VirtualKeyCode>,
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
        self.current_key_pressed = None;
        scene.insert_resource(self.key_input_queue.clone());
        scene.insert_resource(self.modifier_state);
        self.key_input_queue.clear();
    }
}

impl InputManager {

    // Creates a new input manager
    pub fn create_new() -> Self {
        log::info!("Creating input manager...");
        InputManager{
            modifier_state: ModifiersState::empty(),
            current_key_pressed: None,
            key_input_queue: VecDeque::new(),
        }
    }

    // handle a change in modifiers
    pub fn handle_modifier_change(&mut self, new_state: ModifiersState) {
        log::debug!("Modifier changed: {:?}", new_state);
        self.modifier_state = new_state;
    }

    // handle key input
    pub fn handle_key_input(&mut self, key_pressed: Option<VirtualKeyCode>){
        log::debug!("Key input picked up by InputManager...");
        match key_pressed {
            Some(key) => self.key_input_queue.push_back(key),
            _ => (),
        }
        self.current_key_pressed = key_pressed;
    }
}
