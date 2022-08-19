use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};
use bevy_reflect::{Reflect, FromReflect};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod, Serialize, Deserialize, Reflect, FromReflect)]
pub struct Vertex {
    #[reflect(ignore)]
    pub position: [f32; 3],
}

impl Vertex{
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vertex{
            position: [x, y, z]
        }
    }
}
vulkano::impl_vertex!(Vertex, position);