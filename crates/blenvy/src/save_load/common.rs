pub(crate) use bevy::prelude::*;

use crate::{BlueprintInfo, GameWorldTag, HideUntilReady, SpawnBlueprint};

use super::BlueprintWorld;

pub(crate) fn spawn_from_blueprintworld(
    added_blueprint_worlds: Query<(Entity, &BlueprintWorld), Added<BlueprintWorld>>,
    mut commands: Commands,
) {
    for (__entity, blueprint_world) in added_blueprint_worlds.iter() {
        println!("added blueprintWorld {:?}", blueprint_world);

        // here we spawn the static part our game world/level, which is also a blueprint !
        let __static_world = commands
            .spawn((
                BlueprintInfo::from_path(blueprint_world.path.as_str()), // all we need is a Blueprint info...
                SpawnBlueprint, // and spawnblueprint to tell blenvy to spawn the blueprint now
                HideUntilReady, // only reveal the level once it is ready
                GameWorldTag,
            ))
            .id();

        // here we spawn the dynamic entities part of our game world/level, which is also a blueprint !
        let __dynamic_world = commands
            .spawn((
                BlueprintInfo::from_path(
                    blueprint_world
                        .path
                        .replace(".glb", "_dynamic.glb")
                        .replace(".gltf", "_dynamic.gltf")
                        .as_str(),
                ), // all we need is a Blueprint info...
                SpawnBlueprint, // and spawnblueprint to tell blenvy to spawn the blueprint now
                HideUntilReady, // only reveal the level once it is ready
                GameWorldTag,
            ))
            .id();

        // commands.entity(entity).add_child(static_world);
        // commands.entity(entity).add_child(dynamic_world);
    }
}

/*
pub(crate) fn inject_dynamic_into_children(
    added_dynamic: Query<Entity, Added<Dynamic> >,
    all_children: Query<&Children>,
    mut commands: Commands,
) {
    for entity in added_dynamic.iter() {
        for child in all_children.iter_descendants(entity) {
            commands.entity(child).insert(Dynamic);
        }
    }
}*/
