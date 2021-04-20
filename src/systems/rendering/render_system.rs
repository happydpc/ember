use crate::systems::core::system::System;
use crate::systems::rendering::window::Window;
// eventually abstract this out or use an enum to decide which window to use
use crate::systems::rendering::win_64_window::Win64Window;

pub struct RenderSystem{
    // again  abstract this out
    //window: Win64Window,
}

impl System for RenderSystem{
    fn startup(&mut self){
        println!("Starting render system...");
        //Win64Window::create_window();
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
