use crate::core::scene::Scene;
use crate::core::scene::Active;

use std::fs::File;

use ron::ser::PrettyConfig;
use serde::ser::Serialize;
use serde::ser::Serializer;
use ron::ser::{
    to_writer,
    Serializer as RonSerializer,
};
use bevy_reflect::{
    Reflect,
    FromReflect,
    TypeRegistryArc,
    serde::ReflectSerializer,
};
use bevy_ecs::{
    prelude::{
        World,
        ReflectComponent,
    }
};

#[derive(Default)]
pub struct SerdeScene {
    pub entities: Vec<Entity>,
}

pub struct Entity {
    pub entity: u32,
    pub components: Vec<Box<dyn Reflect>>,
}

impl SerdeScene {
    pub fn from_world(world: &World, type_registry: &TypeRegistryArc) -> Self {
        let mut scene = SerdeScene::default();
        let type_registry = type_registry.read();
        for archetype in world.archetypes().iter() {
            let entities_offset = world.entities().len() as usize;
            log::info!("Entity count at serialization: {}", entities_offset);
            for entity in archetype.entities() {
                log::info!("A new entity with an archetype I suppose.");
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
                        log::info!("Entity?");
                        if let Some(component) = reflect_component.reflect(world, *entity)
                        {
                            log::info!("Found component");
                            scene.entities[i]
                                .components
                                .push(component.clone_value());
                        }
                    }
                }
            }
        }
        scene
    }

    pub fn write_to_file(&self, file_name: &str, type_registry: &TypeRegistryArc){
        let pretty = PrettyConfig::new()
            .depth_limit(2)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let writer = File::create(file_name).unwrap();
        let mut serializer = ron::ser::Serializer::new(&writer, Some(pretty.clone()), true).expect("Couldn't create ron serializer.");

        for entity in &self.entities {
            log::info!("Serilizing entity with {:?} comps", entity.components.len());
            to_writer(&writer, &entity.entity);
            for c in &entity.components {
                ReflectSerializer::new(
                    &**c,
                    &type_registry.read()
                ).serialize(&mut serializer);
            }
        }
    }
}