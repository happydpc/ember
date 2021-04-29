use crate::systems::rendering::window::Window;
use crate::systems::rendering::context::Context;
use glium;
use glium::glutin;
// use std::cell::RefCell;
// use std::rc::Rc;
// use std::mem;

pub struct Win64Window{
    event_loop: glutin::event_loop::EventLoop<()>,
    pub context: Context,
}

impl Window for Win64Window {

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
    fn create_new() -> Win64Window {
        let _event_loop = glutin::event_loop::EventLoop::new();
        let _context = Context::create_new();
        let win: Win64Window = Win64Window{
            event_loop: _event_loop,
            context: _context,
        };
        win
    }
}

impl Win64Window {
    pub fn init(&mut self){
        println!("Initializing Window");
        self.context.init(&self.event_loop);
    }
    pub fn run(&mut self){
        // let _replacement = glutin::event_loop::EventLoop::new();
        // self.context.run(mem::replace(&mut self.event_loop, _replacement));
    }
}
