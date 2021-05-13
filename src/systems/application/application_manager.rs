use std::cell::RefCell;
use crate::systems::core::system::System;
use crate::systems::physics::physics_system::PhysicsSystem;
use crate::systems::rendering::render_system::RenderSystem;
use glium;
use glium::Surface;
use glium::glutin;
pub struct DisplayWrapper(glium::Display);

use crate::systems::rendering::renderables::{
    renderable::Renderable,
    triangle::Triangle,
};

use std::borrow::BorrowMut;

//
//
//

pub struct Application{
    state: ApplicationState,
}

enum ApplicationState{
    UninitializedState {},
    InitializedState {
        render_system: RefCell<RenderSystem>,
        physics_system: RefCell<PhysicsSystem>,
    },
}

impl ApplicationState{
    pub fn get_render_system(&self) -> &RefCell<RenderSystem>{
        match self{
            ApplicationState::InitializedState{render_system, ..} => render_system,
            _ => panic!("Cannot access render_system on uninitialized application."),
        }
    }
    pub fn get_physics_system(&self) -> &RefCell<PhysicsSystem>{
        match self {
            ApplicationState::InitializedState{physics_system, ..} => physics_system,
            _ => panic!("Cannot acces physics_system on uninitialized application."),
        }
    }
}

impl System for Application{
    fn startup(&mut self){
        println!("Starting application ...");
        let _state = ApplicationState::InitializedState{
            render_system: RefCell::new(RenderSystem::create_new()),
            physics_system: RefCell::new(PhysicsSystem::create_new()),
        };
        self.state = _state;
        // TODO : consider implementing this using ECS so that systems can be quickly iterated
        self.state.get_physics_system().borrow_mut().startup();
        self.state.get_render_system().borrow_mut().startup();

    }
    fn shutdown(&mut self){
        println!("Shutting down application...");
        // TODO : Definitely find a better way to access the systems
        self.state.get_physics_system().borrow_mut().shutdown();
        self.state.get_render_system().borrow_mut().shutdown();
    }
    fn update(&self){
        // TODO : Will the core app update do anything? should run just call update on loop
        // and then have this iterate over the systems and update? seems like an unecessary
        // layer to have the run function just be a thin wrapper around this.
        self.state.get_physics_system().borrow_mut().update();
        self.state.get_render_system().borrow_mut().update();
    }
}

impl Application{
    // called by the client when they want to create an application
    pub fn create_application() -> Self{
        let _state = ApplicationState::UninitializedState{};
        Self{
            state: _state
        }
    }

    // main game loop
    pub fn run(self) {
        println!("Running the application...");
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Leaf");
        let context_builder = glutin::ContextBuilder::new();
        let event_loop = glutin::event_loop::EventLoop::new();
        let display = DisplayWrapper(
            glium::Display::new(window_builder, context_builder, &event_loop).unwrap(),
        );

        event_loop.run(move |event, _, control_flow| {

            // update scene
            self.update();
            let mut a = Triangle::create(0.0, 0.0, 0.0);
            let mut target = display.0.draw();
            target.clear_color(0.05, 0.1, 0.05, 1.0);
            a.initialize(&display.0);
            a.draw(target.borrow_mut());
            target.finish().unwrap();

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
        });
    }

}
