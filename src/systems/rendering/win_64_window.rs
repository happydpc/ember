use std::rc::{Weak, Rc};
use std::cell::RefCell;
use crate::systems::events::event_system::{Observer, Subject};
use crate::systems::events::event::Event;
use crate::systems::rendering::window::Window;
use crate::systems::rendering::context::Context;
use glium;

pub struct Win64Window{
    pub context: Context,
    observers: Vec<Rc<RefCell<dyn Observer>>>,
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
        let mut win: Win64Window = Win64Window{
            context: Context::create_new(),
            observers: Vec::new(),
        };

        win.init();
        win
    }
}

// TODO : move this down to the context?
impl Subject for Win64Window{
    fn register(&mut self, observer: Rc<RefCell<dyn Observer>>){
        self.observers.push(observer);
    }
    fn notify(&mut self, event: &Event){
        for obs in self.observers.iter(){
            obs.borrow_mut().on_notify(event);
        }
    }
}

impl Win64Window {
    fn init(&mut self){
    }
}
