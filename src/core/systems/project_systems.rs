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
    // mut save_events: ResMut<Events<SaveEvent>>,
    type_registry: Res<TypeRegistryArc>,
){
    // let mut reader = save_events.get_reader();
    // for event in reader.iter(&save_events){
    log::info!("~~~~~~~~~~~~~~~~~~Save detected!~~~~~~~~~~~~~~~~~~~~");
    let pretty = PrettyConfig::new()
        .depth_limit(2)
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let writer = File::create("./savegame2.ron").unwrap();
    let mut serializer = ron::ser::Serializer::new(&writer, Some(pretty.clone()), true).expect("Couldn't create ron serializer.");
    // let entities = world.entities();
    // let mut state = serializer.serialize_seq(Some(entities.len() as usize)).expect("Couldn't serialize entity seq");
    // let tr = type_registry.read();
    // let entities = world.entities
    let storages = world.storages();
    {
        let tables = &storages.tables;
        log::info!("Table count {:?}", tables.len());
    }
    let comps = world.components();
    let menu_id = comps.get_id(TypeId::of::<MainMenuComponent>()).unwrap();
    log::info!("Main Menu omponent id: {:?}", menu_id);
    let info = comps.get_info(menu_id);
    log::info!("Main Menu Component info: {:?}", info);
    // let sets = &storages.sparse_sets;
    let tables = &storages.tables;        
    // let s = sets.get(menu_id).unwrap();
    // log::info!("{:?}", s);
    log::info!("entity count {:?}", world.entities().len());
    log::info!("comp count {:?}", world.components().len());
    log::info!("archetype len {:?}", world.archetypes().len());
    // let mut entities = Vec::new();
    // let mut components = Vec::new();
    //     // iterate over each archetype
    //     for archetype in world.archetypes().iter() {
    //         log::info!("=== New Archetype ===");
    //         let mutentities_offset = world.entities().len();
    //         // iterate over each entity with this archetype
    //         for entity in archetype.entities() {
    //             log::info!("Ent id with this Archetype  {:?}", entity.id());
    //             log::info!("actual entity {:?}", entity);
    //             let table = tables.get(archetype.table_id()).unwrap();
    //             for row in 0..table.entities().len() {
    //                 let mut entity_comps = Vec::new();

    //                 for comp_id in archetype.table_components() {
    //                     let column = table.get_column(*comp_id).unwrap();
    //                     unsafe{
    //                         let comp_addr = column.get_data_unchecked(row as usize);
    //                         log::info!("Comp addr {:?}", comp_addr);
    //                         if comp_addr != 0x1 as *mut u8 {
    //                             // let x: f64 = comp_addr;
    //                             let comp = *comp_addr;
    //                             entity_comps.push(comp);
    //                         }
    //                     }
    //                 }
    //                 entities.push(entity);
    //                 log::info!("Table len {:?}", table.len());
    //                 if entity_comps.len() > 0 {
    //                     log::info!("Pushing components for entity {:?}", entity.id());
    //                     components.push(entity_comps); 
    //                 }
    //             }

    //         }

    //     }
    //     for i in 0..entities.len() {
    //         let entity = entities[i];
    //         let s = entity.serialize(&mut serializer).expect("Serialization failed");
    //         let s = to_string_pretty(&entity, pretty.clone()).expect("Serialization failed");
    //         log::info!("Serializing an entity {:?}", s);
    //         // let serializer = ReflectSerializer::new(&entity, &type_registry);
    //         // to_writer(&writer, &entity);
    //         for comp in &components[i]{
    //             let c = comp.serialize(&mut serializer).expect("Failed");
    //             let c = to_string_pretty(&comp, pretty.clone()).expect("serialize failed");
    //             log::info!("and its comp {:?}", c);
    //             // to_writer(&writer, &comp);
    //             // to_writer(&writer, &comp);
    //         }
    //     }
    // // }
    // // save_events.clear();
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
                log::info!("There's a component");
                for (i, entity) in archetype.entities().iter().enumerate() {
                    if let Some(component) = reflect_component.reflect_component(world, *entity)
                    {
                        log::info!("adf");
                        scene.entities[i]
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


// systems
// pub struct MenuSerializerSystem;
// impl<'a> System<'a> for MenuSerializerSystem{
//     type SystemData = (
//         Read<'a, DebugUiComponent>,
//         WriteExpect<'a, SerializerData>,
//     );

//     pub fn run(&mut self, data: Self::SystemData){
//         (let comps, mut serializer_vec) = data;
//         let pretty = PrettyConfig::new()
//             .depth_limit(2)
//             .separate_tuple_members(true)
//             .enumerate_arrays(true);
//         for comp in comps.join() {
//             serializer_vec.push(to_string_pretty(&comp, pretty.clone).expect("Couldn't serialize menu."));
//         }
//     }
// }

// // systems
// pub struct SerializerSystem;
// impl<'a> System<'a> for SerializerSystem{
//     type SystemData = (
//         Entities<'a>,
//         Read<'a, LazyUpdate>,
//     );

//     pub fn run(&mut self, data: Self::SystemData){
//         // (let entities, mut serializer_vec) = data;
//         // let pretty = PrettyConfig::new()
//             // .depth_limit(2)
//             // .separate_tuple_members(true)
//             // .enumerate_arrays(true);
//         // for comp in comps.join() {
//             // serializer_vec.push(to_string_pretty(&comp, pretty.clone).expect("Couldn't serialize menu."));
//         // }

//     }
// }