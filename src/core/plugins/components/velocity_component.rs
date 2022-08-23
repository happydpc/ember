use bevy_ecs::component::Component;

use ember_math::Vector3f;
use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::{
    Reflect,
    FromReflect
};
use bevy_ecs::prelude::ReflectComponent;

#[derive(Component, Debug, Default, Clone, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]
pub struct VelocityComponent{
    pub velocity: Vector3f,
}
