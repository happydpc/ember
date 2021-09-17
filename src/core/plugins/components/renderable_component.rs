use crate::core::{
    rendering::geometries::{
        geometry::{
            Vertex,
            Geometry,
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
    pub vertex_buffer: Option<CpuAccessibleBuffer<()>>,
    pub geometry: Option<TriangleGeom>,
}

impl RenderableComponent{
    pub fn initialize(&mut self, device: Arc<Device>){
        log::info!("initializing a component");
    }
}
