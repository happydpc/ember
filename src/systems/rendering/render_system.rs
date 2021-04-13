use glium;
use crate::systems::core::system::System;

pub struct RenderSystem{}

impl System for RenderSystem{
    fn startup(&mut self){
        println!("Starting render system...");
    }
    fn shutdown(&mut self){
        println!("Shutting down render system...");
    }
    fn display_system_name(&self){
        println!("Render System");
    }
    fn update(&self){
        println!("Updating render system...");
    }
}
