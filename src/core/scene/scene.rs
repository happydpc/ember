use specs::{World, WorldExt, Builder, Component};

use crate::core::{
    managers::manager::Manager,
};

pub struct Scene{
    pub world: World,
}

impl Scene{
    pub fn initialize(&mut self){

    }
    pub fn deinitialize(&mut self){
        
    }
}
