use specs::{Component, HashMapStorage};
use std::sync::{Arc, Mutex};

use vulkano::command_buffer_builder::PrimaryAutoCommandBufferBuilder;
use serde::{
    Serialize,
    Deserialize,
};


#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(HashMapStorage)]
pub struct CommandBufferBuilderComponent{
use std::sync::{Arc, Mutex};
    pub buffer_builder: Option<Arc<Mutex<PrimaryAutoCommandBufferBuilder>>>,
}