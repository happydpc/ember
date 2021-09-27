use specs::{Component, VecStorage};
// use crate::math::structures::vector::Vector3;
use cgmath::{
    Matrix4,
    Vector3,
};


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct TransformComponent{
    pub global_position: Vector3<f64>,
    pub rotation: Matrix4<f64>,
}

impl TransformComponent{
    pub fn create_empty() -> Self {
        TransformComponent{
            global_position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Matrix4::from_scale(1.0),
        }
    }

    pub fn create(global_pos: Vector3<f64>, rot: Matrix4<f64>) -> Self {
        TransformComponent{
            global_position: global_pos,
            rotation: rot,
        }
    }
}
