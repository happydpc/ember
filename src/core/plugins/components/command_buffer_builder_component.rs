use bevy_ecs::component::Component;
use bevy_reflect::{Reflect};
use std::sync::{Arc, Mutex};

use vulkano::command_buffer_builder::PrimaryAutoCommandBufferBuilder;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct CommandBufferBuilderComponent{
use std::sync::{Arc, Mutex};
    pub buffer_builder: Option<Arc<Mutex<PrimaryAutoCommandBufferBuilder>>>,
}