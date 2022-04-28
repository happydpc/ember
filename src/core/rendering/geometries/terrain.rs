use std::sync::{
    Arc,
};
use std::borrow::Borrow;

use crate::core::rendering::geometries::Vertex;
use crate::core::plugins::components::GeometryComponent;

use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::BufferUsage;
use vulkano::device::Device;

use noise::{NoiseFn, OpenSimplex};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TerrainGeometry{
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub height_map: Vec<Vec<f64>>,
    pub size: usize,
    pub amplitude: f64,
    pub seed: u32,
    #[serde(skip, default="TerrainGeometry::default_noise_fn")]
    pub noise_fn: Box<dyn NoiseFn<[f64; 2]> + Send + Sync>,
    #[serde(skip, default="GeometryComponent::default_vertex_buffer")]
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    #[serde(skip, default="GeometryComponent::default_index_buffer")]
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u16]>>>,
    pub initialized: bool,
}

impl TerrainGeometry{
    pub fn new(size: usize) -> Self{
        TerrainGeometry{
            vertices: Vec::new(),
            indices: Vec::new(),
            height_map: Vec::new(),
            size: size,
            amplitude: 1.0,
            seed: 1,
            noise_fn: Box::new(OpenSimplex::new()),
            vertex_buffer: None,
            index_buffer: None,
            initialized: false
        }
    }

    pub fn set_size(&mut self, size: usize){
        self.size = size;
        self.vertices.resize(size*size, Vertex::new(0.0, 0.0, 0.0));
        self.generate_terrain();
    }

    pub fn generate_terrain(&mut self){
        self.height_map.clear();
        let size = self.size as u16;
        self.vertices.clear();
        self.indices.clear();
        let noise_fn: &(dyn NoiseFn<[f64; 2]> + Send + Sync) = self.noise_fn.borrow();
        let mut i = 0;
        for x in 0..size {
            for y in 0..size {
                let noise = noise_fn.get([x as f64, y as f64]);
                let z = (noise * self.amplitude) as f32;
                self.vertices.push(
                    Vertex{
                        position: [x as f32, y as f32, z as f32]
                    }
                );
                i = i + 1;
            }
        }

        for y in 0..(size-1) {
            for x in 0..(size-1) {
                let ix = (y * size + x) as u16;
                self.indices.push(ix);
                self.indices.push(ix + 1);
                self.indices.push(ix + size + 1);

                self.indices.push(ix + size);
                self.indices.push(ix);
                self.indices.push(ix + size + 1);

            }
        }
    }

    pub fn set_noise_fn(&mut self, noise_fn: Box<dyn NoiseFn<[f64;2 ]> + Send + Sync>) {
        self.noise_fn = noise_fn;
    }

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

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.initialized = true;
    }

    fn default_noise_fn() -> Box<dyn NoiseFn<[f64; 2]> + Send + Sync>{
        Box::new(OpenSimplex::new())
    }
}