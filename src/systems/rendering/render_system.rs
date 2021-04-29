use crate::systems::core::system::System;
// use crate::systems::rendering::window::Window;
// eventually abstract this out or use an enum to decide which window to use
// use crate::systems::rendering::win_64_window::Win64Window;

pub struct RenderSystem{
    // again  abstract this out
    // pub window: Win64Window,
}

impl System for RenderSystem{
    fn startup(&mut self){
        println!("Starting RenderSystem...");
        // self.window.init();
    }
    fn shutdown(&mut self){
        println!("Shutting down render system...");
    }
    fn update(&self){
    }
}
impl RenderSystem{
    // TODO : add a parameter for window type
    pub fn create_new() -> Self{
        println!("Creating RenderSystem");
        let render_sys = RenderSystem{
            // window: Win64Window::create_new(),
        };
        render_sys
    }
    pub fn run(&mut self) {
        // self.window.run();
    }
}
