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
    input::input_manager::InputManager,
    scene::{
        scene_manager::{
            SceneManager,
        },
    },
};


// window and event management
use winit::{
    event::{
        Event,
        WindowEvent,
        KeyboardInput,
        ElementState,
        StartCause,
        VirtualKeyCode
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
    input_manager: Option<RefCell<InputManager>>,
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
        let mut input_manager = InputManager::create_new();

        // initialize other managers
        let (event_loop, surface) = render_manager.startup();
        physics_manager.startup();
        scene_manager.startup();
        input_manager.startup();

        self.render_manager = Some(RefCell::new(render_manager));
        self.physics_manager = Some(RefCell::new(physics_manager));
        self.scene_manager = Some(RefCell::new(scene_manager));
        self.input_manager = Some(RefCell::new(input_manager));
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
            None => log::error!("No render manager to shutdown."),
        }
        match &self.input_manager {
            Some(manager) => manager.borrow_mut().shutdown(),
            None => log::error!("No input manager to shutdown.")
        }
    }

    // update process
    fn update(&mut self){
        match &self.physics_manager {
            Some(manager) => manager.borrow_mut().update(),
            None => log::error!("No physics manager to update."),
        }
        match &self.scene_manager {
            Some(manager) => manager.borrow_mut().update(),
            None => log::error!("No scene manager to update."),
        }
        match &self.render_manager {
            Some(manager) => manager.borrow_mut().update(),
            None => log::error!("No render manager to update."),
        }
        match &self.input_manager {
            Some(manager) => manager.borrow_mut().update(),
            None => log::error!("No input manager to update."),
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
            input_manager: None,
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
                Event::WindowEvent { event, .. } => match event {

                    // close requested
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    // window resized
                    WindowEvent::Resized(_) => {
                        log::debug!("Window resized...");
                        match &self.render_manager {
                            Some(manager) => {
                                manager.borrow_mut().recreate_swapchain();
                            },
                            None => log::error!("Render manager not found when trying to recreate swapchain."),
                        }
                    }

                    // keyboard input
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(virtual_code),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                        } => {
                            match &self.input_manager {
                                Some(manager) => manager.borrow_mut().handle_key_input(Some(virtual_code)),
                                None => log::error!("Key detected, but no input manager is loaded..."),
                            };
                    }

                    // key modifiers, alt, shift, etc
                    WindowEvent::ModifiersChanged(state) => {
                        match &self.input_manager{
                            Some(manager) => manager.borrow_mut().handle_modifier_change(Some(state)),
                            None => log::error!("Key modifier change detected, but no input manager is loaded..."),
                        };
                    }

                    _ => () // catch all for window event
                }

                Event::RedrawEventsCleared => {
                    self.run_managers();
                }


                _ => (), // catch all of event match
            } // end of event match
        // events have been matched

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
