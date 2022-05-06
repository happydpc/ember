use winit::event::VirtualKeyCode;
use bevy_ecs::component::Component;

use std::collections::VecDeque;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct InputComponent{
    #[serde(skip, default="InputComponent::default_input_vec")]
    pub key_buffer: VecDeque<VirtualKeyCode>,
}

impl InputComponent{
    pub fn create() -> Self {
        InputComponent{
            key_buffer: VecDeque::new(),
        }
    }
    fn default_input_vec() -> VecDeque<VirtualKeyCode>{
        VecDeque::new()
    }
}