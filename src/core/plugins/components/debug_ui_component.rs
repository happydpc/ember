use bevy_ecs::component::Component;

use serde::{
    Serialize,
    Deserialize,
};


#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct DebugUiComponent{
    pub show_profiler: bool,
    pub terrain_wireframe: bool
}

impl DebugUiComponent{
    pub fn create() -> Self {
        DebugUiComponent{
            show_profiler: false,
            terrain_wireframe: false,
        }
    }
}