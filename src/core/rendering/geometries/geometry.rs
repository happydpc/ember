use crate::math::structures::vector::Vector3;

#[derive(Debug)]
pub struct GeometryData {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<u16>,
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position);
