use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use vulkano::{
    buffer::CpuAccessibleBuffer,
    device::Device,
};

#[derive(Debug, Clone)]
pub struct GeometryData{
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u16]>>>,
    pub initialized: bool,
}

pub struct Inactive;
pub struct Initialized{
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u16]>>>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex{
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vertex{
            position: [x, y, z]
        }
    }
}

vulkano::impl_vertex!(Vertex, position);

pub trait Geometry{
    fn create(x: f32, y: f32, z: f32, size: f32) -> Self where Self: Sized;
    fn initialize(&mut self, device: Arc<Device>);
    fn vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[Vertex]>>;
    fn index_buffer(&self) -> Arc<CpuAccessibleBuffer<[u16]>>;
    fn is_initialized(&self) -> bool;
}
