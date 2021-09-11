use specs::{World, WorldExt, Builder, Component};

use crate::core::{
    managers::manager::Manager,
};

pub struct Scene<S>{
    pub world: Option<World>,
    pub state: S,
}

pub struct Uninitialized;
pub struct Initialized;

impl From<Scene<Uninitialized>> for Scene<Initialized> {
    fn from(val: Scene<Uninitialized>) -> Scene<Initialized> {
        Scene{
            world: Some(World::new()),
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
