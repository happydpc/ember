use crate::math::structures::vector::Vector3;

#[derive(Debug, Clone)]
pub struct GeometryData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position);

pub trait Geometry{
    fn create(x: f32, y: f32, z: f32) -> Self where Self: Sized;
}
