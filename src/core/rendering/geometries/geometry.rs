use crate::math::structures::vector::Vector3;

#[derive(Debug)]
pub struct GeometryData {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<u16>,
}
