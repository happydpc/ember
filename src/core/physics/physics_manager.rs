use crate::core::systems::system::System;

pub struct PhysicsManager{}

impl System for PhysicsManager{
    fn startup(&mut self){
        println!("Starting PhysicsManager...");
    }
    fn shutdown(&mut self){
        println!("Shutting down physics system...");
    }
    fn update(&self){
    }
}

impl PhysicsManager{
    pub fn create_new() -> Self{
        println!("Creating PhysicsManager...");
        let phys_sys = PhysicsManager{};
        phys_sys
    }
}
