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

use ember_math::{Vector4f};


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
    render_manager: RenderManager,
    scene_manager: SceneManager,
    input_manager: InputManager,
    plugin_manager: PluginManager,
    event_loop: Option<EventLoop<()>>,
    surface: Arc<vulkano::swapchain::Surface>,
    state: Box<dyn ApplicationState>,
    egui_winit_state: egui_winit::State,

    log_level: LevelFilter,
    start_instant: Instant,
}

impl Application{

    // startup process
    pub fn create_application(log_level: LevelFilter) -> Self {
        SimpleLogger::new().with_level(log_level).init().unwrap();
        puffin::set_scopes_on(true);

        log::info!("Starting application ...");
        // create other managers
        let mut render_manager = RenderManager::new();
        let mut scene_manager = SceneManager::new();
        let mut input_manager = InputManager::new();
        let mut plugin_manager = PluginManager::new();

        // initialize other managers
        log::info!("Running manager startup functions ...");
        let (event_loop, surface) = render_manager.startup();
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
            surface,
            state: Box::new(ApplicationIdleState::create()),
            egui_winit_state,
            log_level: log_level,
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
        use crate::core::managers::SceneManagerMessagePump;

        use bevy_ecs::event::Events;
        use bevy_reflect::TypeRegistryArc;
        use ember_math::Vector3f;

        let mut _scene = self.scene_manager.get_staged_scene().unwrap();
        let scene = _scene.deref_mut();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<SaveEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<TypeRegistryArc>();

        scene.get_world()
            .unwrap()
            .init_resource::<SceneManagerMessagePump>();

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
            let registry_arc = world.get_resource_mut::<TypeRegistryArc>().unwrap();
            let mut registry = registry_arc.write();
            registry.register::<AppInterfaceFlag>();
            registry.register::<MainMenuComponent>();
            registry.register::<DebugUiComponent>();
            registry.register::<FileSubMenuComponent>();
            registry.register::<TerrainComponent>();
            registry.register::<TransformComponent>();
            registry.register::<TerrainUiComponent>();
            registry.register::<RenderableComponent>();
            registry.register::<GeometryType>();
            registry.register::<GeometryComponent>();
            registry.register::<DirectionalLightComponent>();
            registry.register::<AmbientLightingComponent>();
            registry.register::<CameraComponent>();
            registry.register::<InputComponent>();
        }

        let _MainMenuEntity = scene.get_world()
            .unwrap()
            .spawn()
            .insert(AppInterfaceFlag{})
            .insert(MainMenuComponent{ui: None})
            .insert(FileSubMenuComponent::new());        
        scene.get_world()
            .unwrap()
            .spawn()
            // .insert(TerrainComponent::create(20))
            .insert(TransformComponent::create_empty())
            .insert(TerrainUiComponent{});

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(RenderableComponent::create())
            .insert(GeometryComponent::create(GeometryType::Box))
            .insert(TransformComponent::create_empty())
            .insert(TransformUiComponent{});
        
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
            );

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
            );
        
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
            .insert(TransformUiComponent{});

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(
                DirectionalLightComponent::new(
                    Vector3f::new(0.5, 0.2, 0.8),
                    Vector4f::one()
                )
            );

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(AmbientLightingComponent::new(Vector3f::one()));

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(CameraComponent::create_default())
            .insert(TransformComponent::create_empty())
            .insert(InputComponent::create());
        
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
                self.update_managers();

                // run physics
                let mut active_scene = self.scene_manager.get_active_scene().unwrap();
                active_scene.run_update_schedule();

                next_tick.add_assign(Duration::from_millis(skip_ticks));
                loops = loops + 1;
            }

            // pass events to egui
            {
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
                    Event::MainEventsCleared => {
                        puffin::GlobalProfiler::lock().new_frame();
                        self.render_scene();
                    },
                    _ => ()
                }
            }; // end egui event passing
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

    fn handle_window_event(
        &mut self,
        event: &winit::event::WindowEvent,
        control_flow: &mut ControlFlow,
    ){  
        match event {

            // close requested
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }

            // window resized
            WindowEvent::Resized(_) => {
                log::debug!("Window resized...");
                self.render_manager.recreate_swapchain();
                log::info!("Swapchain Recreated...");
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
                    self.input_manager.handle_key_input(Some(virtual_code.clone()));
            }
            
            // key modifiers, alt, shift, etc
            WindowEvent::ModifiersChanged(state) => {
                self.input_manager.handle_modifier_change(state.clone());
            }

            _ => () // catch all for window event
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
