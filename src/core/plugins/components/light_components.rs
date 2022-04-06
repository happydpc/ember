use specs::{Component, VecStorage};
use cgmath::Vector3;

#[derive(Component)]
#[storage(VecStorage)]
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