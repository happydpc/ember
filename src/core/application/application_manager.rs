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
            triangle::Triangle,
        },
        geometries::{
            geometry::{
                Vertex
            },
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

// vulkano
use vulkano::{
    swapchain::{
        SwapchainCreationError,
        AcquireError,
        Swapchain,
    },
    swapchain,
    command_buffer::{
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        DynamicState,
        SubpassContents,
    },
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer,
    },
    sync::{
        FlushError,
        GpuFuture,
    },
    sync
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
        let mut event_loop = self.event_loop.take().unwrap();

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
                    // Move this to the top of the event loop
                    match &self.render_manager {
                        Some(manager) => {
                            // just temporarily store this for easier typing
                            let mut _manager = manager.borrow_mut();

                            let mut _framebuffers = _manager.framebuffers.take().unwrap();
                            let mut _pipeline = _manager.pipeline.take().unwrap();
                            let mut _dynamic_state = _manager.dynamic_state.take().unwrap();
                            let mut _device = _manager.device.take().unwrap();
                            let mut _queue = _manager.queue.take().unwrap();
                            let mut _surface = _manager.surface.take().unwrap();
                            let mut _render_pass = _manager.render_pass.take().unwrap();
                            let mut _swapchain = _manager.swapchain.take().unwrap();
                            let mut _previous_frame_end = Some(_manager.previous_frame_end.take().unwrap());

                            // _previous_frame_end.take().unwrap().cleanup_finished();

                            // if the swapchain needs to be recreated
                            let _recreate_swapchain = _manager.recreate_swapchain.clone();
                            if _recreate_swapchain {

                                let dimensions: [u32; 2] = _surface.window().inner_size().into();
                                let (new_swapchain, new_images) =
                                match _swapchain.recreate().dimensions(dimensions).build() {
                                    Ok(r) => r,
                                    // This error tends to happen when the user is manually resizing the window.
                                    // Simply restarting the loop is the easiest way to fix this issue.
                                    Err(SwapchainCreationError::UnsupportedDimensions) => return,
                                    Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                                };

                                _manager.recreate_swapchain = false;
                                _swapchain = new_swapchain;
                            } // end of if on swapchain recreation

                            // acquire an image from the swapchain
                            let (image_num, suboptimal, acquire_future) =
                                match swapchain::acquire_next_image(_swapchain.clone(), None) {
                                    Ok(r) => r,
                                    Err(AcquireError::OutOfDate) => {
                                        _manager.recreate_swapchain = true;
                                        return;
                                    }
                                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                                };

                            if suboptimal {
                                _manager.recreate_swapchain = true;
                            }

                            // this is the default color of the framebuffer
                            let clear_values = vec![[0.2, 0.2, 0.2, 1.0].into()];

                            // create a command buffer builder
                            let mut builder = AutoCommandBufferBuilder::primary(
                                _device.clone(),
                                _queue.family(),
                                CommandBufferUsage::OneTimeSubmit,
                            )
                            .unwrap();

                            // create a vertex buffer
                            // TODO : replace this with real geometries
                            let vertex_buffer = {
                                CpuAccessibleBuffer::from_iter(
                                    _device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    [
                                        Vertex {
                                            position: [-0.5, -0.25, 0.0],
                                        },
                                        Vertex {
                                            position: [0.0, 0.5, 0.0],
                                        },
                                        Vertex {
                                            position: [0.25, -0.1, 0.0],
                                        },
                                    ]
                                    .iter()
                                    .cloned(),
                                )
                                .unwrap()
                            };

                            // prepare contents of command buffer using the builder
                            builder
                                .begin_render_pass(
                                    _framebuffers[image_num].clone(),
                                    SubpassContents::Inline,
                                    clear_values,
                                )
                                .unwrap()
                                .draw(
                                    _pipeline.clone(),
                                    &_dynamic_state,
                                    vertex_buffer.clone(),
                                    (),
                                    (),
                                    vec![],
                                )
                                .unwrap()
                                .end_render_pass()
                                .unwrap();

                            // actually build command buffer now
                            let command_buffer = builder.build().unwrap();

                            // now get future state and try to draw
                            // let x: u32 = _previous_frame_end.take().unwrap();
                            let future = _previous_frame_end
                                .take()
                                .unwrap()
                                .join(acquire_future)
                                .then_execute(_queue.clone(), command_buffer)
                                .unwrap()
                                .then_swapchain_present(_queue.clone(), _swapchain.clone(), image_num)
                                .then_signal_fence_and_flush();

                            match future {
                                Ok(future) => {
                                    _previous_frame_end = Some(future.boxed());
                                }
                                Err(FlushError::OutOfDate) => {
                                    _manager.recreate_swapchain = true;
                                    _previous_frame_end = Some(sync::now(_device.clone()).boxed());
                                }
                                Err(e) => {
                                    log::error!("Failed to flush future: {:?}", e);
                                    _previous_frame_end = Some(sync::now(_device.clone()).boxed());
                                }
                            }

                        // put things back on to the manager
                        // just temporarily store this for easier typing
                        _manager.framebuffers = Some(_framebuffers);
                        _manager.pipeline = Some(_pipeline);
                        _manager.dynamic_state = Some(_dynamic_state);
                        _manager.device = Some(_device);
                        _manager.queue = Some(_queue);
                        _manager.surface = Some(_surface);
                        _manager.render_pass = Some(_render_pass);
                        _manager.swapchain = Some(_swapchain);
                        _manager.previous_frame_end = _previous_frame_end;

                        //  should be end of draw call at this block
                        } // end of some arm on render manager match
                        None => log::error!("Render manager doesn't exist at time of draw."),
                    } // end of render manager match

                } // end of RedrawEventsCleared
                _ => (), // catch all of match
            } // end of event match
        }); // end of event_loop run
    }

}
