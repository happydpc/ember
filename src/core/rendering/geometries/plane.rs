use super::geometry::{
    Geometry,
    GeometryData,
    Vertex
};
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::device::Device;
use std::sync::Arc;


#[derive(Debug, Default)]
pub struct PlaneGeometry{
    pub data: GeometryData,
}

impl Geometry for PlaneGeometry{
    fn create() -> Self{
        let corner_offset = 0.5;

        // top left, top right, bottom left, bottom right
        let tl = Vertex{position: [-corner_offset, corner_offset, 0.0]};
        let tr = Vertex{position: [corner_offset, corner_offset, 0.0]};
        let bl = Vertex{position: [-corner_offset, -corner_offset, 0.0]};
        let br = Vertex{position: [corner_offset, -corner_offset, 0.0]};

        PlaneGeometry{
            data: GeometryData{
                vertices: vec![tl, tr, bl, br],
                indices: vec![0, 1, 3, 2, 0, 3],
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
