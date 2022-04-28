use specs::{Component, VecStorage};
use cgmath::Vector3;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct VelocityComponent{
    pub velocity: Vector3<f32>,
}
