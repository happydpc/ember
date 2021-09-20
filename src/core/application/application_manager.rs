use std::{
    sync::Arc,
    borrow::{
        BorrowMut,
    },
    cell::{
        RefCell,
        RefMut
    },
};

use crate::core::{
    managers::manager::Manager,
    physics::physics_manager::PhysicsManager,
    rendering::render_manager::RenderManager,
    scene::{
        scene::{
            Scene,
            Initialized,
        },
        scene_manager::{
            SceneManager,
        },
    },
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
use log::LevelFilter;

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

    log_level: LevelFilter,
}

impl Manager for Application{
    // startup process
    fn startup(&mut self){
        SimpleLogger::new().with_level(self.log_level).init().unwrap();
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

        log::info!("Startup complete...");
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
    pub fn create_application(log_level: Option<LevelFilter>) -> Self{
        Self {
            render_manager: None,
            physics_manager: None,
            scene_manager: None,
            event_loop: None,
            surface: None,
            log_level: log_level.unwrap_or(LevelFilter::Info),
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
                    self.run_managers();
                } // end of RedrawEventsCleared arm of event match
                _ => (), // catch all of event match
            } // end of event match
        }); // end of event_loop run
    } // end of run function

    pub fn get_scene_manager(&self) -> Option<RefMut<SceneManager>> {
        match &self.scene_manager{
            Some(manager) => Some(manager.borrow_mut()),
            None => None,
        }
    }

    fn run_managers(&self){
        match &self.scene_manager {
            Some(scene_manager) => {
                let scene_manager = scene_manager.borrow_mut();
                let mut current_scene = scene_manager.get_active_scene().unwrap();
                // check if render manager exists, and if so, draw
                match &self.render_manager {
                    Some(manager) => {
                        manager.borrow_mut().prep_scene(current_scene.borrow_mut());
                        manager.borrow_mut().draw(current_scene.borrow_mut());
                    },
                    None => log::error!("Render manager does not exist on application manager."),
                }
            },
            None => {
                log::error!("Scene manager does not exist on application manager.");
            },
        }
    }
} // end of class
