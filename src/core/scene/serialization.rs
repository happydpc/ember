
use bevy_reflect::{
    serde::{ReflectDeserializer, ReflectSerializer},
    Reflect, TypeRegistry, TypeRegistryArc,
};
use serde::{
    de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Serialize,
};
use std::fs::File;
use std::io::Write;
use ron::ser::PrettyConfig;

use ron::ser::{
    to_writer,
};
use bevy_ecs::{
    prelude::{
        World,
        ReflectComponent,
    },
    entity::EntityMap,
    reflect::ReflectMapEntities,
};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum SceneSpawnError {
    #[error("scene contains the unregistered component `{type_name}`. consider adding `#[reflect(Component)]` to your type")]
    UnregisteredComponent { type_name: String },
    #[error("scene contains the unregistered type `{type_name}`. consider registering the type using `app.register_type::<T>()`")]
    UnregisteredType { type_name: String },
    // #[error("scene does not exist")]
    // NonExistentScene { handle: Handle<DynamicScene> },
    // #[error("scene does not exist")]
    // NonExistentRealScene { handle: Handle<Scene> },
}

#[derive(Default)]
pub struct SerdeScene {
    pub entities: Vec<SerdeEntity>,
}

pub struct SerdeEntity {
    pub entity: u32,
    pub components: Vec<Box<dyn Reflect>>,
}

impl SerdeScene {
    pub fn from_world(world: &World, type_registry: &TypeRegistryArc) -> Self {
        let mut scene = SerdeScene::default();
        let type_registry = type_registry.read();
        for archetype in world.archetypes().iter() {
            let entities_offset = scene.entities.len() as usize;
            log::info!("Entity count at serialization: {}", entities_offset);
            for entity in archetype.entities() {
                log::info!("A new entity with an archetype I suppose.");
                scene.entities.push(SerdeEntity {
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
                            
                            scene.entities[entities_offset + i]
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
        let ronald = self.serialize_ron(type_registry).expect("error in serialization");
        let mut buffer = File::create(file_name).expect("Couldn't create file");
        buffer.write(ronald.as_bytes()).expect("Couldn't write to file.");
        
    }

    /// Write the dynamic entities and their corresponding components to the given world.
    ///
    /// This method will return a `SceneSpawnError` if either a type is not registered
    /// or doesn't reflect the `Component` trait.
    pub fn write_to_world(
        &self,
        world: &mut World,
        entity_map: &mut EntityMap,
    ) -> Result<(), SceneSpawnError> {
        let registry = world.resource::<TypeRegistryArc>().clone();
        let type_registry = registry.read();

        for scene_entity in &self.entities {
            // Fetch the entity with the given entity id from the `entity_map`
            // or spawn a new entity with a transiently unique id if there is
            // no corresponding entry.
            let entity = *entity_map
                .entry(bevy_ecs::entity::Entity::from_raw(scene_entity.entity))
                .or_insert_with(|| world.spawn().id());

            // Apply/ add each component to the given entity.
            for component in &scene_entity.components {
                let registration = type_registry
                    .get_with_name(component.type_name())
                    .ok_or_else(|| SceneSpawnError::UnregisteredType {
                        type_name: component.type_name().to_string(),
                    })?;
                let reflect_component =
                    registration.data::<ReflectComponent>().ok_or_else(|| {
                        SceneSpawnError::UnregisteredComponent {
                            type_name: component.type_name().to_string(),
                        }
                    })?;

                // If the entity already has the given component attached,
                // just apply the (possibly) new value, otherwise add the
                // component to the entity.
                reflect_component.apply_or_insert(world, entity, &**component);
            }
        }

        for registration in type_registry.iter() {
            if let Some(map_entities_reflect) = registration.data::<ReflectMapEntities>() {
                map_entities_reflect
                    .map_entities(world, entity_map)
                    .unwrap();
            }
        }

        Ok(())
    }

    // TODO: move to AssetSaver when it is implemented
    /// Serialize this dynamic scene into rust object notation (ron).
    pub fn serialize_ron(&self, registry: &TypeRegistryArc) -> Result<String, ron::Error> {
        serialize_ron(SceneSerializer::new(self, registry))
    }
}

/// Serialize a given Rust data structure into rust object notation (ron).
pub fn serialize_ron<S>(serialize: S) -> Result<String, ron::Error>
where
    S: Serialize,
{
    let pretty_config = ron::ser::PrettyConfig::default()
        .decimal_floats(true)
        .indentor("  ".to_string())
        .new_line("\n".to_string());
    ron::ser::to_string_pretty(&serialize, pretty_config)
}



pub struct SceneSerializer<'a> {
    pub scene: &'a SerdeScene,
    pub registry: &'a TypeRegistryArc,
}

impl<'a> SceneSerializer<'a> {
    pub fn new(scene: &'a SerdeScene, registry: &'a TypeRegistryArc) -> Self {
        SceneSerializer { scene, registry }
    }
}

impl<'a> Serialize for SceneSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.scene.entities.len()))?;
        for entity in &self.scene.entities {
            state.serialize_element(&EntitySerializer {
                entity,
                registry: self.registry,
            })?;
        }
        state.end()
    }
}

pub struct EntitySerializer<'a> {
    pub entity: &'a SerdeEntity,
    pub registry: &'a TypeRegistryArc,
}

impl<'a> Serialize for EntitySerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(ENTITY_STRUCT, 2)?;
        state.serialize_field(ENTITY_FIELD_ENTITY, &self.entity.entity)?;
        state.serialize_field(
            ENTITY_FIELD_COMPONENTS,
            &ComponentsSerializer {
                components: &self.entity.components,
                registry: self.registry,
            },
        )?;
        state.end()
    }
}

pub struct ComponentsSerializer<'a> {
    pub components: &'a [Box<dyn Reflect>],
    pub registry: &'a TypeRegistryArc,
}

impl<'a> Serialize for ComponentsSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.components.len()))?;
        for component in self.components {
            state.serialize_element(&ReflectSerializer::new(
                &**component,
                &*self.registry.read(),
            ))?;
        }
        state.end()
    }
}

pub struct SceneDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneDeserializer<'a> {
    type Value = SerdeScene;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(SerdeScene {
            entities: deserializer.deserialize_seq(SceneEntitySeqVisitor {
                type_registry: self.type_registry,
            })?,
        })
    }
}

struct SceneEntitySeqVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for SceneEntitySeqVisitor<'a> {
    type Value = Vec<SerdeEntity>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("list of entities")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut entities = Vec::new();
        while let Some(entity) = seq.next_element_seed(SceneEntityDeserializer {
            type_registry: self.type_registry,
        })? {
            entities.push(entity);
        }

        Ok(entities)
    }
}

pub struct SceneEntityDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneEntityDeserializer<'a> {
    type Value = SerdeEntity;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            ENTITY_STRUCT,
            &[ENTITY_FIELD_ENTITY, ENTITY_FIELD_COMPONENTS],
            SceneEntityVisitor {
                registry: self.type_registry,
            },
        )
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum EntityField {
    Entity,
    Components,
}

pub const ENTITY_STRUCT: &str = "Entity";
pub const ENTITY_FIELD_ENTITY: &str = "entity";
pub const ENTITY_FIELD_COMPONENTS: &str = "components";

struct SceneEntityVisitor<'a> {
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for SceneEntityVisitor<'a> {
    type Value = SerdeEntity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("entities")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut id = None;
        let mut components = None;
        while let Some(key) = map.next_key()? {
            match key {
                EntityField::Entity => {
                    if id.is_some() {
                        return Err(Error::duplicate_field(ENTITY_FIELD_ENTITY));
                    }
                    id = Some(map.next_value::<u32>()?);
                }
                EntityField::Components => {
                    if components.is_some() {
                        return Err(Error::duplicate_field(ENTITY_FIELD_COMPONENTS));
                    }

                    components = Some(map.next_value_seed(ComponentVecDeserializer {
                        registry: self.registry,
                    })?);
                }
            }
        }

        let entity = id
            .as_ref()
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_ENTITY))?;

        let components = components
            .take()
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_COMPONENTS))?;
        Ok(SerdeEntity {
            entity: *entity,
            components,
        })
    }
}

pub struct ComponentVecDeserializer<'a> {
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for ComponentVecDeserializer<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ComponentSeqVisitor {
            registry: self.registry,
        })
    }
}

struct ComponentSeqVisitor<'a> {
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for ComponentSeqVisitor<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("list of components")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut dynamic_properties = Vec::new();
        while let Some(entity) = seq.next_element_seed(ReflectDeserializer::new(self.registry))? {
            dynamic_properties.push(entity);
        }

        Ok(dynamic_properties)
    }
}