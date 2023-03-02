use crate::core::scene::dynamic_scene::{DynamicEntity, DynamicScene};
use bevy_reflect::TypeRegistryArc;
use bevy_ecs::{prelude::Entity, reflect::ReflectComponent, world::World};
use bevy_utils::default;
use std::collections::BTreeMap;

use super::TypeRegistryResource;

/// A [`DynamicScene`] builder, used to build a scene from a [`World`] by extracting some entities.
///
/// # Entity Order
///
/// Extracted entities will always be stored in ascending order based on their [id](Entity::index).
/// This means that inserting `Entity(1v0)` then `Entity(0v0)` will always result in the entities
/// being ordered as `[Entity(0v0), Entity(1v0)]`.
///
/// # Example
/// ```
/// # use bevy_scene::DynamicSceneBuilder;
/// # use bevy_app::TypeRegistryArc;
/// # use bevy_ecs::{
/// #     component::Component, prelude::Entity, query::With, reflect::ReflectComponent, world::World,
/// # };
/// # use bevy_reflect::Reflect;
/// # #[derive(Component, Reflect, Default, Eq, PartialEq, Debug)]
/// # #[reflect(Component)]
/// # struct ComponentA;
/// # let mut world = World::default();
/// # world.init_resource::<TypeRegistryArc>();
/// # let entity = world.spawn(ComponentA).id();
/// let mut builder = DynamicSceneBuilder::from_world(&world);
/// builder.extract_entity(entity);
/// let dynamic_scene = builder.build();
/// ```
pub struct DynamicSceneBuilder<'w> {
    entities: BTreeMap<u32, DynamicEntity>,
    type_registry: TypeRegistryArc,
    world: &'w World,
}

impl<'w> DynamicSceneBuilder<'w> {
    /// Prepare a builder that will extract entities and their component from the given [`World`].
    /// All components registered in that world's [`TypeRegistryArc`] resource will be extracted.
    pub fn from_world(world: &'w World) -> Self {
        Self {
            entities: default(),
            type_registry: world.resource::<TypeRegistryResource>().0.clone(),
            world,
        }
    }

    /// Prepare a builder that will extract entities and their component from the given [`World`].
    /// Only components registered in the given [`TypeRegistryArc`] will be extracted.
    pub fn from_world_with_type_registry(world: &'w World, type_registry: TypeRegistryArc) -> Self {
        Self {
            entities: default(),
            type_registry,
            world,
        }
    }

    /// Consume the builder, producing a [`DynamicScene`].
    pub fn build(self) -> DynamicScene {
        DynamicScene {
            entities: self.entities.into_values().collect(),
        }
    }

    /// Extract one entity from the builder's [`World`].
    ///
    /// Re-extracting an entity that was already extracted will have no effect.
    pub fn extract_entity(&mut self, entity: Entity) -> &mut Self {
        self.extract_entities(std::iter::once(entity))
    }

    /// Extract entities from the builder's [`World`].
    ///
    /// Re-extracting an entity that was already extracted will have no effect.
    ///
    /// Extracting entities can be used to extract entities from a query:
    /// ```
    /// # use bevy_scene::DynamicSceneBuilder;
    /// # use bevy_app::TypeRegistryArc;
    /// # use bevy_ecs::{
    /// #     component::Component, prelude::Entity, query::With, reflect::ReflectComponent, world::World,
    /// # };
    /// # use bevy_reflect::Reflect;
    /// #[derive(Component, Default, Reflect)]
    /// #[reflect(Component)]
    /// struct MyComponent;
    ///
    /// # let mut world = World::default();
    /// # world.init_resource::<TypeRegistryArc>();
    /// # let _entity = world.spawn(MyComponent).id();
    /// let mut query = world.query_filtered::<Entity, With<MyComponent>>();
    ///
    /// let mut builder = DynamicSceneBuilder::from_world(&world);
    /// builder.extract_entities(query.iter(&world));
    /// let scene = builder.build();
    /// ```
    pub fn extract_entities(&mut self, entities: impl Iterator<Item = Entity>) -> &mut Self {
        let type_registry = self.type_registry.read();

        for entity in entities {
            let index = entity.index();

            if self.entities.contains_key(&index) {
                continue;
            }

            let mut entry = DynamicEntity {
                entity: index,
                components: Vec::new(),
            };

            for component_id in self.world.entity(entity).archetype().components() {
                let reflect_component = self
                    .world
                    .components()
                    .get_info(component_id)
                    .and_then(|info| type_registry.get(info.type_id().unwrap()))
                    .and_then(|registration| registration.data::<ReflectComponent>());

                if let Some(reflect_component) = reflect_component {
                    if let Some(component) = reflect_component.reflect(self.world, entity) {
                        entry.components.push(component.clone_value());
                    }
                }
            }
            self.entities.insert(index, entry);
        }

        drop(type_registry);
        self
    }
}
