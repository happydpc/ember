use bevy_ecs::{
    prelude::Schedule,
    prelude::SystemStage,
    world::World,
    schedule::Stage,
    system::Resource,
};



use std::{
    cell::{
        RefCell,
        RefMut,
    },
};
use std::borrow::BorrowMut;

use crate::core::managers::input_manager::KeyInputQueue;
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
    },
    CameraInitSystem,
    TerrainInitSystem,
    TerrainDrawSystem,
    TerrainAssemblyStateModifierSystem,
    TerrainUiSystem,
    GeometryInitializerSystem,
    SceneSerializationSystem,
    TerrainUpdateSystem,
    MainMenuInitSystem,
    ShowNewProjectWindow,
    ShowOpenProjectWindow,
};




pub struct Scene<S>{
    pub world: Option<RefCell<World>>,
    pub state: S,
}

pub struct Active{
    pub device_loaded: bool,
    pub update_schedule: Option<Schedule>,
    pub render_schedule: Option<Schedule>,
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
    fn create_setup_schedule(&mut self){
        let mut schedule = Schedule::default();
        log::info!("Creating setup schedule.");
        schedule
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
    // pub fn get_resource<R>(&mut self, r: R) -> Option<&R>
    // where
    //     R: Resource,
    // {
    //     match &self.world{
    //         Some(world) => {
    //             world.borrow().get_resource::<R>()
    //         },
    //         None => None,
    //     }
    // }

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
        ).add_stage_after("camera_update", "main", SystemStage::parallel()
            .with_system(RenderableDrawSystem)
            .with_system(DirectionalLightingSystem)
            .with_system(AmbientLightingSystem)
            .with_system(TerrainDrawSystem)
        ).add_stage("pre_ui", SystemStage::single_threaded()
            .with_system(MainMenuInitSystem)
        )
        .add_stage_after("pre_ui", "ui", SystemStage::single_threaded()
            .with_system(TerrainUiSystem)
            .with_system(CameraUiSystem)
            .with_system(FileSubMenuSystem)
            .with_system(ShowNewProjectWindow)
            .with_system(ShowOpenProjectWindow)
            .with_system(TransformUiSystem)
        ).add_stage("event_processing", SystemStage::parallel()
            .with_system(SceneSerializationSystem)
            .with_system(TerrainUpdateSystem)
        );
        self.state.render_schedule = Some(schedule);
    }

    pub fn create_update_schedule(&mut self){
        let schedule = Schedule::default();
        self.state.update_schedule = Some(schedule);
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

    pub fn insert_required_resources(&mut self){
        self.insert_resource(KeyInputQueue::new());
    }
}

impl <Active> Scene<Active>{
    pub fn serialize(&mut self){
        log::info!("Serializing Scene");
        // let mut worldref = self.world.take().unwrap();
        // let world = worldref.get_mut();
        
        // // Actually serialize
        // {
        //     let data = ( world.entities(), world.read_storage::<SimpleMarker<SerializerFlag>>() );

        //     let pretty = PrettyConfig::new()
        //         .depth_limit(2)
        //         .separate_tuple_members(true)
        //         .enumerate_arrays(true);

        //     let writer = File::create("./savegame.ron").unwrap();
        //     let mut serializer = ron::ser::Serializer::new(writer, Some(pretty), true).expect("Couldn't create ron serializer.");
        //     serialize_individually!(
        //         world,
        //         serializer,
        //         data,
        //         InputComponent,
        //         CameraComponent,
        //         TransformComponent,
        //         TransformUiComponent,
        //         RenderableComponent,
        //         DirectionalLightComponent,
        //         AmbientLightingComponent,
        //         TerrainComponent,
        //         TerrainUiComponent,
        //         GeometryComponent
        //     );
        // }

        // self.world = Some(worldref);

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
            }
        };
        scene.create_render_schedule();
        scene.create_update_schedule();
        scene.insert_required_resources();
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
