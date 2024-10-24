use std::fs::File;
use std::io::Write;
use std::path::Path;

use bevy::prelude::World;
use bevy::{prelude::*, tasks::IoTaskPool};

use crate::{BlenvyConfig, BlueprintInfo, Dynamic, FromBlueprint, RootEntity, SpawnBlueprint};

use super::{DynamicEntitiesRoot, OriginalParent, StaticEntitiesRoot};

#[derive(Event, Debug)]
pub struct SavingRequest {
    pub path: String,
}
#[derive(Event)]
pub struct SaveFinished; // TODO: merge the the events above

/// resource that keeps track of the current save request
#[derive(Resource, Default)]
pub struct SavingRequested {
    pub path: String,
}

pub fn process_save_requests(
    mut saving_requests: EventReader<SavingRequest>,
    mut commands: Commands,
) {
    let mut save_path: String = "".into();
    for saving_request in saving_requests.read() {
        if !saving_request.path.is_empty() {
            save_path.clone_from(&saving_request.path);
        }
    }
    if !save_path.is_empty() {
        commands.insert_resource(SavingRequested { path: save_path });
    }
}

pub fn should_save(saving_requests: Option<Res<SavingRequested>>) -> bool {
    resource_exists::<SavingRequested>(saving_requests)
}

// any child of dynamic/ saveable entities that is not saveable itself should be removed from the list of children
pub(crate) fn prepare_save_game(
    saveables: Query<Entity, (With<Dynamic>, With<BlueprintInfo>)>,
    root_entities: Query<Entity, Or<(With<DynamicEntitiesRoot>, Without<Parent>)>>, //  With<DynamicEntitiesRoot>
    dynamic_entities: Query<(Entity, &Parent, Option<&Children>), With<Dynamic>>,
    _static_entities: Query<(Entity, &BlueprintInfo), With<StaticEntitiesRoot>>,

    mut commands: Commands,
) {
    for entity in saveables.iter() {
        // FIXME : not sure about this one
        commands.entity(entity).insert(SpawnBlueprint);
    }

    for (entity, parent, children) in dynamic_entities.iter() {
        println!("prepare save game for entity");
        let parent = parent.get();
        if root_entities.contains(parent) {
            commands.entity(entity).insert(RootEntity);
        }

        if let Some(children) = children {
            for sub_child in children.iter() {
                if !dynamic_entities.contains(*sub_child) {
                    commands.entity(*sub_child).insert(OriginalParent(entity));
                    commands.entity(entity).remove_children(&[*sub_child]);
                }
            }
        }
    }
    /*for (_, blueprint_name) in static_entities.iter() {
        let library_path: String = library.map_or_else(|| "", |l| l.0.to_str().unwrap()).into();
        commands.insert_resource(StaticEntitiesStorage {
            name: blueprint_name.0.clone(),
            library_path,
        });
    }*/
}

pub(crate) fn save_game(world: &mut World) {
    info!("saving");

    let mut save_path: String = "".into();
    let mut events = world.resource_mut::<Events<SavingRequest>>();

    for event in events.get_cursor().read(&events) {
        info!("SAVE EVENT !! {:?}", event);
        save_path.clone_from(&event.path);
    }
    events.clear();

    let saveable_entities: Vec<Entity> = world
        .query_filtered::<Entity, (With<Dynamic>, Without<RootEntity>)>()
        // .query_filtered::<Entity, (With<Dynamic>, Without<FromBlueprint>, Without<RootEntity>)>()
        .iter(world)
        .collect();

    let saveable_root_entities: Vec<Entity> = world
        // .query_filtered::<Entity, (With<Dynamic>, With<RootEntity>)>()
        .query_filtered::<Entity, (With<Dynamic>, Without<FromBlueprint>, With<RootEntity>)>()
        .iter(world)
        .collect();

    info!("saveable entities {}", saveable_entities.len());
    info!("saveable root entities {}", saveable_root_entities.len());

    let config = world
        .get_resource::<BlenvyConfig>()
        .expect("Blenvy configuration should exist at this stage");

    // we hardcode some of the always allowed types
    let filter = config
        .save_component_filter
        .clone()
        //.allow::<Parent>() // TODO: add back
        .allow::<Children>()
        .allow::<BlueprintInfo>()
        .allow::<SpawnBlueprint>()
        .allow::<Dynamic>()
        /*.deny::<CameraRenderGraph>()
        .deny::<CameraMainTextureUsages>()
        .deny::<Handle<Mesh>>()
        .deny::<Handle<StandardMaterial>>() */
        ;

    // for root entities, it is the same EXCEPT we make sure parents are not included
    let filter_root = filter.clone().deny::<Parent>();

    let filter_resources = config
        .clone()
        .save_resource_filter
        .deny::<Time<Real>>()
        .clone();
    //.allow::<StaticEntitiesStorage>();

    // for default stuff
    let scene_builder = DynamicSceneBuilder::from_world(world)
        .with_component_filter(filter.clone())
        .with_resource_filter(filter_resources.clone());

    let dyn_scene = scene_builder
        .extract_resources()
        .extract_entities(saveable_entities.clone().into_iter())
        .remove_empty_entities()
        .build();

    // for root entities
    let scene_builder_root = DynamicSceneBuilder::from_world(world)
        .with_component_filter(filter_root.clone())
        .with_resource_filter(filter_resources.clone());

    let mut __dyn_scene_root = scene_builder_root
        .extract_resources()
        .extract_entities(
            saveable_root_entities.clone().into_iter(), // .chain(static_world_markers.into_iter()),
        )
        .remove_empty_entities()
        .build();

    // dyn_scene.entities.append(&mut dyn_scene_root.entities);
    // dyn_scene.resources.append(&mut dyn_scene_root.resources);

    let serialized_scene = dyn_scene
        .serialize(&world.resource::<AppTypeRegistry>().read())
        .expect("filtered scene should serialize correctly");

    let save_path_assets = Path::new("assets")
        //.join(&config.save_path)
        .join(Path::new(save_path.as_str())); // Path::new(&save_load_config.save_path).join(Path::new(save_path.as_str()));
    info!("saving game to {:?}", save_path_assets);

    // world.send_event(SavingFinished);
    let bla = save_path_assets.clone().to_string_lossy().into_owned();

    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(save_path_assets)
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .expect("Error while writing save to file");
        })
        .detach();

    let static_world_path = "levels/world.glb";
    let fake_foo = format!("(dynamic: {bla}, static: {static_world_path})");
    let real_save_path = format!("{bla}.save.ron");
    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(real_save_path)
                .and_then(|mut file| file.write(fake_foo.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();
}

pub(crate) fn cleanup_save(
    needs_parent_reset: Query<(Entity, &OriginalParent)>,
    mut saving_finished: EventWriter<SaveFinished>,
    mut commands: Commands,
) {
    for (entity, original_parent) in needs_parent_reset.iter() {
        commands.entity(original_parent.0).add_child(entity);
    }
    // commands.remove_resource::<StaticEntitiesStorage>();
    saving_finished.send(SaveFinished);

    commands.remove_resource::<SavingRequested>();
}
