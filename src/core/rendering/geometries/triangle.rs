use super::geometry::{
    Geometry,
    GeometryData,
    Vertex
};
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::device::Device;
use std::sync::Arc;


#[derive(Debug)]
pub struct TriangleGeometry{
    pub data: GeometryData,
}

impl Geometry for TriangleGeometry{
    fn create(x: f32, y: f32, z: f32, scale: f32) -> Self{
        let corner_offset = 0.5 * scale;
        TriangleGeometry{
            data: GeometryData{
                vertices: vec![
                    Vertex{position: [-corner_offset + x, -corner_offset + y, 0.0 + z]},
                    Vertex{position: [0.0 + x, corner_offset + y, 0.0 + z]},
                    Vertex{position: [corner_offset + x, -corner_offset + y, 0.0 + z]},
                ],
                indices: vec![0, 1, 2],
                vertex_buffer: None,
                index_buffer: None,
                initialized: false,
            }
        }
    }

    fn initialize(&mut self, device: Arc<Device>){
        // Vertex buffer init
        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                self.data.vertices.clone()
                .iter()
                .cloned(),
            )
            .unwrap()
        };

        // index buffer init
        let index_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            self.data.indices.clone()
            .iter()
            .cloned(),
        ).unwrap();

        self.data.vertex_buffer = Some(vertex_buffer);
        self.data.index_buffer = Some(index_buffer);
        self.data.initialized = true;
    }

    fn vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        self.data.vertex_buffer.clone().unwrap().clone()
    }

    fn index_buffer(&self) -> Arc<CpuAccessibleBuffer<[u16]>> {
        self.data.index_buffer.clone().unwrap().clone()
    }

    fn is_initialized(&self) -> bool {
        self.data.initialized
    }
}
