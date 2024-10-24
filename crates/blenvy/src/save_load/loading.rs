use bevy::prelude::*;

use crate::{BlueprintInfo, DynamicEntitiesRoot, GameWorldTag, HideUntilReady, SpawnBlueprint};

#[derive(Event)]
pub struct LoadingRequest {
    pub path: String,
}

#[derive(Event)]
pub struct LoadingFinished; // TODO: merge the two above

/// resource that keeps track of the current load request
#[derive(Resource, Default)]
pub struct LoadingRequested {
    pub path: String,
}

/*
- Loading
    - load request recieved
    - pause things ?
    - unload everything
    - load static data using blueprintInfo
    - load dynamic data from save file


General:
    * wrap loading a bevy scene as a blueprint ?
    * meh, has no assets & co, different logic ?
*/
pub fn process_load_requests(
    mut load_requests: EventReader<LoadingRequest>,
    mut commands: Commands,
) {
    let mut save_path: String = "".into();
    for load_request in load_requests.read() {
        if !load_request.path.is_empty() {
            save_path.clone_from(&load_request.path);
        }
    }
    if !save_path.is_empty() {
        commands.insert_resource(LoadingRequested { path: save_path });
    }
}

pub fn should_load(loading_requests: Option<Res<LoadingRequested>>) -> bool {
    resource_exists::<LoadingRequested>(loading_requests)
}

// TODO: replace with generic despawner ?
pub(crate) fn prepare_loading(
    mut commands: Commands,
    gameworlds: Query<Entity, With<GameWorldTag>>,
) {
    for e in gameworlds.iter() {
        info!("--loading: despawn old world/level");
        commands.entity(e).despawn_recursive();
    }
}

pub(crate) fn load_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    load_request: Res<LoadingRequested>,
) {
    info!("--loading: load dynamic data");

    //let save_path = Path::new(load_request.path.clone().as_str());

    info!("LOADING FROM {:?}", load_request.path.clone());

    /*let world_root = commands
    .spawn((
        bevy::prelude::Name::from("world"),
        GameWorldTag,
        TransformBundle::default(),
        InheritedVisibility::default(),
    ))
    .id();*/

    // and we fill it with dynamic data
    // let input = std::fs::read(&path)?;
    let _dynamic_data = commands
        .spawn((
            DynamicSceneRoot(asset_server.load(load_request.path.clone())),
            bevy::prelude::Name::from("World_dynamic"),
            DynamicEntitiesRoot,
            GameWorldTag,
        ))
        .id();

    let _static_data = commands
        .spawn((
            BlueprintInfo::from_path("levels/World.glb"), // all we need is a Blueprint info...
            SpawnBlueprint,
            HideUntilReady,
            GameWorldTag,
        ))
        .id();

    //commands.entity(world_root).add_child(static_data);
    //commands.entity(world_root).add_child(dynamic_data);

    // commands.insert_resource(LoadFirstStageDone);

    info!("--loading: loaded dynamic data");
    commands.remove_resource::<LoadingRequested>();
}
