use bevy::{
    core::Name,
    ecs::{
        entity::Entity,
        query::{Added, Without},
        reflect::{AppTypeRegistry, ReflectComponent},
        world::World,
    },
    gltf::{GltfExtras, GltfMaterialExtras, GltfMeshExtras, GltfSceneExtras},
    hierarchy::Parent,
    log::{debug, warn},
    reflect::{PartialReflect, TypeRegistration},
    utils::HashMap,
};

use crate::{ronstring_to_reflect_component, GltfProcessed};

// , mut entity_components: HashMap<Entity, Vec<(Box<dyn Reflect>, TypeRegistration)>>
fn find_entity_components(
    entity: Entity,
    name: Option<&Name>,
    parent: Option<&Parent>,
    reflect_components: Vec<(Box<dyn PartialReflect>, TypeRegistration)>,
    entity_components: &HashMap<Entity, Vec<(Box<dyn PartialReflect>, TypeRegistration)>>,
) -> (Entity, Vec<(Box<dyn PartialReflect>, TypeRegistration)>) {
    // we assign the components specified /xxx_components objects to their parent node
    let mut target_entity = entity;
    // if the node contains "components" or ends with "_pa" (ie add to parent), the components will not be added to the entity itself but to its parent
    // this is mostly used for Blender collections
    if parent.is_some() {
        if let Some(name) = name {
            if name.as_str().contains("components") || name.as_str().ends_with("_pa") {
                debug!("adding components to parent");
                target_entity = parent.expect("the target entity had a parent ").get();
            }
        }
    }
    debug!("adding to {:?}", target_entity);

    // if there where already components set to be added to this entity (for example when entity_data was refering to a parent), update the vec of entity_components accordingly
    // this allows for example blender collection to provide basic ecs data & the instances to override/ define their own values
    if entity_components.contains_key(&target_entity) {
        let mut updated_components: Vec<(Box<dyn PartialReflect>, TypeRegistration)> = Vec::new();
        let current_components = &entity_components[&target_entity];
        // first inject the current components
        for (component, type_registration) in current_components {
            //updated_components.push((component.clone().downcast().unwrap(), type_registration.clone()));
            updated_components.push((component.clone_value(), type_registration.clone()));
        }
        // then inject the new components: this also enables overwrite components set in the collection
        for (component, type_registration) in reflect_components {
            updated_components.push((component.clone_value(), type_registration));
        }
        return (target_entity, updated_components);
    }
    (target_entity, reflect_components)
}

/// main function: injects components into each entity in gltf files that have `gltf_extras`, using reflection
pub fn add_components_from_gltf_extras(world: &mut World) {
    let mut extras = world.query_filtered::<(Entity, Option<&Name>, &GltfExtras, Option<&Parent>), (Added<GltfExtras>, Without<GltfProcessed>)>();
    let mut scene_extras = world.query_filtered::<(Entity, Option<&Name>, &GltfSceneExtras, Option<&Parent>), (Added<GltfSceneExtras>, Without<GltfProcessed>)>();
    let mut mesh_extras = world.query_filtered::<(Entity, Option<&Name>, &GltfMeshExtras, Option<&Parent>), (Added<GltfMeshExtras>, Without<GltfProcessed>)>();
    let mut material_extras = world.query_filtered::<(Entity, Option<&Name>, &GltfMaterialExtras, Option<&Parent>), (Added<GltfMaterialExtras>, Without<GltfProcessed>)>();

    let mut entity_components: HashMap<Entity, Vec<(Box<dyn PartialReflect>, TypeRegistration)>> =
        HashMap::new();

    // let gltf_components_config = world.resource::<GltfComponentsConfig>();

    for (entity, name, extra, parent) in extras.iter(world) {
        debug!(
            "Gltf Extra: Name: {:?}, entity {:?}, parent: {:?}, extras {:?}",
            name, entity, parent, extra
        );

        let type_registry: &AppTypeRegistry = world.resource();
        let type_registry = type_registry.read();
        let reflect_components = ronstring_to_reflect_component(&extra.value, &type_registry);
        // let name = name.unwrap_or(&Name::new(""));

        let (target_entity, updated_components) =
            find_entity_components(entity, name, parent, reflect_components, &entity_components);
        entity_components.insert(target_entity, updated_components);
    }

    for (entity, name, extra, parent) in scene_extras.iter(world) {
        debug!(
            "Gltf Scene Extra: Name: {:?}, entity {:?}, parent: {:?}, scene_extras {:?}",
            name, entity, parent, extra
        );

        let type_registry: &AppTypeRegistry = world.resource();
        let type_registry = type_registry.read();
        let reflect_components = ronstring_to_reflect_component(&extra.value, &type_registry);

        let (target_entity, updated_components) =
            find_entity_components(entity, name, parent, reflect_components, &entity_components);
        entity_components.insert(target_entity, updated_components);
    }

    for (entity, name, extra, parent) in mesh_extras.iter(world) {
        debug!(
            "Gltf Mesh Extra: Name: {:?}, entity {:?}, parent: {:?}, mesh_extras {:?}",
            name, entity, parent, extra
        );

        let type_registry: &AppTypeRegistry = world.resource();
        let type_registry = type_registry.read();
        let reflect_components = ronstring_to_reflect_component(&extra.value, &type_registry);

        let (target_entity, updated_components) =
            find_entity_components(entity, name, parent, reflect_components, &entity_components);
        entity_components.insert(target_entity, updated_components);
    }

    for (entity, name, extra, parent) in material_extras.iter(world) {
        debug!(
            "Name: {:?}, entity {:?}, parent: {:?}, material_extras {:?}",
            name, entity, parent, extra
        );

        let type_registry: &AppTypeRegistry = world.resource();
        let type_registry = type_registry.read();
        let reflect_components = ronstring_to_reflect_component(&extra.value, &type_registry);

        let (target_entity, updated_components) =
            find_entity_components(entity, name, parent, reflect_components, &entity_components);
        entity_components.insert(target_entity, updated_components);
    }

    for (entity, components) in entity_components {
        let type_registry: &AppTypeRegistry = world.resource();
        let type_registry = type_registry.clone();
        let type_registry = type_registry.read();

        if !components.is_empty() {
            debug!("--entity {:?}, components {}", entity, components.len());
        }
        for (component, type_registration) in components {
            debug!(
                "------adding {} {:?}",
                component.get_represented_type_info().unwrap().type_path(),
                component
            );

            {
                let mut entity_mut = world.entity_mut(entity);
                let Some(reflected_component) = type_registration.data::<ReflectComponent>() else {
                    warn!(?component, "unable to reflect component");
                    entity_mut.insert(GltfProcessed);
                    continue;
                };
                reflected_component.insert(&mut entity_mut, &*component.into_partial_reflect(), &type_registry);

                entity_mut.insert(GltfProcessed); //
            }
        }
    }
}
