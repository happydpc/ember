use crate::core::rendering::geometries::geometry::Geometry;
use vulkano::{
    device::{
        Device
    }
};
use specs::{Component, VecStorage};
use std::sync::{Arc, Mutex};

#[derive(Component)]
#[storage(VecStorage)]
pub struct RenderableComponent{
    pub geometry: Option<Arc<Mutex<Box<dyn Geometry + Sync + Send>>>>,
}

impl RenderableComponent{

    pub fn create(geometry: Box<dyn Geometry + Sync + Send>) -> Self{
        RenderableComponent{
            geometry: Some(Arc::new(Mutex::new(geometry)))
        }
    }

    pub fn initialize(&mut self, device: Arc<Device>){
        log::debug!("Initializing renderable component...");
        let mut geometry = self.geometry.take().unwrap();//.as_ref();//unwrap();
        geometry.lock().unwrap().initialize(device);
        self.geometry = Some(geometry);
    }

    pub fn geometry(&self) -> Arc<Mutex<Box<dyn Geometry + Sync + Send>>>{
        self.geometry.clone().unwrap().clone()
    }

    pub fn initialized(&self) -> bool {
        match &self.geometry{
            Some(g) => g.lock().unwrap().is_initialized(),
            None => false
        }
    }

}
