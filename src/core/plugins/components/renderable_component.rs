use crate::core::{
    rendering::geometries::{
        geometry,
        geometry::{
            Vertex,
            Geometry,
            GeometryData,
        },
        triangle::{
            TriangleGeom,
        }
    }
};

use vulkano::{
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer,
    },
    device::{
        Device
    }
};
use specs::{Component, VecStorage};
use std::sync::Arc;

use log::info;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct RenderableComponent{
    // pub renderable: Mutex<Box<dyn Renderable>>,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[geometry::Vertex]>>>,
    pub geometry: Option<GeometryData>,
    pub initialized: bool,
}

impl RenderableComponent{
    pub fn initialize(&mut self, device: Arc<Device>){
        log::debug!("Initializing renderable component...");
        let geometry = GeometryData{
            vertices: vec![
                Vertex{position: [-0.5, -0.25, 0.0]},
                Vertex{position: [0.5, -0.25, 0.0]},
                Vertex{position: [0.0, 0.5, 0.0]},
            ],
            indices: vec![0, 1, 2],
        };
        self.geometry = Some(geometry);

        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                self.geometry.clone().unwrap().vertices
                .iter()
                .cloned(),
            )
            .unwrap()
        };
        self.vertex_buffer = Some(vertex_buffer);
        
        self.initialized = true;
    }
}
