use crate::core::managers::manager::Manager;

pub struct PhysicsManager{}

impl Manager for PhysicsManager{
    fn startup(&mut self){
        println!("Starting PhysicsManager...");
    }
    fn shutdown(&mut self){
        println!("Shutting down physics manager...");
    }
    fn update(&mut self){
    }
}

impl PhysicsManager{
    pub fn create_new() -> Self{
        println!("Creating PhysicsManager...");
        let phys_sys = PhysicsManager{};
        phys_sys
    }
}
