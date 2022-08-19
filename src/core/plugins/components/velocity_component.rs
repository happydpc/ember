use bevy_ecs::component::Component;

use ember_math::Vector3f;
use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::Reflect;


#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct VelocityComponent{
    pub velocity: Vector3f,
}
