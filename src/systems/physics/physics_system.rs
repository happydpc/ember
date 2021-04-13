use crate::systems::core::system::System;

pub struct PhysicsSystem{}

impl System for PhysicsSystem{
    fn startup(&mut self){
        println!("Starting physics system...");
    }
    fn shutdown(&mut self){
        println!("Shutting down physics system...");
    }
    fn display_system_name(&self){
        println!("Physics System");
    }
    fn update(&self){
        println!("Updating physics system...");
    }
}
