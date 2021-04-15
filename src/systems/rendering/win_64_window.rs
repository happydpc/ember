use crate::systems::rendering::window::Window;
use glium;
use glium::glutin;

pub struct Win64Window{
    events_loop: glutin::event_loop::EventLoop,
    window_builder: glutin::window::WindowBuilder,
    context_builder: glutin::ContextBuilder,
    display: glium::Display,
}

impl Window for Win64Window{

    fn get_width() -> i16{
        32
    }
    fn get_height() -> i16{
        32
    }
    fn on_update() {

    }
    fn set_event_callback() {

    }
    fn create_window() -> Win64Window {
        let mut win: Win64Window = Win64Window{};
        win.init();
    }
}

impl Win64Window{
    fn init(&mut self){
        self.events_loop = glutin::event_loop::EventLoop::new();
        self.window_builder = glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 780.0))
            .with_title("Leaf");
        self.context_builder = glutin::ContextBuilder::new();
        self.display = glium::Display::new(&self.window_builder, &self.context_builder, &self.events_loop).unwrap();
    }
}
