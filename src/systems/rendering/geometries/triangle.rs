use crate::math::structures::vector::Vector3;

pub struct TriangleGeom{
    pub vertices: Vec<Vector3>,
    pub indices: [u16; 3],
}

impl TriangleGeom{
    pub fn create(x: f64, y: f64, z: f64) -> Self{
        TriangleGeom{
            vertices: vec![
                Vector3{position: [-0.5 + x, -0.5 + y, 0.0 + z]},
                Vector3{position: [0.0 + x, 0.5 + y, 0.0 + z]},
                Vector3{position: [0.5 + x, -0.5 + y, 0.0 + z]},
            ],
            indices: [0, 1, 2],
        }
    }
}
