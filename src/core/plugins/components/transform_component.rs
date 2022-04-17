use specs::{Component, VecStorage};
// use crate::math::structures::vector::Vector3;
use cgmath::{
    Matrix4,
    Vector3,
};


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct TransformComponent{
    pub global_position: Vector3<f32>,
    pub rotation: Matrix4<f32>,
    pub scale: f32,
}

impl TransformComponent{
    pub fn create_empty() -> Self {
        TransformComponent{
            global_position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Matrix4::from_scale(1.0),
            scale: 1.0,
        }
    }

    pub fn create(global_pos: Vector3<f32>, rot: Matrix4<f32>, s: f32) -> Self {
        TransformComponent{
            global_position: global_pos,
            rotation: rot,
            scale: s,
        }
    }
}

impl Default for TransformComponent{
    fn default() -> Self{
        TransformComponent{
            global_position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Matrix4::from_scale(1.0),
            scale: 1.0
        }
    }
}