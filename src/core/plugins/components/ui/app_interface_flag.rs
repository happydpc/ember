use bevy_ecs::component::Component;
use bevy_reflect::{
    Reflect,
    FromReflect
};
use bevy_ecs::prelude::ReflectComponent;

use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Clone, Serialize, Deserialize, Reflect, FromReflect, Default)]
#[reflect(Component)]
pub struct AppInterfaceFlag;
