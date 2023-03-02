use bevy_ecs::{
    prelude::Schedule,
    prelude::SystemStage,
    world::World,
    schedule::Stage,
    system::Resource,
};


use bevy_reflect::TypeRegistryArc;
use bevy_ecs::prelude::Events;
use crate::core::events::project_events::SaveEvent;
use crate::core::managers::SceneManagerMessagePump;
use crate::core::events::project_events::CreateProjectEvent;
use crate::core::events::project_events::CloseProjectEvent;
use crate::core::events::project_events::OpenProjectEvent;
use crate::core::events::menu_messages::MenuMessage;
use crate::core::events::terrain_events::TerrainRecalculateEvent;
use crate::core::systems::initalize_editor_interface;
use crate::core::systems::ui_systems::EntityInspectionUiSystem;
use crate::core::systems::ui_systems::PanelInitSystem;

use std::{
    cell::{
        RefCell,
        RefMut,
    },
};

use crate::core::managers::input_manager::KeyInputQueue;
use crate::core::plugins::components::*;
use crate::core::systems::{
    input_systems::{
        CameraMoveSystem,
    },
    render_systems::{
        RenderableInitializerSystem,
        RenderableDrawSystem,
        CameraUpdateSystem,
        DirectionalLightingSystem,
        AmbientLightingSystem,
        RenderableAssemblyStateModifierSystem,
    },
    ui_systems::{
        CameraUiSystem,
        FileSubMenuSystem,
        TransformUiSystem,
        SceneGraphUiSystem
    },
    CameraInitSystem,
    TerrainInitSystem,
    TerrainDrawSystem,
    TerrainAssemblyStateModifierSystem,
    TerrainUiSystem,
    GeometryInitializerSystem,
    SceneSerializationSystem,
    TerrainUpdateSystem,
    ShowNewProjectWindow,
    ShowOpenProjectWindow,
    ProjectCreationSystem,
    OpenProjectSystem,
};


#[derive(Resource, Default)]
pub struct TypeRegistryResource(pub TypeRegistryArc);


pub struct Scene<S>{
    pub world: Option<RefCell<World>>,
    pub state: S,
}

pub struct Active{
    pub device_loaded: bool,
    pub update_schedule: Option<Schedule>,
    pub render_schedule: Option<Schedule>,
    pub teardown_schedule: Option<Schedule>,
}
pub struct Inactive;
pub struct Staged{
    pub setup_schedule: Option<Schedule>,
    pub teardown_schedule: Option<Schedule>,
}

impl Scene<Inactive> {
    pub fn new() -> Self {
        Scene{
            world: None,
            state: Inactive,
        }
    }
}


impl Scene<Staged> {
    pub fn new() -> Self {
        let mut scene = Scene{
            world: Some(RefCell::new(World::new())),
            state: Staged{
                setup_schedule: None,
                teardown_schedule: None,
            }
        };
        scene.create_setup_schedule();
        scene.create_teardown_schedule();
        Self::init_resources(&mut scene);
        scene
    }

    fn init_resources(scene: &mut Scene<Staged>){
        scene.get_world()
            .unwrap()
            .init_resource::<Events<SaveEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<TypeRegistryResource>();

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
            .init_resource::<Events<MenuMessage<FileSubMenuComponent>>>();
        
        scene.get_world()
            .unwrap()
            .init_resource::<Events<TerrainRecalculateEvent>>();

        {
            let mut world = scene.get_world().unwrap();
            let registry_arc = world.get_resource_mut::<TypeRegistryResource>().unwrap();
            let mut registry = registry_arc.0.write();
            registry.register::<AppInterfaceFlag>();
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
    }

    fn create_setup_schedule(&mut self){
        let mut schedule = Schedule::default();
        log::info!("Creating setup schedule.");
        schedule
        .add_stage("ui_init", SystemStage::parallel()
            .with_system(initalize_editor_interface)
        )
        .add_stage("geometry_init", SystemStage::parallel()
            .with_system(GeometryInitializerSystem)
            .with_system(TerrainInitSystem)
        ).add_stage("final_init", SystemStage::parallel()
            .with_system(CameraInitSystem)
            .with_system(RenderableInitializerSystem)
        );
        self.state.setup_schedule = Some(schedule);
    }
    
    fn create_teardown_schedule(&mut self){
        let schedule = Schedule::default();
        self.state.teardown_schedule = Some(schedule);
    }

    pub fn run_setup_schedule(&mut self){
        log::info!("Running setup schedule for staged scene...");
        let mut schedule = self.state.setup_schedule.take().expect("No setup schedule");
        log::debug!("about to run schedule");
        schedule.run(&mut *self.get_world().unwrap());
        log::debug!("Reinserting schedule");
        self.state.setup_schedule = Some(schedule);
    }

    pub fn run_teardown_schedule(&mut self){
        log::info!("Running teardown schedule...");
        let mut schedule = self.state.teardown_schedule.take().expect("no teardown schedule");
        schedule.run(&mut *self.get_world().unwrap());
        self.state.teardown_schedule = Some(schedule);
    }

    pub fn get_world(&mut self) -> Option<RefMut<World>>{
        match &self.world {
            Some(world) => Some(world.borrow_mut()),
            None => None,
        }
    }

    pub fn insert_resource<R>(&mut self, r: R)
    where
        R: Resource,
    {
        match &self.world{
            Some(world) =>{
                world.borrow_mut().insert_resource(r);
                log::debug!("New resources insterted into scene.");
            },
            None=> (),
        }
    }

    pub fn contains_resource<R>(&mut self) -> bool
    where
        R: Resource,
    {
        match &self.world{
            Some(world) => {
                world.borrow().contains_resource::<R>()
            },
            None => false
        }
    }
}

impl Scene<Active> {

    pub fn insert_resource<R>(&mut self, r: R)
    where
        R: Resource,
    {
        match &self.world{
            Some(world) =>{
                world.borrow_mut().insert_resource(r);
                log::debug!("New resources insterted into scene.");
            },
            None=> (),
        }
    }

    pub fn contains_resource<R>(&mut self) -> bool
    where
        R: Resource,
    {
        match &self.world{
            Some(world) => {
                world.borrow().contains_resource::<R>()
            },
            None => false
        }
    }

    pub fn get_world(&mut self) -> Option<RefMut<World>>{
        match &self.world {
            Some(world) => Some(world.borrow_mut()),
            None => None,
        }
    }

    pub fn create_render_schedule(&mut self){
        let mut schedule = Schedule::default();
        
        schedule
        .add_stage("camera_move", SystemStage::parallel()
            .with_system(CameraMoveSystem)
        ).add_stage_after("camera_move", "camera_update", SystemStage::parallel()
            .with_system(CameraUpdateSystem)
        ).add_stage("wireframe_input_system", SystemStage::parallel()
            .with_system(RenderableAssemblyStateModifierSystem)
        ).add_stage("assembly_state_modifier_system", SystemStage::parallel()
            .with_system(TerrainAssemblyStateModifierSystem)
        ).add_stage_after("camera_update", "main", SystemStage::single_threaded()
            .with_system(RenderableDrawSystem)
            .with_system(DirectionalLightingSystem)
            .with_system(AmbientLightingSystem)
            .with_system(TerrainDrawSystem)
        ).add_stage("pre_ui", SystemStage::single_threaded()
            .with_system(PanelInitSystem)
        )
        .add_stage_after("pre_ui", "ui", SystemStage::single_threaded()
            .with_system(TerrainUiSystem)
            .with_system(CameraUiSystem)
            .with_system(FileSubMenuSystem)
            .with_system(ShowNewProjectWindow)
            .with_system(ShowOpenProjectWindow)
            .with_system(TransformUiSystem)
            .with_system(SceneGraphUiSystem)
            .with_system(EntityInspectionUiSystem)
        ).add_stage_after("ui", "event_processing", SystemStage::parallel()
            .with_system(SceneSerializationSystem)
            .with_system(TerrainUpdateSystem)
            .with_system(ProjectCreationSystem)
            .with_system(OpenProjectSystem)
        );
        self.state.render_schedule = Some(schedule);
    }

    pub fn create_update_schedule(&mut self){
        let schedule = Schedule::default();
        self.state.update_schedule = Some(schedule);
    }

    pub fn create_teardown_schedule(&mut self){
        let schedule = Schedule::default();
        self.state.teardown_schedule = Some(schedule);
    }

    pub fn run_render_schedule(&mut self){
        let mut schedule = self.state.render_schedule.take().unwrap();
        schedule.run(&mut *self.get_world().unwrap());
        self.state.render_schedule = Some(schedule);
    }

    pub fn run_update_schedule(&mut self){
        let mut schedule = self.state.update_schedule.take().unwrap();
        schedule.run(&mut *self.get_world().unwrap());
        self.state.update_schedule = Some(schedule);
    }

    pub fn run_teardown_schedule(&mut self){
        let mut schedule = self.state.teardown_schedule.take().unwrap();
        schedule.run(&mut *self.get_world().unwrap());
        self.state.teardown_schedule = Some(schedule);
    }

    pub fn insert_required_resources(&mut self){
        self.insert_resource(KeyInputQueue::default());
    }
}

impl <Active> Scene<Active>{
    pub fn serialize(&mut self){
    }
}

impl From<Scene<Staged>> for Scene<Active> {
    fn from(mut staged_scene: Scene<Staged>) -> Scene<Active> {
        staged_scene.run_setup_schedule();
        let mut scene = Scene{
            world: staged_scene.world,
            state: Active{
                device_loaded: false,
                update_schedule: None,
                render_schedule: None,
                teardown_schedule: None,
            }
        };
        scene.create_render_schedule();
        scene.create_update_schedule();
        scene.create_teardown_schedule();
        scene
    }
}

impl From<Scene<Active>> for Scene<Inactive> {
    fn from(_active_scene: Scene<Active>) -> Scene<Inactive> {
        Scene{
            world: None,
            state: Inactive,
        }
    }
}

impl From<Scene<Active>> for Scene<Staged> {
    fn from(mut active_scene: Scene<Active>) -> Scene<Staged> {
        active_scene.run_teardown_schedule();
        let mut scene = Scene{
            world: active_scene.world,
            state: Staged{
                setup_schedule: None,
                teardown_schedule: None,
            },
        };
        scene.create_setup_schedule();
        scene.create_teardown_schedule();
        scene
    }
}

impl From<Scene<Staged>> for Scene<Inactive> {
    fn from(_staged_scene: Scene<Staged>) -> Scene<Inactive> {
        let scene = Scene {
            world: None,
            state: Inactive
        };
        scene
    }
}

impl From<Scene<Inactive>> for Scene<Staged> {
    fn from(_inactive_scene: Scene<Inactive>) -> Scene<Staged> {
        let mut scene = Scene{
            world: Some(RefCell::new(World::new())),
            state: Staged{
                setup_schedule: None,
                teardown_schedule: None,
            },
        };
        scene.create_setup_schedule();
        scene.create_teardown_schedule();
        scene
    }
}
