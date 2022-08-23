
use vulkano::{
    device::{
        Device
    }
};
use bevy_ecs::component::Component;
use bevy_reflect::{Reflect, FromReflect};
use bevy_ecs::prelude::ReflectComponent;

use std::sync::{Arc};
use serde::{
    Serialize,
    Deserialize,
};


#[derive(Component, Default, Clone, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]
pub struct RenderableComponent{
    pub initialized: bool,
}

impl RenderableComponent{

    pub fn create() -> Self {
        RenderableComponent{
            initialized: false,
        }
    }

    pub fn initialize(&mut self, _device: Arc<Device>){
        log::debug!("Initializing renderable component...");
        self.initialized = true;
    }

}
