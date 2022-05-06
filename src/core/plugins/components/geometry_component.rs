
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use vulkano::{
    buffer::CpuAccessibleBuffer,
    device::Device,
    buffer::BufferUsage,
};
use bevy_ecs::component::Component;

use crate::core::rendering::geometries::Vertex;


#[derive(Clone, Serialize, Deserialize)]
pub enum GeometryType{
    Triangle,
    Box,
    Plane,
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct GeometryComponent{
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    #[serde(skip, default="GeometryComponent::default_vertex_buffer")]
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    #[serde(skip, default="GeometryComponent::default_index_buffer")]
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u16]>>>,
    pub initialized: bool,
    pub geometry_type: GeometryType,
}

impl GeometryComponent{
    pub fn create(t: GeometryType) -> Self {
        GeometryComponent{
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
            initialized: false,
            geometry_type: t,
        }
    }
    pub fn default_vertex_buffer() -> Option<Arc<CpuAccessibleBuffer<[Vertex]>>> {
        None
    }
    pub fn default_index_buffer() -> Option<Arc<CpuAccessibleBuffer<[u16]>>> {
        None
    }

    // ---- 
    pub fn initialize(&mut self, device: Arc<Device>){
        // Vertex buffer init
        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                self.vertices.clone()
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
            self.indices.clone()
            .iter()
            .cloned(),
        ).unwrap();

        log::debug!("Setting vbuffer right here.");

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.initialized = true;
    }

    pub fn vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        self.vertex_buffer.clone().unwrap().clone()
    }

    pub fn index_buffer(&self) -> Arc<CpuAccessibleBuffer<[u16]>> {
        self.index_buffer.clone().unwrap().clone()
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }  
}