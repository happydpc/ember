use super::geometry::{
    Geometry,
    GeometryData,
    Vertex
};

#[derive(Debug)]
pub struct TriangleGeom{
    pub data: GeometryData,
}

impl Geometry for TriangleGeom{
    fn create(x: f32, y: f32, z: f32, scale: f32) -> Self{
        TriangleGeom{
            data: GeometryData{
                vertices: vec![
                    Vertex{position: [-0.5 + x, -0.5 + y, 0.0 + z]},
                    Vertex{position: [0.0 + x, 0.5 + y, 0.0 + z]},
                    Vertex{position: [0.5 + x, -0.5 + y, 0.0 + z]},
                ],
                indices: vec![0, 1, 2],
            }
        }
    }
}
