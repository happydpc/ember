use specs::{Component, VecStorage};
use crate::math::structures::vector::Vector3;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct VelocityComponent{
    pub velocity: Vector3,
}
