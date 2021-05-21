use specs::{Component, VecStorage};
use crate::math::structures::vector::Vector3;


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct TransformComponent{
    pub global_position: Vector3,
    pub euler_angles: Vector3,
}
