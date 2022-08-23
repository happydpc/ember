use bevy_ecs::component::Component;

use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::{Reflect, FromReflect};
use bevy_ecs::prelude::ReflectComponent;


#[derive(Component, Default, Clone, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]
pub struct SerializerFlag;