use std::{
    sync::Arc,
    borrow::{
        BorrowMut,
    },
    cell::RefCell,
};

use crate::core::{
    managers::manager::Manager,
    physics::physics_manager::PhysicsManager,
    rendering::render_manager::RenderManager,
    scene::scene_manager::SceneManager,
    rendering::{
        renderables::{
            renderable::Renderable,
        },
        geometries::{
            geometry::{
                Vertex
            },
        },
    },
};


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

//////////////////////////////////////////////////
// Done with imports. actual application below  //
//////////////////////////////////////////////////

pub struct Application{
    // state: ApplicationState,
    render_manager: Option<RefCell<RenderManager>>,
    physics_manager: Option<RefCell<PhysicsManager>>,
    scene_manager: Option<RefCell<SceneManager>>,
    event_loop: Option<EventLoop<()>>,
    surface: Option<Arc<vulkano::swapchain::Surface<winit::window::Window>>>,
}

impl Manager for Application{
    // startup process
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

        self.render_manager = Some(RefCell::new(render_manager));
        self.physics_manager = Some(RefCell::new(physics_manager));
        self.scene_manager = Some(RefCell::new(scene_manager));
        self.event_loop = Some(event_loop);
        self.surface = Some(surface);

    }

    // Shutdown process
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

    // update process
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
        let event_loop = self.event_loop.take().unwrap();

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
                    match &self.render_manager {
                        Some(manager) => manager.borrow_mut().recreate_swapchain = true,
                        None => log::error!("Render manager not found when trying to recreate swapchain."),
                    }
                }
                Event::RedrawEventsCleared => {
                    // check if render manager exists, and if so, draw
                    match &self.render_manager {
                        Some(manager) => {
                            manager.borrow_mut().draw();
                        },
                        None => log::error!("Render manager does not exist on application manager."),
                    }

                } // end of RedrawEventsCleared arm of event match
                _ => (), // catch all of event match
            } // end of event match
        }); // end of event_loop run
    } // end of run function

    // pub fn get_scene_manager(&self) -> RefCell<SceneManager> {
    //     match self.scene_manager{
    //         Some(manager) => self.scene_manager.
    //     }
    // }
} // end of class
