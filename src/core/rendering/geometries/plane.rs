use super::geometry::{
    Geometry,
    GeometryData,
    Vertex
};
use vulkano::buffer::{BufferAccess, CpuAccessibleBuffer, BufferUsage};
use vulkano::device::Device;
use std::sync::Arc;


#[derive(Debug)]
pub struct PlaneGeometry{
    pub data: GeometryData,
}

impl Geometry for PlaneGeometry{
    fn create(x: f32, y: f32, z: f32, scale: f32) -> Self{
        let corner_offset = 0.5 * scale;

        // top left, top right, bottom left, bottom right
        let tl = Vertex{position: [-corner_offset + x, corner_offset + y, 0.0 + z]};
        let tr = Vertex{position: [corner_offset + x, corner_offset + y, 0.0 + z]};
        let bl = Vertex{position: [-corner_offset + x, -corner_offset + y, 0.0 + z]};
        let br = Vertex{position: [corner_offset + x, -corner_offset + y, 0.0 + z]};

        PlaneGeometry{
            data: GeometryData{
                vertices: vec![tl, tr, bl, br],
                indices: vec![0, 1, 3, 3, 2, 0],
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
