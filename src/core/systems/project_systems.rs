
use crate::core::scene::SerdeScene;

use bevy_ecs::prelude::{
    Query, 
    World,
    Res,
};
use bevy_ecs::entity::{
    Entity,
};

use bevy_ecs::prelude::EventReader;

use bevy_reflect::TypeRegistryArc;

use crate::core::events::project_events::{SaveEvent, CreateProjectEvent};

// types
pub struct SerializerData(Vec<String>);

pub fn SceneSerializationSystem(
    world: &World,
    _query: Query<Entity>,
    mut save_events: EventReader<SaveEvent>,
    type_registry: Res<TypeRegistryArc>,
){
    for _event in save_events.iter(){
        let scene = SerdeScene::from_world(&world, &type_registry);
        scene.write_to_file("./new_save.ron", &type_registry);
    }
    save_events.clear();
}

pub fn ProjectCreationSystem(
    mut new_project_events: EventReader<CreateProjectEvent>
){
    for _event in new_project_events.iter() {
        log::info!("Creating a project");
    }
    new_project_events.clear();
}