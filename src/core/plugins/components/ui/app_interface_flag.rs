use bevy_ecs::component::Component;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct AppInterfaceFlag;
