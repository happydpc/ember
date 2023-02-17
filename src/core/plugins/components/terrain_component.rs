use bevy_ecs::component::Component;

use crate::core::rendering::geometries::TerrainGeometry;
use vulkano::{memory::allocator::StandardMemoryAllocator};
use std::sync::{Arc, Mutex};
use serde::{
    Serialize,
    Deserialize,
};
use bevy_reflect::{
    Reflect,
    FromReflect
};
use bevy_ecs::prelude::ReflectComponent;

#[derive(Component, Default, Clone, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]
pub struct TerrainUiComponent;

#[derive(Component, Clone, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]
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

    pub fn initialize(&mut self, memory_allocator: Arc<StandardMemoryAllocator>){
        log::debug!("Initializing renderable component...");
        let geometry = self.geometry.clone();//.as_ref();//unwrap();
        geometry.lock().unwrap().initialize(memory_allocator);
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

impl Default for TerrainComponent {
    fn default() -> Self {
        TerrainComponent::create(16)
    }
}