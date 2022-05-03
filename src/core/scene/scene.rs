use specs::{
    prelude::Resource,
    WorldExt,
    Component,
    World,
    saveload::{
        SerializeComponents,
        SimpleMarker,
    },
};
use ron;
use ron::ser::PrettyConfig;

use std::{
    cell::{
        RefCell,
        RefMut,
    },
    fs::File,
    convert::Infallible,
};

use super::system_dispatch::{MultiThreadedDispatcher, SystemDispatch};
use crate::core::input::input_manager::KeyInputQueue;
use crate::core::systems::{
    ui_systems::{
        DebugUiSystem,
    },
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
    },
    CameraInitSystem,
    TerrainInitSystem,
    TerrainDrawSystem,
    TerrainAssemblyStateModifierSystem,
    TerrainUiSystem,
    GeometryInitializerSystem,
};
use crate::core::plugins::components::{
    InputComponent,
    CameraComponent,
    TransformComponent,
    TransformUiComponent,
    DebugUiComponent,
    RenderableComponent,
    DirectionalLightComponent,
    AmbientLightingComponent,
    TerrainComponent,
    TerrainUiComponent,
    SerializerFlag,
    GeometryComponent,
};

use crate::construct_dispatcher;
use crate::serialize_individually;

pub struct Scene<S>{
    pub world: Option<RefCell<World>>,
    pub state: S,
}

pub struct Active{
    pub device_loaded: bool,
    pub update_dispatch: Option<Box<dyn SystemDispatch + 'static>>,
    pub render_dispatch: Option<Box<dyn SystemDispatch + 'static>>,
}
pub struct Inactive;
pub struct Staged{
    pub setup_dispatch: Option<Box<dyn SystemDispatch + 'static>>,
    pub teardown_dispatch: Option<Box<dyn SystemDispatch + 'static>>,
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
    fn create_setup_dispatch(&mut self){
        construct_dispatcher!(
            (CameraInitSystem, "camear_init", &[]),
            (TerrainInitSystem, "terrain_init", &[]),
            (GeometryInitializerSystem, "geom_init", &[]),
            (RenderableInitializerSystem, "render_init", &["geom_init"])
        );
        self.state.setup_dispatch = Some(new_dispatch());
    }
    
    fn create_teardown_dispatch(&mut self){
        construct_dispatcher!(
        );
        self.state.teardown_dispatch = Some(new_dispatch());
    }

    pub fn run_setup_dispatch(&mut self){
        log::info!("Running setup dispatch...");
        let mut dispatch = self.state.setup_dispatch.take().expect("No setup dispatch");
        dispatch.run_now(&mut *self.get_world().unwrap());
        self.state.setup_dispatch = Some(dispatch);
    }

    pub fn run_teardown_dispatch(&mut self){
        log::info!("Running teardown dispatch...");
        let mut dispatch = self.state.teardown_dispatch.take().expect("no teardown dispatch");
        dispatch.run_now(&mut *self.get_world().unwrap());
        self.state.teardown_dispatch = Some(dispatch);
    }

    pub fn get_world(&mut self) -> Option<RefMut<World>>{
        match &self.world {
            Some(world) => Some(world.borrow_mut()),
            None => None,
        }
    }

    // pass through for world register function
    pub fn register<T: Component>(&mut self)
    where
        T::Storage: Default
    {
        match &self.world {
            Some(world) => {
                world.borrow_mut().register::<T>();
                log::info!("New component type registered with scene.");
            },
            None => (),
        }
    }

    pub fn insert_resource<R>(&mut self, r: R)
    where
        R: Resource,
    {
        match &self.world{
            Some(world) =>{
                world.borrow_mut().insert(r);
                log::debug!("New resources insterted into scene.");
            },
            None=> (),
        }
    }
}

impl Scene<Active> {

    // pass through for world register function
    pub fn register<T: Component>(&mut self)
    where
        T::Storage: Default
    {
        match &self.world {
            Some(world) => {
                world.borrow_mut().register::<T>();
                log::info!("New component type registered with scene.");
            },
            None => (),
        }
    }

    pub fn insert_resource<R>(&mut self, r: R)
    where
        R: Resource,
    {
        match &self.world{
            Some(world) =>{
                world.borrow_mut().insert(r);
                log::debug!("New resources insterted into scene.");
            },
            None=> (),
        }
    }

    pub fn get_world(&mut self) -> Option<RefMut<World>>{
        match &self.world {
            Some(world) => Some(world.borrow_mut()),
            None => None,
        }
    }

    pub fn create_render_dispatch(&mut self){
        construct_dispatcher!(
            // (TransformUiSystem, "transform_ui", &[]),
            (TerrainInitSystem, "terrain_init", &[]),
            (TerrainUiSystem, "terrain_ui", &[]),
            (DebugUiSystem, "debug_ui", &[]),
            (CameraMoveSystem, "camera_move", &[]),
            (CameraUpdateSystem, "camera_update", &["camera_move"]),
            (RenderableDrawSystem, "renderable_draw", &["camera_update"]),
            (DirectionalLightingSystem, "directional_lighting", &[]),
            (AmbientLightingSystem, "ambient_lighting", &[]),
            (RenderableAssemblyStateModifierSystem, "wireframe_system", &[]),
            (CameraUiSystem, "camera_ui", &[]),
            (TerrainAssemblyStateModifierSystem, "terrain_wireframe", &["wireframe_system"]),
            (TerrainDrawSystem, "terrain_draw", &["camera_update"])
        );
        self.state.render_dispatch = Some(new_dispatch());
    }

    pub fn create_update_dispatch(&mut self){
        construct_dispatcher!(
        );
        self.state.update_dispatch = Some(new_dispatch());
    }

    pub fn run_render_dispatch(&mut self){
        let mut dispatch = self.state.render_dispatch.take().unwrap();
        dispatch.run_now(&mut *self.get_world().unwrap());
        self.state.render_dispatch = Some(dispatch);
    }

    pub fn run_update_dispatch(&mut self){
        let mut dispatch = self.state.update_dispatch.take().unwrap();
        dispatch.run_now(&mut *self.get_world().unwrap());
        self.state.update_dispatch = Some(dispatch);
    }

    pub fn insert_required_resources(&mut self){
        self.insert_resource(KeyInputQueue::new());
    }
}

impl <Active> Scene<Active>{
    pub fn serialize(&mut self){
        let mut worldref = self.world.take().unwrap();
        let mut world = worldref.get_mut();
        
        // Actually serialize
        {
            let data = ( world.entities(), world.read_storage::<SimpleMarker<SerializerFlag>>() );

            let pretty = PrettyConfig::new()
                .depth_limit(2)
                .separate_tuple_members(true)
                .enumerate_arrays(true);

            let writer = File::create("./savegame.ron").unwrap();
            let mut serializer = ron::ser::Serializer::new(writer, Some(pretty), true).expect("Couldn't create ron serializer.");
            serialize_individually!(
                world,
                serializer,
                data,
                InputComponent,
                CameraComponent,
                TransformComponent,
                TransformUiComponent,
                DebugUiComponent,
                RenderableComponent,
                DirectionalLightComponent,
                AmbientLightingComponent,
                TerrainComponent,
                TerrainUiComponent,
                GeometryComponent
            );
        }

        self.world = Some(worldref);

    }
}

impl From<Scene<Staged>> for Scene<Active> {
    fn from(mut staged_scene: Scene<Staged>) -> Scene<Active> {
        staged_scene.run_setup_dispatch();
        let mut scene = Scene{
            world: staged_scene.world,
            state: Active{
                device_loaded: false,
                update_dispatch: None,
                render_dispatch: None,
            }
        };
        scene.create_render_dispatch();
        scene.create_update_dispatch();
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
                setup_dispatch: None,
                teardown_dispatch: None,
            },
        };
        scene.create_setup_dispatch();
        scene.create_teardown_dispatch();
        scene
    }
}
