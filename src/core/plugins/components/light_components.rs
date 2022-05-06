use bevy_ecs::component::Component;

use cgmath::Vector3;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct DirectionalLightComponent{
    pub direction: Vector3<f32>,
    pub color: [f32; 3],
}

impl DirectionalLightComponent{
    pub fn new(direction: Vector3<f32>, color: [f32; 3]) -> Self {
        DirectionalLightComponent{
            direction: direction,
            color: color,
        }
    }
}

impl Default for DirectionalLightComponent {
    fn default() -> Self { 
        DirectionalLightComponent{
            direction: Vector3::new(1.0, 1.0, -1.0),
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