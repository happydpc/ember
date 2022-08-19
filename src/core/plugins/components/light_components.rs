use bevy_ecs::component::Component;

use ember_math::Vector3f;
use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::{Reflect, FromReflect};


#[derive(Component, Clone, Serialize, Deserialize)]
pub struct DirectionalLightComponent{
    pub direction: Vector3f,
    pub color: [f32; 3],
}

impl DirectionalLightComponent{
    pub fn new(direction: Vector3f, color: [f32; 3]) -> Self {
        DirectionalLightComponent{
            direction: direction,
            color: color,
        }
    }
}

impl Default for DirectionalLightComponent {
    fn default() -> Self { 
        DirectionalLightComponent{
            direction: Vector3f::new(1.0, 1.0, -1.0),
            color: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct AmbientLightingComponent{
    pub color: [f32; 3],
}

impl AmbientLightingComponent{
    pub fn new(color: [f32; 3]) -> Self{
        AmbientLightingComponent{
            color: color,
        }
    }
}

impl Default for AmbientLightingComponent {
    fn default() -> Self {
        AmbientLightingComponent{
            color: [1.0, 1.0, 1.0],
        }
    }
}