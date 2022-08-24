use crate::core::plugins::components::{
    MainMenuComponent,
    FileSubMenuComponent,
    DebugUiComponent,
};
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
        let pretty = PrettyConfig::new()
            .depth_limit(2)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let writer = File::create("./savegame2.ron").unwrap();
        let mut serializer = ron::ser::Serializer::new(&writer, Some(pretty.clone()), true).expect("Couldn't create ron serializer.");

        let storages = world.storages();
        {
            let tables = &storages.tables;
            log::info!("Table count {:?}", tables.len());
        }
        let comps = world.components();
        let menu_id = comps.get_id(TypeId::of::<MainMenuComponent>()).unwrap();
        let info = comps.get_info(menu_id);
        let tables = &storages.tables;        

        #[derive(Default)]
        pub struct DynamicScene {
            pub entities: Vec<Entity>,
        };
        
        pub struct Entity {
            pub entity: u32,
            pub components: Vec<Box<dyn Reflect>>,
        };

        let mut scene = DynamicScene::default();
        let type_registry = type_registry.read();
        for archetype in world.archetypes().iter() {
            let entities_offset = world.entities().len() as usize;
            for entity in archetype.entities() {
                scene.entities.push(Entity {
                    entity: entity.id(),
                    components: Vec::new(),
                });
            }

            for component_id in archetype.components() {
                let reflect_component = world
                    .components()
                    .get_info(component_id)
                    .and_then(|info| type_registry.get(info.type_id().unwrap()))
                    .and_then(|registration| registration.data::<ReflectComponent>());
                if let Some(reflect_component) = reflect_component {
                    for (i, entity) in archetype.entities().iter().enumerate() {
                        if let Some(component) = reflect_component.reflect(world, *entity)
                        {
                            scene.entities[entities_offset + i]
                                .components
                                .push(component.clone_value());
                        }
                    }
                }
            }
        }

        // serialize now?
        for entity in scene.entities {
            log::info!("Serilizing entity with {:?} comps", entity.components.len());
            to_writer(&writer, &entity.entity);
            for c in entity.components {
                // to_writer(&writer, &c);
                log::info!("Serializing comp");
                ReflectSerializer::new(
                    &*c,
                    &type_registry
                ).serialize(&mut serializer);
            }
        }
    }
    save_events.clear();
}