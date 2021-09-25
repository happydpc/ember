use super::geometry::{
    Geometry,
    GeometryData,
    Vertex
};
use vulkano::buffer::{BufferAccess, CpuAccessibleBuffer, BufferUsage};
use vulkano::device::Device;
use std::sync::Arc;


#[derive(Debug)]
pub struct CubeGeometry{
    pub data: GeometryData,
}

impl Geometry for CubeGeometry{
    fn create(x: f32, y: f32, z: f32, scale: f32) -> Self{
        // dx here is just delta, not associated with x axis
        let dx = 0.5 * scale;

        // bottom plane
        let tl0 = Vertex::new(-dx + x, dx + y, -dx + z);
        let tr0 = Vertex::new(dx + x, dx + y, -dx + z);
        let bl0 = Vertex::new(-dx + x, -dx + y, -dx + z);
        let br0 = Vertex::new(dx + x, -dx + y, -dx + z);

        // top plane
        let tl1 = Vertex::new(-dx + x, dx + y, dx + z);
        let tr1 = Vertex::new(dx + x, dx + y, dx + z);
        let bl1 = Vertex::new(-dx + x, -dx + y, dx + z);
        let br1 = Vertex::new(dx + x, -dx + y, dx + z);

        // store verts.       0    1    2    3    4    5    6    7
        let vertices = vec![tl0, tr0, bl0, br0, tl1, tr1, bl1, br1];

        // top, front, right, back, left, bottom
        let indices = vec![
            4, 5, 7, 7, 6, 4, // top
            6, 7, 3, 3, 2, 6, // front
            7, 5, 1, 1, 3, 7, // right
            5, 4, 0, 0, 1, 5, // back
            4, 6, 2, 2, 0, 4, // left
            2, 3, 0, 0, 1, 2, // bottom
        ];

        CubeGeometry{
            data: GeometryData{
                vertices: vertices,
                indices: indices,
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
