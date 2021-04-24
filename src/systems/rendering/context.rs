use std::rc::Rc;
use std::cell::RefCell;
use crate::systems::events::event_system::{Observer, Subject};
use crate::systems::events::event::Event;
use glium;
use glium::glutin;

// loose wrapper around the display. Saw this in a post and copied
// it but i'm honestly not sure why this is done. will mess with this later
pub struct DisplayWrapper(glium::Display);

enum ContextState{
    UninitializedState {},
    InitializedState {observers: Vec<Rc<RefCell<dyn Observer>>>, display: DisplayWrapper},
    // UninitializedState(Vec<Rc<RefCell<dyn Observer>>>),
    // InitializedState(Vec<Rc<RefCell<dyn Observer>>>, DisplayWrapper),
}

impl ContextState{
    pub fn get_display(&self) -> &DisplayWrapper{
        match self {
            ContextState::InitializedState{observers, display} => display,
            _ => panic!("Called get_display on an uninitialized context state."),
        }
    }
    pub fn get_observers(&mut self) -> &mut Vec<Rc<RefCell<dyn Observer>>> {
        match self {
            ContextState::InitializedState{observers, display} => observers,
            _ => panic!("Bungus."),
        }
    }
}

pub struct ContextBaseState{
    observers: Vec<Rc<RefCell<dyn Observer>>>,
}

pub struct ContextInitState{
    observers: Vec<Rc<RefCell<dyn Observer>>>,
    display: DisplayWrapper,
}

// stores the related gluting context info
pub struct Context{
    state: ContextState,
}

// implementation for context. mostly includes creation.
impl Context{

    // create the event loop and builders, create a display, then
    // create a context and return it with an event loop and a display
    pub fn create_new() -> Self {
        let _state = ContextState::UninitializedState{
        };
        println!("Creating Context");
        Context{
            state: _state,
        }
    }

    pub fn init(&mut self, event_loop: &glutin::event_loop::EventLoop<()>) {
        println!("Initializing Context");
        let window_builder = glutin::window::WindowBuilder::new();
        let context_builder = glutin::ContextBuilder::new();
        let display = DisplayWrapper(
            glium::Display::new(window_builder, context_builder, &event_loop).unwrap(),
        );
        let observers = Vec::new();
        let _state = ContextState::InitializedState{
            observers: observers,
            display: display,
        };
        self.state = _state;
    }

    pub fn run(self, event_loop: glutin::event_loop::EventLoop<()>){
        event_loop.run(move |event, _, control_flow| {
            let next_frame_time =
                std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
            *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    }
                    _ => return,
                },
                glutin::event::Event::NewEvents(cause) => match cause {
                    glutin::event::StartCause::ResumeTimeReached { .. } => (),
                    glutin::event::StartCause::Init => (),
                    _ => return,
                },
                _ => return,
            }
            // update scene etc

        });
    }
}

impl Subject for Context{
    fn register(&mut self, observer: Rc<RefCell<dyn Observer>>){
        self.state.get_observers().push(observer);
    }
    fn notify(&mut self, event: &Event){
        for obs in self.state.get_observers().iter(){
            obs.borrow_mut().on_notify(event);
        }
    }
}
