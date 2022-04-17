use specs::{Component, HashMapStorage};

use crate::core::rendering::geometries::geometry::Geometry;
use crate::core::rendering::geometries::TerrainGeometry;

use vulkano::device::Device;

use std::sync::{Arc, Mutex};


#[derive(Component)]
#[storage(HashMapStorage)]
pub struct TerrainUiComponent;

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct TerrainComponent{
    pub geometry: Arc<Mutex<Box<TerrainGeometry>>>,
}

impl TerrainComponent{

    pub fn new(size: usize) -> Self{
        TerrainComponent{
            geometry: Arc::new(Mutex::new(Box::new(TerrainGeometry::new(size))))
        }
    }

    pub fn initialize(&mut self, device: Arc<Device>){
        log::debug!("Initializing renderable component...");
        let geometry = self.geometry.clone();//.as_ref();//unwrap();
        geometry.lock().unwrap().initialize(device);
    }

    pub fn initialized(&self) -> bool {
        self.geometry.lock().unwrap().initialized
    }

    pub fn set_size(&self, size: usize){
        self.geometry.clone().lock().unwrap().size = size;
    }

    pub fn get_size(&self) -> usize {
        self.geometry.clone().lock().unwrap().size
    }
}