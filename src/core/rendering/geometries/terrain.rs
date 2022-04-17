use std::sync::{
    Arc,
    Mutex,
};
use std::borrow::Borrow;

use crate::core::rendering::geometries::Vertex;

use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::BufferUsage;
use vulkano::device::Device;

use noise::{Perlin, NoiseFn};

// #[derive(Debug, Clone)]
pub struct TerrainGeometry{
    pub vertices: Vec<Vertex>,
    pub height_map: Vec<Vec<f64>>,
    pub size: usize,
    pub amplitude: f64,
    pub seed: u32,
    pub noise_fn: Box<dyn NoiseFn<[f64; 2]> + Send + Sync>,
    // pub indices: Vec<u16>,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u16]>>>,
    pub initialized: bool,
}

impl TerrainGeometry{
    pub fn new(size: usize) -> Self{
        TerrainGeometry{
            vertices: Vec::new(),
            height_map: Vec::new(),
            size: size,
            amplitude: 1.0,
            seed: 1,
            noise_fn: Box::new(Perlin::new()),
            // indices: None,
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
        let size = self.size;
        self.vertices.clear();//resize(size*size, Vertex::new(0.0, 0.0, 1.0));

        let noise_fn: &(dyn NoiseFn<[f64; 2]> + Send + Sync) = self.noise_fn.borrow();
        let mut i = 0;
        for x in 0..size {
            for y in 0..size {
                let noise = noise_fn.get([x as f64 * 10.0 as f64, y as f64 * 10.0 as f64]);
                let z = (noise * 20.0) as f32;
                log::info!("x {:?}, y: {:?} z: {:?}", x, y, z);
                // self.height_map[x].push(self.noise_fn.get([x as f64, y as f64])*self.amplitude);
                self.vertices.push(Vertex::new(x as f32, y as f32, z));
                i = i + 1;
            }
        }
        log::info!("Terrain size: {:?}", self.vertices.len());
    }

    pub fn set_noise_fn(&mut self, noise_fn: Box<dyn NoiseFn<[f64;2 ]> + Send + Sync>) {
        self.noise_fn = noise_fn;
    }

    pub fn initialize(&mut self, device: Arc<Device>){
        log::info!("Terrain init...");
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
        // let index_buffer = CpuAccessibleBuffer::from_iter(
        //     device.clone(),
        //     BufferUsage::all(),
        //     false,
        //     self.indices.clone()
        //     .iter()
        //     .cloned(),
        // ).unwrap();

        self.vertex_buffer = Some(vertex_buffer);
        // self.index_buffer = Some(index_buffer);
        self.initialized = true;
    }
}