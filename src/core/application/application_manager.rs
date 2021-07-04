use std::cell::RefCell;

use crate::core::{
    managers::manager::Manager,
    physics::physics_manager::PhysicsManager,
    rendering::render_manager::RenderManager,
    scene::scene_manager::SceneManager,
};

use glium;
use glium::Surface;
use glium::glutin;

pub struct DisplayWrapper(glium::Display);

use crate::core::rendering::renderables::{
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
        render_manager: RefCell<RenderManager>,
        physics_manager: RefCell<PhysicsManager>,
        scene_manager: RefCell<SceneManager>,
    },
}

// in a way this is actually kind of acting like a service locator
impl ApplicationState{
    pub fn get_render_manager(&self) -> &RefCell<RenderManager>{
        match self{
            ApplicationState::InitializedState{render_manager, ..} => render_manager,
            _ => panic!("Cannot access render_manager on uninitialized application."),
        }
    }
    pub fn get_physics_manager(&self) -> &RefCell<PhysicsManager>{
        match self {
            ApplicationState::InitializedState{physics_manager, ..} => physics_manager,
            _ => panic!("Cannot acces physics_manager on uninitialized application."),
        }
    }
    pub fn get_scene_manager(&self) -> &RefCell<SceneManager>{
        match self {
            ApplicationState::InitializedState{scene_manager, ..} => scene_manager,
            _ => panic!("Cannot acces scene manager on uninitialized application"),
        }
    }
}

impl Manager for Application{
    fn startup(&mut self){
        println!("Starting application ...");
        let _state = ApplicationState::InitializedState{
            render_manager: RefCell::new(RenderManager::create_new()),
            physics_manager: RefCell::new(PhysicsManager::create_new()),
            scene_manager: RefCell::new(SceneManager::create_new()),
        };
        self.state = _state;
        // TODO : consider implementing this using ECS so that managers can be quickly iterated
        self.state.get_physics_manager().borrow_mut().startup();
        self.state.get_render_manager().borrow_mut().startup();
        self.state.get_scene_manager().borrow_mut().startup();

    }
    fn shutdown(&mut self){
        println!("Shutting down application...");
        // TODO : Definitely find a better way to access the managers
        self.state.get_physics_manager().borrow_mut().shutdown();
        self.state.get_render_manager().borrow_mut().shutdown();
        self.state.get_scene_manager().borrow_mut().shutdown();
    }
    fn update(&mut self){
        // TODO : Will the core app update do anything? should run just call update on loop
        // and then have this iterate over the managers and update? seems like an unecessary
        // layer to have the run function just be a thin wrapper around this.
        self.state.get_physics_manager().borrow_mut().update();
        self.state.get_scene_manager().borrow_mut().update();
        self.state.get_render_manager().borrow_mut().update();
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
    pub fn run(mut self) {
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

            // create and draw a triangle
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
