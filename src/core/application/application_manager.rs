use std::{
    sync::Arc,
    borrow::{
        BorrowMut,
    },
    cell::{
        RefCell,
        RefMut
    },
    time::Instant,
    time::Duration,
    cmp::Ordering,
    ops::AddAssign,
};
use std::ops::DerefMut;
use std::borrow::Borrow;




use crate::core::{
    managers::manager::Manager,
    managers::physics_manager::PhysicsManager,
    managers::render_manager::RenderManager,
    managers::input_manager::InputManager,
    scene::{
        Scene,
        Active,
        scene_manager::{
            SceneManager,
        },
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
        ElementState,
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
    render_manager: Option<RefCell<RenderManager>>,
    physics_manager: Option<RefCell<PhysicsManager>>,
    scene_manager: Option<RefCell<SceneManager>>,
    input_manager: Option<RefCell<InputManager>>,
    event_loop: Option<EventLoop<()>>,
    surface: Option<Arc<vulkano::swapchain::Surface<winit::window::Window>>>,
    state: Box<dyn ApplicationState>,

    log_level: LevelFilter,
    start_instant: Instant,
}

impl Manager for Application{
    // startup process
    fn startup(&mut self){
        SimpleLogger::new().with_level(self.log_level).init().unwrap();
        puffin::set_scopes_on(true);

        log::info!("Starting application ...");
        // create other managers
        let mut render_manager = RenderManager::create_new();
        let mut physics_manager = PhysicsManager::create_new();
        let mut scene_manager = SceneManager::create_new();
        let mut input_manager = InputManager::create_new();

        // initialize other managers
        log::info!("Running manager startup functions ...");
        let (event_loop, surface) = render_manager.startup();
        physics_manager.startup();
        scene_manager.startup();
        input_manager.startup();

        // set to idle state
        log::info!("Setting application idle state ...");
        let state: &(dyn ApplicationState) = self.state.borrow();
        scene_manager.load_scene_interface(state.scene_interface_path());
        
        // store managers and other created things
        self.render_manager = Some(RefCell::new(render_manager));
        self.physics_manager = Some(RefCell::new(physics_manager));
        self.scene_manager = Some(RefCell::new(scene_manager));
        self.input_manager = Some(RefCell::new(input_manager));
        self.event_loop = Some(event_loop);
        self.surface = Some(surface);

        // prep staged scene
        log::info!("Prepping and activating idle scene ...");
        self.prep_staged_scene();
        self.temp_prep();
        self.activate_staged_scene();

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
    fn update(&mut self, scene: &mut Scene<Active>){
        match &self.input_manager {
            Some(manager) => manager.borrow_mut().update(scene),
            None => log::error!("No input manager to update."),
        }
        match &self.physics_manager {
            Some(manager) => manager.borrow_mut().update(scene),
            None => log::error!("No physics manager to update."),
        }
        match &self.scene_manager {
            Some(manager) => manager.borrow_mut().update(scene),
            None => log::error!("No scene manager to update."),
        }
        match &self.render_manager {
            Some(manager) => manager.borrow_mut().update(scene),
            None => log::error!("No render manager to update."),
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
            start_instant: Instant::now(),
            state: Box::new(ApplicationIdleState::create()),
        }
    }

    // preps a staged scene
    fn prep_staged_scene(&mut self){

        let mut scene_manager = self.get_scene_manager().unwrap();
        let mut _scene = scene_manager.get_staged_scene().unwrap();
        let scene = _scene.deref_mut();

        match &self.input_manager {
            Some(manager) => manager.borrow_mut().prep_staged_scene(scene.borrow_mut()),
            None => log::error!("No input manager to prep scene."),
        }
        match &self.render_manager {
            Some(manager) => manager.borrow_mut().prep_staged_scene(scene.borrow_mut()),
            None => log::error!("No render manager to prep scene."),
        }
    }

    fn temp_prep(&mut self){
        use crate::core::plugins::components::DebugUiComponent;
        let mut scene_manager = self.get_scene_manager().unwrap();
        let mut _scene = scene_manager.get_staged_scene().unwrap();
        let scene = _scene.deref_mut();

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(AppInterfaceFlag{})
            .insert(DebugUiComponent::create())
            // .marked::<SimpleMarker<SerializerFlag>>()
            .id();
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

            let mut loops = 0;
            while (Instant::now().cmp(&next_tick) == Ordering::Greater) && loops < max_frame_skip {
                let scene_manager = self.get_scene_manager().unwrap();
                let mut active_scene = scene_manager.get_active_scene().unwrap();
                self.get_physics_manager().unwrap().update(active_scene.borrow_mut());
                self.get_input_manager().unwrap().update(active_scene.borrow_mut());
                active_scene.run_update_schedule();

                next_tick.add_assign(Duration::from_millis(skip_ticks));
                loops = loops + 1;
            }

            // pass events to egui
            let egui_consumed_event = {
                let scene_manager = self.get_scene_manager().unwrap();
                let mut scene = scene_manager.get_active_scene().unwrap();
                let mut world = scene.get_world().unwrap();
                let egui_ctx = {
                    world.get_resource_mut::<EguiState>().expect("Couldn't get Egui state from world").ctx.clone()
                };
                let mut egui_winit = {
                    world.get_resource_mut::<egui_winit::State>().expect("Couldn't get egui_winit state from world")
                };
                // let egui_ctx = state.ctx.clone();
                match event{
                    Event::WindowEvent{ref event, ..} => {
                        egui_winit.on_event(&egui_ctx, &event)
                    },
                    _ => false
                }
            };

            // if egui didn't need it
            if !egui_consumed_event{
                self.handle_event(&event, control_flow);
            }

            // if it's a draw, draw
            match event{
                Event::MainEventsCleared => {
                    puffin::GlobalProfiler::lock().new_frame();
                    self.render_scene();
                },
                _ => (),
            }
        }); // end of event_loop run
    } // end of run function

    fn handle_event(
        &mut self,
        event: &winit::event::Event<()>,
        control_flow: &mut ControlFlow,
    ){  
        match event {
            Event::WindowEvent { event, .. } => {
                let egui_consumed_event = false;//egui_winit.on_event(&egui_ctx, &event);
                if !egui_consumed_event{
                    match event {

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
                                    log::info!("Swapchain Recreated...");
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
                                    Some(manager) => manager.borrow_mut().handle_key_input(Some(virtual_code.clone())),
                                    None => log::error!("Key detected, but no input manager is loaded..."),
                                };
                        }
                        
                        // key modifiers, alt, shift, etc
                        WindowEvent::ModifiersChanged(state) => {
                            match &self.input_manager{
                                Some(manager) => manager.borrow_mut().handle_modifier_change(state.clone()),
                                None => log::error!("Key modifier change detected, but no input manager is loaded..."),
                            };
                        }

                        _ => () // catch all for window event
                    }
                }
            }

            // Event::MainEventsCleared => {
            //     self.render_scene();
            // }


            _ => (), // catch all of event match
        } // end of event match
    }

    pub fn create_scene(&mut self) -> i16{
        // get scene manager
        let mut scene_manager = self.get_scene_manager().unwrap();
        let id = scene_manager.generate_and_register_scene();
        id // return id
    }

    // pub fn stage_scene(&mut self, scene_id: i16) {
    pub fn initialize_egui_state_on_staged_scene(&mut self){// scene: &mut Scene<Active>){
        let mut scene_manager = self.get_scene_manager().unwrap();

        // create scene and register egui component
        // scene_manager.stage_scene(scene_id);

        let mut _scene = scene_manager.get_staged_scene().unwrap();
        let scene = _scene.deref_mut();

        // get required egui data
        let render_manager = self.get_render_manager().unwrap();
        let (egui_ctx, egui_painter) = render_manager.initialize_egui();
        let egui_winit = render_manager.create_egui_winit_state();
        let egui_state = EguiState{ctx: egui_ctx, painter: egui_painter};
        scene.insert_resource(egui_state);
        scene.insert_resource(egui_winit);
    }

    pub fn activate_staged_scene(&self){
        let mut scene_manager = self.get_scene_manager().unwrap();
        scene_manager.activate_staged_scene();
    }

    pub fn get_scene_manager(&self) -> Option<RefMut<SceneManager>> {
        match &self.scene_manager{
            Some(manager) => Some(manager.borrow_mut()),
            None => None,
        }
    }

    pub fn get_physics_manager(&self) -> Option<RefMut<PhysicsManager>> {
        match &self.physics_manager {
            Some(manager) => Some(manager.borrow_mut()),
            None => None,
        }
    }

    pub fn get_input_manager(&self) -> Option<RefMut<InputManager>> {
        match &self.input_manager {
            Some(manager) => Some(manager.borrow_mut()),
            None => None,
        }
    }

    pub fn get_render_manager(&self) -> Option<RefMut<RenderManager>> {
        match &self.render_manager{
            Some(manager) => Some(manager.borrow_mut()),
            None => None,
        }
    }

    fn render_scene(&self){
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
