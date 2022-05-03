use specs::{Component, VecStorage};
// use crate::math::structures::vector::Vector3;
use cgmath::{
    Matrix4,
    Vector3,
};
use std::sync::Arc;
use std::sync::Mutex;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct TransformUiComponent;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct TransformComponent{
    pub global_position: Arc<Mutex<Vector3<f32>>>,
    pub rotation: Arc<Mutex<Matrix4<f32>>>,
    pub scale: Arc<Mutex<f32>>,
}

impl TransformComponent{
    pub fn create_empty() -> Self {
        TransformComponent{
            global_position: Arc::new(Mutex::new(Vector3::new(0.0, 0.0, 0.0))),
            rotation: Arc::new(Mutex::new(Matrix4::from_scale(1.0))),
            scale: Arc::new(Mutex::new(1.0)),
        }
    }

    pub fn create(global_pos: Vector3<f32>, rot: Matrix4<f32>, s: f32) -> Self {
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

    pub fn global_position(&self) -> Vector3<f32> {
        self.global_position.lock().expect("Transform can't read its own position.").clone()
    }

    pub fn rotation(&self) -> Matrix4<f32> {
        self.rotation.lock().expect("Transform can't read its own rotation.").clone()
    }

    pub fn scale(&self) -> f32 {
        self.scale.lock().expect("Transform can't read its own scale").clone()
    }
}

impl Default for TransformComponent{
    fn default() -> Self{
        TransformComponent{
            global_position: Arc::new(Mutex::new(Vector3::new(0.0, 0.0, 0.0))),
            rotation: Arc::new(Mutex::new(Matrix4::from_scale(1.0))),
            scale: Arc::new(Mutex::new(1.0))
        }
    }
}

pub struct TransformBuilder{
    global_position: Option<Vector3<f32>>,
    rotation: Option<Matrix4<f32>>,
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
                None => Arc::new(Mutex::new(Vector3::new(0.0, 0.0, 0.0))),
            },
            rotation: match self.rotation {
                Some(r) => Arc::new(Mutex::new(r)),
                None => Arc::new(Mutex::new(Matrix4::from_scale(1.0))),
            },
            scale: match self.scale {
                Some(s) => Arc::new(Mutex::new(s)),
                None => Arc::new(Mutex::new(1.0)),
            }
        }
    }

    pub fn with_global_position(mut self, pos: Vector3<f32>) -> Self {
        self.global_position = Some(pos);
        self
    }

    pub fn with_rotation(mut self, rot: Matrix4<f32>) -> Self {
        self.rotation = Some(rot);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }
}