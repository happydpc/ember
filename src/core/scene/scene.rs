use specs::{
    prelude::Resource,
    WorldExt,
    Component,
    World,
    Dispatcher,
    System
};

use std::{
    cell::{
        RefCell,
        RefMut,
    },
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
    update_systems::{
    },
    render_systems::{
        RenderableInitializerSystem,
        RenderableDrawSystem,
        CameraUpdateSystem,
    }
};
use crate::construct_dispatcher;

pub struct Scene<S>{
    pub world: Option<RefCell<World>>,
    pub state: S,
    pub update_dispatch: Option<Box<dyn SystemDispatch + 'static>>,
    pub render_dispatch: Option<Box<dyn SystemDispatch + 'static>>,
}

pub struct Uninitialized;
pub struct Initialized{
    pub device_loaded: bool,
}

impl Scene<Uninitialized> {
    pub fn new() -> Self {
        Scene{
            // world: None,
            world: Some(RefCell::new(World::new())),
            state: Uninitialized,
            update_dispatch: None,
            render_dispatch: None,
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

    // TODO : put this in a trait
    pub fn get_world(&mut self) -> Option<RefMut<World>>{
        match &self.world {
            Some(world) => Some(world.borrow_mut()),
            None => None,
        }
    }
}

impl Scene<Initialized> {

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
            (RenderableInitializerSystem, "render_init", &[]),
            (DebugUiSystem, "debug_ui", &[]),
            (CameraMoveSystem, "camera_move", &[]),
            (CameraUpdateSystem, "camera_update", &["camera_move"]),
            (RenderableDrawSystem, "renderable_draw", &["camera_update","render_init"])
        );
        self.render_dispatch = Some(new_dispatch());
    }

    pub fn create_update_dispatch(&mut self){
        construct_dispatcher!(
        );
        self.update_dispatch = Some(new_dispatch());
    }

    pub fn run_render_dispatch(&mut self){
        let mut dispatch = self.render_dispatch.take().unwrap();
        dispatch.run_now(&mut *self.get_world().unwrap());
        self.render_dispatch = Some(dispatch);
    }

    pub fn run_update_dispatch(&mut self){
        let mut dispatch = self.update_dispatch.take().unwrap();
        dispatch.run_now(&mut *self.get_world().unwrap());
        self.update_dispatch = Some(dispatch);
    }

    pub fn insert_required_resources(&mut self){
        self.insert_resource(KeyInputQueue::new());
    }
}

impl From<Scene<Uninitialized>> for Scene<Initialized> {
    fn from(val: Scene<Uninitialized>) -> Scene<Initialized> {
        let mut scene = Scene{
            // world: Some(RefCell::new(World::new())),
            world: val.world,
            state: Initialized{
                device_loaded: false,
            },
            update_dispatch: None,
            render_dispatch: None,
        };
        scene.create_render_dispatch();
        scene.create_update_dispatch();
        scene.insert_required_resources();
        scene
    }
}

impl From<Scene<Initialized>> for Scene<Uninitialized> {
    fn from(val: Scene<Initialized>) -> Scene<Uninitialized> {
        Scene{
            world: None,
            state: Uninitialized,
            update_dispatch: None,
            render_dispatch: None,
        }
    }
}
