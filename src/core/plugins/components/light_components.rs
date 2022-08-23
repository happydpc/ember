use bevy_ecs::component::Component;

use ember_math::{Vector3f, Vector4f};
use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::{Reflect, FromReflect};
use bevy_ecs::prelude::ReflectComponent;


#[derive(Component, Clone, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]
pub struct DirectionalLightComponent{
    pub direction: Vector3f,
    pub color: Vector4f,
}

impl DirectionalLightComponent{
    pub fn new(direction: Vector3f, color: Vector4f) -> Self {
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
            color: Vector4f::one(),
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]
pub struct AmbientLightingComponent{
    pub color: Vector3f,
}

impl AmbientLightingComponent{
    pub fn new(color: Vector3f) -> Self{
        AmbientLightingComponent{
            color: color,
        }
    }
}

impl Default for AmbientLightingComponent {
    fn default() -> Self {
        AmbientLightingComponent{
            color: Vector3f::one(),
        }
    }
}