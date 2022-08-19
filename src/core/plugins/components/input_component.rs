use winit::event::VirtualKeyCode;
use bevy_ecs::component::Component;
use bevy_reflect::Reflect;
use bevy_ecs::prelude::ReflectComponent;

use std::collections::VecDeque;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct InputComponent{
    #[serde(skip, default="InputComponent::default_input_vec")]
    #[reflect(ignore)]
    pub key_buffer: VecDeque<VirtualKeyCode>,
}

impl Default for InputComponent {
    fn default() -> Self {
        InputComponent{
            key_buffer: VecDeque::new(),
        }
    }
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