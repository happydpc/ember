use specs::{World, WorldExt, Builder, Component};

use std::{
    cell::{
        RefCell,
        RefMut,
    },
};

use crate::core::{
    managers::manager::Manager,
};

pub struct Scene<S>{
    pub world: Option<RefCell<World>>,
    pub state: S,
}

pub struct Uninitialized;
pub struct Initialized;

impl Scene<Uninitialized> {
    pub fn new() -> Self {
        Scene{
            world: None,
            state: Uninitialized,
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
}

impl From<Scene<Uninitialized>> for Scene<Initialized> {
    fn from(val: Scene<Uninitialized>) -> Scene<Initialized> {
        Scene{
            world: Some(RefCell::new(World::new())),
            state: Initialized,
        }
    }
}

impl From<Scene<Initialized>> for Scene<Uninitialized> {
    fn from(val: Scene<Initialized>) -> Scene<Uninitialized> {
        Scene{
            world: None,
            state: Uninitialized,
        }
    }
}
