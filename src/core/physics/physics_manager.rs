use crate::core::managers::manager::Manager;
use crate::core::scene::{Scene, Initialized};

pub struct PhysicsManager{}

impl Manager for PhysicsManager{
    fn startup(&mut self){
        log::info!("Starting PhysicsManager...");
    }
    fn shutdown(&mut self){
        log::info!("Shutting down physics manager...");
    }
    fn update(&mut self, scene: &mut Scene<Initialized>){
    }
}

impl PhysicsManager{
    pub fn create_new() -> Self{
        log::info!("Creating PhysicsManager...");
        let phys_sys = PhysicsManager{};
        phys_sys
    }
}
