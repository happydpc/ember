use std::cell::RefCell;
use std::sync::Arc;
use std::borrow::BorrowMut;

use crate::core::{
    managers::manager::Manager,
    physics::physics_manager::PhysicsManager,
    rendering::render_manager::RenderManager,
    scene::scene_manager::SceneManager,
    rendering::{
        renderables::{
            renderable::Renderable,
            triangle::Triangle,
        },
    },
};

pub struct DisplayWrapper(glium::Display);

// window and event management
use winit::{
    event::{
        Event,
        WindowEvent,
    },
    event_loop::{
        EventLoop,
        ControlFlow,
    }
};


//logging
use simple_logger::SimpleLogger;
use log;


pub struct Application{
    // state: ApplicationState,
    render_manager: Option<RefCell<RenderManager>>,
    physics_manager: Option<RefCell<PhysicsManager>>,
    scene_manager: Option<RefCell<SceneManager>>,
    event_loop: Option<EventLoop<()>>,
    surface: Option<Arc<vulkano::swapchain::Surface<winit::window::Window>>>,
}

impl Manager for Application{
    fn startup(&mut self){
        SimpleLogger::new().init().unwrap();
        log::info!("Starting application ...");
        // create other managers
        let mut render_manager = RenderManager::create_new();
        let mut physics_manager = PhysicsManager::create_new();
        let mut scene_manager = SceneManager::create_new();

        // initialize other managers
        let (event_loop, surface) = render_manager.startup();
        physics_manager.startup();
        scene_manager.startup();

        // assign managers into options on self
        if self.render_manager.is_none() {
            self.render_manager = Some(RefCell::new(render_manager));
        }
        if self.physics_manager.is_none() {
            self.physics_manager = Some(RefCell::new(physics_manager));
        }
        if self.scene_manager.is_none() {
            self.scene_manager = Some(RefCell::new(scene_manager));
        }
        // assign event loop and surface
        if self.event_loop.is_none() {
            self.event_loop = Some(event_loop);
        }
        if self.surface.is_none() {
            self.surface = Some(surface);
        }

    }
    fn shutdown(&mut self){
        log::info!("Shutting down application...");
        match &self.physics_manager {
            Some(manager) => manager.borrow_mut().shutdown(),
            None => log::error!("No physics manager to shutdown."),
        }
        match &self.scene_manager {
            Some(manager) => manager.borrow_mut().shutdown(),
            None => log::error!("No scene manager to shutdown."),
        }
        match &self.render_manager {
            Some(manager) => manager.borrow_mut().shutdown(),
            None => log::error!("No render manager to shut down"),
        }
    }
    fn update(&mut self){
        match &self.physics_manager {
            Some(manager) => manager.borrow_mut().update(),
            None => log::error!("No physics manager to shutdown."),
        }
        match &self.scene_manager {
            Some(manager) => manager.borrow_mut().update(),
            None => log::error!("No scene manager to shutdown."),
        }
        match &self.render_manager {
            Some(manager) => manager.borrow_mut().update(),
            None => log::error!("No render manager to shut down"),
        }
    }
}

impl Application{
    // called by the client when they want to create an application
    pub fn create_application() -> Self{
        Self {
            render_manager: None,
            physics_manager: None,
            scene_manager: None,
            event_loop: None,
            surface: None,
        }
    }

    // main game loop
    pub fn run(mut self) {
        log::info!("Running the application...");
        let mut event_loop = self.event_loop.unwrap();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    // recreate_swapchain = true;
                }
                Event::RedrawEventsCleared => {
                    log::info!("would be drawing here");
                }
                _ => (),
            }
        });
    }

}
