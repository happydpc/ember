use crate::core::plugins::components::{
    MainMenuComponent,
    FileSubMenuComponent,
    DebugUiComponent,
};
use crate::core::scene::SerdeScene;
use ron::ser::{to_string_pretty, PrettyConfig};
use bevy_ecs::prelude::{
    ResMut,
    Query, 
    World,
    Res,
    ReflectComponent,
};
use bevy_ecs::entity::{
    Entities,
    Entity,
};
use bevy_ecs::event::{
    // Reader,
    Events,
};
use bevy_ecs::prelude::EventReader;
use bevy_reflect::Reflect;
use bevy_reflect::TypeRegistryArc;
use bevy_reflect::serde::ReflectSerializer;
use crate::core::events::project_events::SaveEvent;
use std::fs::File;
use serde::ser::Serializer;
use serde::ser::Serialize;
use ron::ser::{
    // Serialize,
    to_writer,
    Serializer as RonSerializer,
};

use std::any::TypeId;


// types
pub struct SerializerData(Vec<String>);

pub fn SceneSerializationSystem(
    world: &World,
    query: Query<Entity>,
    mut save_events: EventReader<SaveEvent>,
    type_registry: Res<TypeRegistryArc>,
){
    for event in save_events.iter(){
        // let pretty = PrettyConfig::new()
        //     .depth_limit(2)
        //     .separate_tuple_members(true)
        //     .enumerate_arrays(true);
        // let writer = File::create("./savegame20.ron").unwrap();
        // let mut serializer = ron::ser::Serializer::new(&writer, Some(pretty.clone()), true).expect("Couldn't create ron serializer.");

        let mut scene = SerdeScene::from_world(&world, &type_registry);
        scene.write_to_file("./new_save.ron", &type_registry);
        // serialize now?
        // for entity in scene.entities {
        //     log::info!("Serilizing entity with {:?} comps", entity.components.len());
        //     to_writer(&writer, &entity.entity);
        //     for c in entity.components {
        //         // to_writer(&writer, &c);
        //         log::info!("Serializing comp");
        //         ReflectSerializer::new(
        //             &*c,
        //             &type_registry.read()
        //         ).serialize(&mut serializer);
        //     }
        // }
    }
    save_events.clear();
}