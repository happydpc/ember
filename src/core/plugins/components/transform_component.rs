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


#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformUiComponent;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct TransformComponent{
    pub global_position: Arc<Mutex<Vector3f>>,
    pub rotation: Arc<Mutex<Matrix4f>>,
    pub scale: Arc<Mutex<f32>>,
}

impl TransformComponent{
    pub fn create_empty() -> Self {
        TransformComponent{
            global_position: Arc::new(Mutex::new(Vector3f::new(0.0, 0.0, 0.0))),
            rotation: Arc::new(Mutex::new(Matrix4f::from_scale(1.0))),
            scale: Arc::new(Mutex::new(1.0)),
        }
    }

    pub fn create(global_pos: Vector3f, rot: Matrix4f, s: f32) -> Self {
        TransformComponent{
            global_position: Arc::new(Mutex::new(global_pos)),
            rotation: Arc::new(Mutex::new(rot)),
            scale: Arc::new(Mutex::new(s)),
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
        self.global_position.lock().expect("Transform can't read its own position.").clone()
    }

    pub fn rotation(&self) -> Matrix4f {
        self.rotation.lock().expect("Transform can't read its own rotation.").clone()
    }

    pub fn scale(&self) -> f32 {
        self.scale.lock().expect("Transform can't read its own scale").clone()
    }
}

impl Default for TransformComponent{
    fn default() -> Self{
        TransformComponent{
            global_position: Arc::new(Mutex::new(Vector3f::new(0.0, 0.0, 0.0))),
            rotation: Arc::new(Mutex::new(Matrix4f::from_scale(1.0))),
            scale: Arc::new(Mutex::new(1.0))
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
                Some(gp) => Arc::new(Mutex::new(gp)),
                None => Arc::new(Mutex::new(Vector3f::new(0.0, 0.0, 0.0))),
            },
            rotation: match self.rotation {
                Some(r) => Arc::new(Mutex::new(r)),
                None => Arc::new(Mutex::new(Matrix4f::from_scale(1.0))),
            },
            scale: match self.scale {
                Some(s) => Arc::new(Mutex::new(s)),
                None => Arc::new(Mutex::new(1.0)),
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