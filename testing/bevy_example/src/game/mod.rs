pub mod in_game;
pub use in_game::*;

pub mod animation;
pub use animation::*;

use std::{collections::HashMap, fs, time::Duration};

use blenvy::{
    BlueprintAnimationPlayerLink, BlueprintAssets, BlueprintEvent, BlueprintInfo,
    InstanceAnimations,
};

use crate::{AppState, GameState};
use bevy::{
    prelude::*,
    time::common_conditions::on_timer,
    render::view::screenshot::{save_to_disk, Capturing, Screenshot},
    window::PrimaryWindow,
};

use json_writer::to_json_string;

fn start_game(mut next_app_state: ResMut<NextState<AppState>>) {
    println!("START GAME");
    //next_app_state.set(AppState::AppLoading);
    next_app_state.set(AppState::AppRunning);
}

// if the export from Blender worked correctly, we should have animations (simplified here by using AnimationPlayerLink)
// if the export from Blender worked correctly, we should have an Entity called "Blueprint4_nested" that has a child called "Blueprint3" that has a "BlueprintInfo" component with value Blueprint3
// if the export from Blender worked correctly, we should have an assets_list
// if the export from Blender worked correctly, we should have the correct tree of entities
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn validate_export(
    parents: Query<&Parent>,
    children: Query<&Children>,
    names: Query<&Name>,
    blueprints: Query<(Entity, &Name, &BlueprintInfo)>,
    animation_player_links: Query<(Entity, &BlueprintAnimationPlayerLink)>,
    scene_animations: Query<(Entity, &InstanceAnimations)>,
    empties_candidates: Query<(Entity, &Name, &GlobalTransform)>,

    assets_list: Query<(Entity, &BlueprintAssets)>,
    root: Query<(Entity, &Name, &Children), (Without<Parent>, With<Children>)>,
) {
    let animations_found =
        !animation_player_links.is_empty() && scene_animations.into_iter().len() == 4;

    let mut nested_blueprint_found = false;
    for (entity, name, blueprint_info) in blueprints.iter() {
        if name.to_string() == *"Blueprint4_nested" && blueprint_info.name == *"Blueprint4_nested" {
            if let Ok(cur_children) = children.get(entity) {
                for child in cur_children.iter() {
                    if let Ok((_, child_name, child_blueprint_info)) = blueprints.get(*child) {
                        if child_name.to_string() == *"Blueprint3"
                            && child_blueprint_info.name == *"Blueprint3"
                        {
                            nested_blueprint_found = true;
                        }
                    }
                }
            }
        }
    }

    let mut empty_found = false;
    for (_, name, _) in empties_candidates.iter() {
        if name.to_string() == *"Empty" {
            empty_found = true;
            break;
        }
    }
    // check if there are assets_list components
    let assets_list_found = !assets_list.is_empty();

    // there should be no entity named xxx____bak as it means an error in the Blender side export process
    let mut exported_names_correct = true;
    for name in names.iter() {
        if name.to_string().ends_with("___bak") {
            exported_names_correct = false;
            break;
        }
    }

    // generate parent/child "tree"
    if !root.is_empty() {
        let root = root.single();
        let mut tree: HashMap<String, Vec<String>> = HashMap::new();

        for child in children.iter_descendants(root.0) {
            let child_name: String = names
                .get(child)
                .map_or(String::from("no_name"), |e| e.to_string()); //|e| e.to_string(), || "no_name".to_string());
                                                                     //println!("  child {}", child_name);
            let parent = parents.get(child).unwrap();
            let parent_name: String = names
                .get(parent.get())
                .map_or(String::from("no_name"), |e| e.to_string()); //|e| e.to_string(), || "no_name".to_string());
            tree.entry(parent_name)
                .or_default()
                .push(child_name.clone());
        }

        let hierarchy = to_json_string(&tree);
        fs::write("bevy_hierarchy.json", hierarchy).expect("unable to write hierarchy file")
    }

    fs::write(
        "bevy_diagnostics.json",
        format!(
            "{{ \"animations\": {},  \"nested_blueprint_found\": {}, \"empty_found\": {}, \"assets_list_found\": {}, \"exported_names_correct\": {} }}",
            animations_found, nested_blueprint_found, empty_found, assets_list_found, exported_names_correct
        ),
    )
    .expect("Unable to write file");
}

fn generate_screenshot(
    mut commands: Commands,
) {
    commands
      .spawn(Screenshot::primary_window())
      .observe(save_to_disk("screenshot.png"));
}

fn exit_game(mut app_exit_events: ResMut<Events<bevy::app::AppExit>>) {
    app_exit_events.send(bevy::app::AppExit::Success);
}

fn check_for_gltf_events(
    mut blueprint_events: EventReader<BlueprintEvent>,
    all_names: Query<&Name>,
) {
    for event in blueprint_events.read() {
        match event {
            BlueprintEvent::InstanceReady {
                entity,
                blueprint_name: _,
                blueprint_path: _,
            } => {
                info!(
                    "BLUEPRINT EVENT: {:?} for {:?}",
                    event,
                    all_names.get(*entity)
                );
            }
            BlueprintEvent::AssetsLoaded {
                entity,
                blueprint_name: _,
                blueprint_path: _,
            } => {
                info!(
                    "BLUEPRINT EVENT: {:?} for {:?}",
                    event,
                    all_names.get(*entity)
                );
            }
        }
    }
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Marker1>()
            .register_type::<Marker2>()
            .register_type::<Marker3>()
            .register_type::<MarkerAllFoxes>()
            .register_type::<MarkerSpecificFox>()

            .add_systems(Update, (spawn_test).run_if(in_state(GameState::InGame)))
            .add_systems(Update, (validate_export, check_for_gltf_events))
            //.add_systems(OnEnter(AppState::CoreLoading), start_game)

            .add_systems(OnEnter(AppState::MenuRunning), start_game)
            .add_systems(OnEnter(AppState::AppRunning), setup_game)

            /* .add_systems(Update, (animations)
                .run_if(in_state(AppState::AppRunning))
                .after(GltfBlueprintsSet::AfterSpawn)
            )*/
            .add_systems(Update, play_animations) // check_animations
            //.add_systems(Update, react_to_animation_markers)

            .add_systems(Update, generate_screenshot.run_if(on_timer(Duration::from_secs_f32(0.2)))) // TODO: run once
            .add_systems(
                Update,
                exit_game.run_if(on_timer(Duration::from_secs_f32(0.5))),
            ) // shut down the app after this time
            ;
    }
}
