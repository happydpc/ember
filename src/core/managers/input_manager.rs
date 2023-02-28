use ember_math::Vector2f;
use winit::event::ElementState;
use winit::event::ModifiersState;
use winit::event::VirtualKeyCode;
use super::super::managers::manager::Manager;
use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

use crate::core::scene::{Scene, Active, Staged};



use log;

pub type KeyInputQueue = VecDeque<VirtualKeyCode>;

#[derive(Clone)]
pub struct MouseState{
    pub mouse_down: [bool; 3],
    pub mouse_clicked: [bool; 3],
    pub mouse_released: [bool; 3],
    pub mouse_press_time: [Option<Instant>; 3],
    pub mouse_delta: Vector2f,
    pub scroll: [f32; 2]
}

impl Default for MouseState {
    fn default() -> Self {
        MouseState {
            mouse_down: [false; 3],
            mouse_clicked: [false; 3],
            mouse_released: [false; 3],
            mouse_press_time: [None; 3],
            mouse_delta: Vector2f::zero(),
            scroll: [0.0; 2]
        }
    }
}

pub struct InputManager{
    modifier_state: ModifiersState,
    current_key_pressed: Option<VirtualKeyCode>,
    key_input_queue: VecDeque<VirtualKeyCode>,
    mouse_state: MouseState,
}

impl Manager for InputManager{
    fn startup(&mut self){
        log::info!("Starting input manager...");
    }

    fn shutdown(&mut self){
        log::info!("Shutting down input manager.");
    }

    fn update(&mut self, scene: &mut Scene<Active>){
        log::debug!("Updating input manager.");
        scene.insert_resource(self.key_input_queue.clone());
        scene.insert_resource(self.modifier_state);
        scene.insert_resource(self.mouse_state.clone());

        self.clear_input_state();
    }
}

impl InputManager {

    // Creates a new input manager
    pub fn new() -> Self {
        log::info!("Creating input manager...");
        InputManager{
            modifier_state: ModifiersState::empty(),
            current_key_pressed: None,
            key_input_queue: VecDeque::new(),
            mouse_state: MouseState::default(),
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

    pub fn handle_mouse_button(&mut self, button: &u32, state: &ElementState){
        // button is 1: left, 2: middle, 3: right, so this shifts to zero index
        let button_index: usize = (button - 1) as usize;  
        match state{
            ElementState::Pressed => {
                self.mouse_state.mouse_clicked[button_index] = true;
                self.mouse_state.mouse_down[button_index] = true;
                self.mouse_state.mouse_press_time[button_index] = Some(Instant::now());
            },
            ElementState::Released => {
                self.mouse_state.mouse_clicked[button_index] = false;
                self.mouse_state.mouse_down[button_index] = false;
                self.mouse_state.mouse_released[button_index] = true;
                self.mouse_state.mouse_press_time[button_index] = None;
            }
        }
    }

    pub fn handle_mouse_move(&mut self, delta: Vector2f){
        self.mouse_state.mouse_delta = delta;
    }

    pub fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32){
        self.mouse_state.scroll[0] = delta_x;
        self.mouse_state.scroll[1] = delta_y;
    }

    pub fn prep_staged_scene(&mut self, scene: &mut Scene<Staged>){
        scene.insert_resource(self.key_input_queue.clone());
        scene.insert_resource(self.modifier_state);
    }

    pub fn clear_input_state(&mut self) {
        self.current_key_pressed = None;
        self.key_input_queue.clear();
        self.mouse_state.mouse_clicked = [false; 3];
        self.mouse_state.mouse_delta = Vector2f::zero();
        self.mouse_state.scroll = [0.0; 2];
    }
}
