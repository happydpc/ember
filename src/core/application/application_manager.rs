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

use ember_math::{Vector3f, Vector4f};


use crate::core::{
    managers::manager::Manager,
    managers::RenderManager,
    managers::InputManager,
    managers::SceneManager,
    scene::{
        Scene,
        Active,
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
        let mut scene_manager = SceneManager::create_new();
        let mut input_manager = InputManager::create_new();

        // initialize other managers
        log::info!("Running manager startup functions ...");
        let (event_loop, surface) = render_manager.startup();
        scene_manager.startup();
        input_manager.startup();

        // set to idle state
        log::info!("Setting application idle state ...");
        let state: &(dyn ApplicationState) = self.state.borrow();
        scene_manager.load_scene_interface(state.scene_interface_path());
        
        // store managers and other created things
        self.render_manager = Some(RefCell::new(render_manager));
        self.scene_manager = Some(RefCell::new(scene_manager));
        self.input_manager = Some(RefCell::new(input_manager));
        self.event_loop = Some(event_loop);
        self.surface = Some(surface);

        // prep staged scene
        log::info!("Prepping and activating idle scene ...");
        self.prep_staged_scene();
        self.temp_prep(); // here until i fix serialization again and actually have a functional editor state
        self.activate_staged_scene();

        log::info!("Startup complete...");
    }

    // Shutdown process
    fn shutdown(&mut self){
        log::info!("Shutting down application...");
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
        log::debug!("Prepping idle scene...");
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
        use crate::core::plugins::components::{
            DebugUiComponent,
            TerrainUiComponent,
            TerrainComponent,
            TransformComponent,
            TransformUiComponent,
            DirectionalLightComponent,
            CameraComponent,
            InputComponent,
            GeometryType,
            GeometryComponent,
            RenderableComponent,
            AmbientLightingComponent,
            MainMenuComponent,
            FileSubMenuComponent,
        };
        use crate::core::events::project_events::SaveEvent;
        use crate::core::events::project_events::CreateProjectEvent;
        use crate::core::events::project_events::CloseProjectEvent;
        use crate::core::events::project_events::OpenProjectEvent;
        use crate::core::events::menu_messages::MenuMessage;
        use crate::core::events::terrain_events::TerrainRecalculateEvent;
        use bevy_ecs::event::Events;
        use bevy_reflect::TypeRegistryArc;
        use ember_math::Vector3f;

        let mut scene_manager = self.get_scene_manager().unwrap();
        let mut _scene = scene_manager.get_staged_scene().unwrap();
        let scene = _scene.deref_mut();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<SaveEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<TypeRegistryArc>();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<CreateProjectEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<CloseProjectEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<OpenProjectEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<MenuMessage<MainMenuComponent>>>();
        scene.get_world()
            .unwrap()
            .init_resource::<Events<MenuMessage<FileSubMenuComponent>>>();
        
        scene.get_world()
            .unwrap()
            .init_resource::<Events<TerrainRecalculateEvent>>();

        {
            let mut world = scene.get_world().unwrap();
            let mut registry_arc = world.get_resource_mut::<TypeRegistryArc>().unwrap();
            let mut registry = registry_arc.write();
            registry.register::<AppInterfaceFlag>();
            registry.register::<MainMenuComponent>();
            registry.register::<DebugUiComponent>();
            registry.register::<FileSubMenuComponent>();
            registry.register::<TerrainComponent>();
            registry.register::<TransformComponent>();
            registry.register::<TerrainUiComponent>();
            registry.register::<RenderableComponent>();
            registry.register::<GeometryComponent>();
            registry.register::<DirectionalLightComponent>();
            registry.register::<AmbientLightingComponent>();
            registry.register::<CameraComponent>();
            registry.register::<InputComponent>();

        }

        let MainMenuEntity = scene.get_world()
            .unwrap()
            .spawn()
            .insert(AppInterfaceFlag{})
            .insert(MainMenuComponent{ui: None})
            .insert(DebugUiComponent::create())
            // .marked::<SimpleMarker<SerializerFlag>>()
            .id();
        
        scene.get_world()
            .unwrap()
            .spawn()
            .insert(FileSubMenuComponent::with_parent(MainMenuEntity));    


        scene.get_world()
            .unwrap()
            .spawn()
            // .insert(TerrainComponent::create(20))
            .insert(TransformComponent::create_empty())
            .insert(TerrainUiComponent{})
            .id();

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(RenderableComponent::create())
            .insert(GeometryComponent::create(GeometryType::Box))
            .insert(TransformComponent::create_empty())
            .insert(TransformUiComponent{})
            .id();

        
        scene.get_world()
            .unwrap()
            .spawn()
            .insert(RenderableComponent::create())
            .insert(GeometryComponent::create(GeometryType::Box))
            .insert(
                TransformComponent::start()
                    .with_global_position(Vector3f::new(0.0, 2.0, 0.0))
                    .with_scale(0.3)
                    .build()
            )
            .id();

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(RenderableComponent::create())
            .insert(GeometryComponent::create(GeometryType::Box))
            .insert(
                TransformComponent::start()
                    .with_global_position(Vector3f::new(0.0, 0.0, 2.0))
                    .with_scale(0.3)
                    .build()
            )
            .id();
        
        scene.get_world()
            .unwrap()
            .spawn()
            .insert(RenderableComponent::create())
            .insert(GeometryComponent::create(GeometryType::Box))
            .insert(
                TransformComponent::start()
                    .with_global_position(Vector3f::new(2.0, 0.0, 0.0))
                    .with_scale(0.1)
                    .build()
            )
            .insert(TransformUiComponent{})
            .id();

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(
                DirectionalLightComponent::new(
                    Vector3f::new(0.5, 0.2, 0.8),
                    Vector4f::one()
                )
            )
            .id();

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(AmbientLightingComponent::new(Vector3f::one()))
            .id();

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(CameraComponent::create_default())
            .insert(TransformComponent::create_empty())
            .insert(InputComponent::create())
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
                // get scene
                let scene_manager = self.get_scene_manager().unwrap();
                let mut active_scene = scene_manager.get_active_scene().unwrap();

                // run input 
                self.get_input_manager().unwrap().update(active_scene.borrow_mut());
                
                // run physics
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
