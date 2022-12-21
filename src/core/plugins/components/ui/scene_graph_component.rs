use bevy_ecs::prelude::{ReflectComponent, Component};
use bevy_reflect::prelude::{Reflect};
use serde::{Serialize, Deserialize};

#[derive(Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component)]
pub struct SceneGraphComponent{
}