use bevy::prelude::*;
use blenvy::{
    AddToGameWorld, BlenvyPlugin, BluePrintBundle, BlueprintInfo, Dynamic, GameWorldTag,
    HideUntilReady, SpawnBlueprint,
};
use rand::Rng;

mod component_examples;
use component_examples::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin::default()),
            // our custom plugins
            ComponentsExamplesPlugin, // Showcases different type of components /structs
            BlenvyPlugin::default(),
        ))
        .add_systems(Startup, setup_game)
        .add_systems(Update, spawn_blueprint_instance)
        .run();
}

// this is how you setup & spawn a level from a blueprint
fn setup_game(mut commands: Commands) {
    // here we spawn our game world/level, which is also a blueprint !
    commands.spawn((
        BlueprintInfo::from_path("levels/Level1.glb"), // all we need is a Blueprint info...
        SpawnBlueprint, // and spawnblueprint to tell blenvy to spawn the blueprint now
        HideUntilReady, // only reveal the level once it is ready
        GameWorldTag,
    ));
}

// you can also spawn blueprint instances at runtime
pub fn spawn_blueprint_instance(keycode: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keycode.just_pressed(KeyCode::KeyS) {
        let mut rng = rand::thread_rng();
        let range = 5.5;
        let x: f32 = rng.gen_range(-range..range);
        let y: f32 = rng.gen_range(-range..range);

        let name_index: u64 = rng.gen();

        let __new_entity = commands
            .spawn((
                BluePrintBundle {
                    blueprint: BlueprintInfo {
                        name: "spawned".into(),
                        path: "blueprints/Blueprint 3.gltf".into(),
                    }, // FIXME
                    ..Default::default()
                },
                Dynamic,
                bevy::prelude::Name::from(format!("test{}", name_index)),
                HideUntilReady,
                AddToGameWorld,
                TransformBundle::from_transform(Transform::from_xyz(x, 2.0, y)),
            ))
            .id();
        //         commands.entity(world).add_child(new_entity);
    }
}
