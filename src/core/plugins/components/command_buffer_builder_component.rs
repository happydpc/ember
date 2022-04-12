use specs::{Component, HashMapStorage};
use std::sync::{Arc, Mutex};

use vulkano::command_buffer_builder::PrimaryAutoCommandBufferBuilder;

#[derive(Component, Debug)]
#[storage(HashMapStorage)]
pub struct CommandBufferBuilderComponent{
use std::sync::{Arc, Mutex};
    pub buffer_builder: Option<Arc<Mutex<PrimaryAutoCommandBufferBuilder>>>,
}