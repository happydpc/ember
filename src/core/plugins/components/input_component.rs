use winit::event::VirtualKeyCode;
use specs::{Component, HashMapStorage};
use std::collections::VecDeque;


#[derive(Component, Debug)]
#[storage(HashMapStorage)]
pub struct InputComponent{
    pub key_buffer: VecDeque<VirtualKeyCode>,
}

impl InputComponent{
    pub fn create() -> Self {
        InputComponent{
            key_buffer: VecDeque::new(),
        }
    }
}