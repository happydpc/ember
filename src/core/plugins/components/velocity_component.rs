use bevy_ecs::component::Component;

use cgmath::Vector3;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct VelocityComponent{
    pub velocity: Vector3<f32>,
}
