use std::{
    borrow::{
        BorrowMut,
    },
    time::Instant,
    time::Duration,
    cmp::Ordering,
    ops::AddAssign,
};
use std::ops::DerefMut;
use std::borrow::Borrow;

use egui::Window;
use ember_math::{Vector4f, Vector2f};


use crate::core::{
    managers::manager::Manager,
    managers::RenderManager,
    managers::InputManager,
    managers::SceneManager,
    managers::PluginManager,
    managers::scene_manager::{
        SceneManagerUpdateResults,
    },
    systems::ui_systems::EguiState,
};
use crate::core::application::{
    ApplicationState,
    ApplicationIdleState,
};


// window and event management
use winit::{
    event::{
        Event,
        WindowEvent,
        KeyboardInput,
        ElementState, DeviceEvent, MouseScrollDelta,
    },
    event_loop::{
        EventLoop,
        ControlFlow,
    }
};

// egui

use crate::core::plugins::components::AppInterfaceFlag;


// logging
use simple_logger::SimpleLogger;
use log;
use log::LevelFilter;

//////////////////////////////////////////////////
// Done with imports. actual application below  //
//////////////////////////////////////////////////

pub struct Application{
    // state: ApplicationState,
    render_manager: RenderManager,
    scene_manager: SceneManager,
    input_manager: InputManager,
    plugin_manager: PluginManager,
    event_loop: Option<EventLoop<()>>,
    state: Box<dyn ApplicationState>,
    egui_winit_state: egui_winit::State,

    start_instant: Instant,
}

impl Application{

    // startup process
    pub fn create_application(log_level: LevelFilter) -> Self {
        SimpleLogger::new().with_level(log_level).init().unwrap();
        puffin::set_scopes_on(true);

        log::info!("Starting application ...");
        // create other managers
        let (mut render_manager, event_loop) = RenderManager::new();
        let mut scene_manager = SceneManager::new();
        let mut input_manager = InputManager::new();
        let mut plugin_manager = PluginManager::new();

        // initialize other managers
        log::info!("Running manager startup functions ...");
        scene_manager.startup();
        input_manager.startup();
        plugin_manager.startup();
        
        // get egui_winit state from render manager
        let egui_winit_state = render_manager.create_egui_winit_state(&event_loop);

        // set to idle state
        log::info!("Setting application idle state ...");
        scene_manager.create_and_set_staged_scene();

        let mut app = Self{
            render_manager,
            scene_manager,
            input_manager,
            plugin_manager,
            event_loop: Some(event_loop),
            state: Box::new(ApplicationIdleState::create()),
            egui_winit_state,
            start_instant: Instant::now(),
        };

        // prep staged scene
        log::info!("Prepping and activating idle scene ...");
        app.prep_staged_scene();
        app.activate_staged_scene();

        log::info!("Startup complete...");

        app

    }

    // Shutdown process
    fn shutdown(&mut self){
        log::info!("Shutting down application...");
        self.scene_manager.shutdown();
        self.render_manager.shutdown();
        self.input_manager.shutdown();
    }

    // preps a staged scene. this mostly lends the scene to managers so they can do whatever prep they
    // need to do in the ecs world like creating resources and storages etc
    fn prep_staged_scene(&mut self){
        log::info!("Prepping idle scene...");
        {
            let mut _scene = self.scene_manager.get_staged_scene().unwrap();
            let scene = _scene.deref_mut();

            let state: &(dyn ApplicationState) = self.state.borrow();
            state.overlay_interface_on_staged_scene(scene.borrow_mut());
            self.input_manager.prep_staged_scene(scene.borrow_mut());
            self.render_manager.prep_staged_scene(scene.borrow_mut());
        }
    }

    // main game loop
    pub fn run(mut self) {
        log::info!("Running the application...");
        let event_loop = self.event_loop.take().unwrap();

        // overwrite time
        log::info!("Startup time: {:?}", Instant::now().duration_since(self.start_instant));

        self.start_instant = Instant::now();
        let ticks_per_second = 25;
        let skip_ticks = 1000 / ticks_per_second; // tick every 40 ms
        let max_frame_skip = 5;
        let _interpolation: f32 = 0.0;
        let mut next_tick = Instant::now();

        event_loop.run(move |event, _, control_flow| {

            // handle winit event. if it's a redraw request, true is returned, otherwise the event
            // is just processed
            let should_render = self.handle_winit_event(&event, control_flow);
            
            // if should render, do that
            if should_render {
                puffin::GlobalProfiler::lock().new_frame();
                self.render_scene();
            }

            // do physics / non-render updates
            let mut loops = 0;
            while (Instant::now().cmp(&next_tick) == Ordering::Greater) && loops < max_frame_skip {
                // run update in all of the managers
                self.update_managers();

                // run physics
                let mut active_scene = self.scene_manager.get_active_scene().unwrap();
                active_scene.run_update_schedule();

                //
                next_tick.add_assign(Duration::from_millis(skip_ticks));
                loops = loops + 1;
            }


        }); // end of event_loop run
    } // end of run function

    fn update_managers(&mut self){
        let scene_manager_update_result = {
            match self.scene_manager.update(){
                Ok(r) => r,
                Err(e) => panic!("{:?}", e)
            }
        };
        match scene_manager_update_result {
            SceneManagerUpdateResults::NewSceneOpened => {
                self.prep_staged_scene();
                self.activate_staged_scene();
            },
            SceneManagerUpdateResults::NoUpdate => log::debug!("No action required from scene manager"),
        }
        
        // get scene
        let mut active_scene = self.scene_manager.get_active_scene().unwrap();
        
        // run input
        self.input_manager.update(active_scene.borrow_mut());
        self.render_manager.update(active_scene.borrow_mut());
    }

    fn handle_winit_event(
        &mut self,
        event: &winit::event::Event<()>,
        control_flow: &mut ControlFlow
    ) -> bool {
        match event {
            Event::WindowEvent{ref event, ..} => {
                let event_response = {
                    let mut scene = self.scene_manager.get_active_scene().unwrap();
                    let mut world = scene.get_world().unwrap();
                    let egui_ctx = {
                        world.get_resource_mut::<EguiState>().expect("Couldn't get Egui state from world").ctx.clone()
                    };
                    let event_response = self.egui_winit_state.on_event(&egui_ctx, &event);
                    event_response    
                };
                if !event_response.consumed {
                    self.handle_window_event(&event, control_flow);
                }
            },
            Event::DeviceEvent{event, ..} => {
                self.handle_device_event(&event);
            },
            Event::MainEventsCleared => {
                return true;
            },
            _ => ()
        }
        return false;
    }

    fn handle_window_event(
        &mut self,
        event: &winit::event::WindowEvent,
        control_flow: &mut ControlFlow,
    ){  
        match event {

            // close requested
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            },

            // window resized
            WindowEvent::Resized(_) => {
                log::debug!("Window resized...");
                self.render_manager.recreate_swapchain();
                log::info!("Swapchain Recreated...");
            },

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
                    self.input_manager.handle_key_input(Some(virtual_code.clone()));
            },

            WindowEvent::CursorEntered { device_id } => {
                log::debug!("Cursor entered window: todo")
            },

            WindowEvent::CursorLeft { device_id } => {
                log::debug!("Cursor left window: todo")
            },

            WindowEvent::Focused(focused) => {
                if *focused {
                    log::info!("Window Gained Focus");
                } else {
                    log::info!("Window Lost Focus");
                }
            }

            // key modifiers, alt, shift, etc
            WindowEvent::ModifiersChanged(state) => {
                self.input_manager.handle_modifier_change(state.clone());
            },
            _ => () // catch all for window event
        } 
    }

    fn handle_device_event(&mut self, event: &winit::event::DeviceEvent){
        match event{
            DeviceEvent::Button {button, state} => {
                self.input_manager.handle_mouse_button(button, &state);
            },
            DeviceEvent::MouseMotion {delta} => {
                let delta_vec = Vector2f::new(delta.0 as f32, delta.1 as f32);
                self.input_manager.handle_mouse_move(delta_vec);
            },
            DeviceEvent::MouseWheel { delta } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.input_manager.handle_mouse_wheel(*x, *y);
                }
                MouseScrollDelta::PixelDelta(p) => {
                    log::warn!("Mouse Scroll Pixel Delta {p:?} TODO");
                }
            },
            DeviceEvent::Added => log::warn!("Device added: todo"),
            DeviceEvent::Removed => log::warn!("Device removed: todo"),
            _ => (),
        }
    }

    pub fn activate_staged_scene(&mut self){
        self.scene_manager.activate_staged_scene();
    }

    fn render_scene(&mut self){
        let mut current_scene = self.scene_manager.get_active_scene().unwrap();
        let mut egui_winit_state = &mut self.egui_winit_state;
        self.render_manager.draw(
            &mut current_scene,
            &mut egui_winit_state
        );
    }

} // end of class
