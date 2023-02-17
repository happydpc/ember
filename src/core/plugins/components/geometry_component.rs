
use std::sync::Arc;

use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::{
    buffer::CpuAccessibleBuffer,
    buffer::BufferUsage,
};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::ReflectComponent;

use bevy_reflect::{
    Reflect, FromReflect
};
use bevy_reflect::ReflectSerialize;
use bevy_reflect::ReflectDeserialize;
use serde::{Deserialize, Serialize};
use crate::core::rendering::geometries::Vertex;


#[derive(Reflect, FromReflect, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum GeometryType{
    Triangle,
    Box,
    Plane,
}

impl Default for GeometryType{
    fn default() -> Self {
        GeometryType::Box
    }
}

#[derive(Component, Clone, Reflect, FromReflect)]
#[reflect(Component)]
pub struct GeometryComponent{
    #[reflect(ignore)]
    pub vertices: Vec<Vertex>,
    #[reflect(ignore)]
    pub indices: Vec<u32>,
    // #[serde(skip, default="GeometryComponent::default_vertex_buffer")]
    #[reflect(ignore)]
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    // #[serde(skip, default="GeometryComponent::default_index_buffer")]
    #[reflect(ignore)]
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u32]>>>,
    pub initialized: bool,
    pub geometry_type: GeometryType,
}

impl Default for GeometryComponent{
    fn default() -> Self {
        GeometryComponent{
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
            initialized: false,
            geometry_type: GeometryType::Triangle,
        }
    }
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
    pub fn default_index_buffer() -> Option<Arc<CpuAccessibleBuffer<[u32]>>> {
        None
    }

    // ---- 
    pub fn initialize(&mut self, memory_allocator: Arc<StandardMemoryAllocator>){
        // Vertex buffer init
        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                &memory_allocator,
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                self.vertices.clone()
            )
            .unwrap()
        };

        // index buffer init
        let index_buffer = CpuAccessibleBuffer::from_iter(
            &memory_allocator,
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            self.indices.clone()
        ).unwrap();

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.initialized = true;
    }

    pub fn vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        self.vertex_buffer.clone().unwrap().clone()
    }

    pub fn index_buffer(&self) -> Arc<CpuAccessibleBuffer<[u32]>> {
        self.index_buffer.clone().unwrap().clone()
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }  
}