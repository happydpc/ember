use bevy_ecs::component::Component;

use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::Reflect;
use bevy_ecs::prelude::ReflectComponent;

#[derive(Component, Debug, Serialize, Deserialize, Clone, Hash, Reflect)]
#[reflect(Component)]
pub struct DebugUiComponent{
    pub show_profiler: bool,
    pub terrain_wireframe: bool
}

impl Default for DebugUiComponent{
    fn default() -> Self {
        DebugUiComponent{
            show_profiler: false,
            terrain_wireframe: false,
        }
    }
}

impl DebugUiComponent{
    pub fn create() -> Self {
        DebugUiComponent{
            show_profiler: false,
            terrain_wireframe: false,
        }
    }
}