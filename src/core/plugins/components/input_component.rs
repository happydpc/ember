use winit::event::VirtualKeyCode;
use specs::{Component, HashMapStorage};
use std::collections::VecDeque;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(HashMapStorage)]
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