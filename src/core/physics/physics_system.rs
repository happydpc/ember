use crate::core::systems::system::System;

pub struct PhysicsSystem{}

impl System for PhysicsSystem{
    fn startup(&mut self){
        println!("Starting PhysicsSystem...");
    }
    fn shutdown(&mut self){
        println!("Shutting down physics system...");
    }
    fn update(&self){
    }
}

impl PhysicsSystem{
    pub fn create_new() -> Self{
        println!("Creating PhysicsSystem");
        let phys_sys = PhysicsSystem{};
        phys_sys
    }
}
