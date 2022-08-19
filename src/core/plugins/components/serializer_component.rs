use bevy_ecs::component::Component;

use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::Reflect;


#[derive(Component, Clone, Serialize, Deserialize)]
pub struct SerializerFlag;