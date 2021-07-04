use crate::math::structures::vector::Vector3;
use super::geometry::GeometryData;

#[derive(Debug)]
pub struct TriangleGeom{
    pub data: GeometryData,
}

impl TriangleGeom{
    pub fn create(x: f32, y: f32, z: f32) -> Self{
        TriangleGeom{
            data: GeometryData{
                vertices: vec![
                    Vector3{position: [-0.5 + x, -0.5 + y, 0.0 + z]},
                    Vector3{position: [0.0 + x, 0.5 + y, 0.0 + z]},
                    Vector3{position: [0.5 + x, -0.5 + y, 0.0 + z]},
                ],
                indices: vec![0, 1, 2],
            }
        }
    }
}
