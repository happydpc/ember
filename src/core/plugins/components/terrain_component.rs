use bevy_ecs::component::Component;

use crate::core::rendering::geometries::TerrainGeometry;
use vulkano::device::Device;
use std::sync::{Arc, Mutex};
use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::{
    Reflect,
    FromReflect
};


#[derive(Component, Clone, Serialize, Deserialize, Reflect, FromReflect)]
pub struct TerrainUiComponent;

#[derive(Component, Clone, Serialize, Deserialize, Reflect, FromReflect)]
pub struct TerrainComponent{
    #[reflect(ignore)]
    pub geometry: Arc<Mutex<Box<TerrainGeometry>>>,
}

impl TerrainComponent{

    pub fn create(size: usize) -> Self{
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