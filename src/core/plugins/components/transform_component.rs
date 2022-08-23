use bevy_ecs::component::Component;

use ember_math::{
    Matrix4f,
    Vector3f,
};
use std::sync::Arc;
use std::sync::Mutex;
use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::{
    Reflect,
    FromReflect
};
use bevy_ecs::prelude::ReflectComponent;


#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]
pub struct TransformUiComponent;

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]

pub struct TransformComponent{
    pub global_position: Vector3f,
    pub rotation: Matrix4f,
    pub scale: f32,
}

impl TransformComponent{
    pub fn create_empty() -> Self {
        TransformComponent{
            global_position: Vector3f::zero(),
            rotation: Matrix4f::identity(),
            scale: 1.0,
        }
    }

    pub fn create(global_pos: Vector3f, rot: Matrix4f, s: f32) -> Self {
        TransformComponent{
            global_position: global_pos,
            rotation: rot,
            scale: s,
        }
    }

    pub fn start() -> TransformBuilder{
        TransformBuilder{
            global_position: None,
            rotation: None,
            scale: None
        }
    }

    pub fn global_position(&self) -> Vector3f {
        self.global_position
    }

    pub fn rotation(&self) -> Matrix4f {
        self.rotation.clone()
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }
}

impl Default for TransformComponent{
    fn default() -> Self{
        TransformComponent{
            global_position: Vector3f::zero(),
            rotation: Matrix4f::identity(),
            scale: 1.0
        }
    }
}

pub struct TransformBuilder{
    global_position: Option<Vector3f>,
    rotation: Option<Matrix4f>,
    scale: Option<f32>
}

impl TransformBuilder{
    pub fn new() -> Self{
        TransformBuilder{
            global_position: None,
            rotation: None,
            scale: None
        }
    }

    pub fn build(self) -> TransformComponent{
        TransformComponent{
            global_position: match self.global_position {
                Some(gp) => gp,
                None => Vector3f::zero(),
            },
            rotation: match self.rotation {
                Some(r) => r,
                None => Matrix4f::identity(),
            },
            scale: match self.scale {
                Some(s) => s,
                None => 1.0
            }
        }
    }

    pub fn with_global_position(mut self, pos: Vector3f) -> Self {
        self.global_position = Some(pos);
        self
    }

    pub fn with_rotation(mut self, rot: Matrix4f) -> Self {
        self.rotation = Some(rot);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }
}