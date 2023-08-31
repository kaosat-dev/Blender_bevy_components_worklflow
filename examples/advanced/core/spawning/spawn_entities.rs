use rand::Rng;
use bevy::prelude::*;
use bevy::gltf::Gltf;
use bevy_rapier3d::prelude::*; // FIXME: temporary: used for velocity of newly spawned items

use crate::{assets::{LibraryPack, GameAssets}, core::AnimationHelper};
use super::{SpawnRequestedEvent, Spawner, GameWorldTag, ItemType};

pub fn spawn_entities(
    mut spawn_requested_events: EventReader<SpawnRequestedEvent>,  
    mut commands: Commands,
  
    library_pack: Res<LibraryPack>,
    assets_gltf: Res<Assets<Gltf>>,
    mut game_world: Query<Entity, With<GameWorldTag>>,
  ){
  
    for spawn_request in spawn_requested_events.iter() {
  
      //let (spawner, transform) = spawners.get(ev.spawner_id).unwrap();
      //let position = transform.translation.clone() + Vec3::new(0.0, 0.0, 0.5);
      let what = spawn_request.what.clone();
      let position = spawn_request.position;
      let amount = spawn_request.amount;
  
      if let Some(library) = assets_gltf.get(&library_pack.0) {
        info!("attempting to spawn {}", what);
        if !library.named_scenes.contains_key(&what) {
          warn!("there is no item in the library called {}", what)
        } else {
          let world = game_world.single_mut();
          // println!("spawner velocity ranges {:?}", spawner.vel_x_range);
  
          for _ in 0..amount {
           
            let child_scene = commands.spawn(
              (
                  SceneBundle {
                      scene: library.named_scenes[&what.to_string()].clone(),
                      transform: Transform::from_translation(position),
                      ..Default::default()
                  },
                  bevy::prelude::Name::from(["scene_wrapper", &what.clone()].join("_") ),
                  // Parent(world) // FIXME/ would be good if this worked directly
                  SpawnedRoot,
              )).id();
              commands.entity(world).add_child(child_scene);
          }
  
        }  
      }
    }
  }
  


  pub fn spawn_entities2(
    mut spawn_requested_events: EventReader<SpawnRequestedEvent>,  
    game_assets: Res<GameAssets>,
    mut commands: Commands,

    mut game_world: Query<(Entity, &Children), With<GameWorldTag>>,

    assets_gltf: Res<Assets<Gltf>>,
  ) {
    for spawn_request in spawn_requested_events.iter() {
      let what = spawn_request.what.clone();
      let position = spawn_request.position;
      let amount = spawn_request.amount;

      let model_path = format!("models/library/{}.glb", &what);
      let scene = game_assets.models.get(&model_path).expect(&format!("no matching model {:?} found", model_path));
      info!("attempting to spawn {:?}",model_path);
      // let scene_foo = assets_scenes.get(scene).expect("this scene should have been loaded");
      let world = game_world.single_mut();
      let world = world.1[0]; // FIXME: dangerous hack because our gltf data have a single child like this, but might not always be the case
      
      let gltf = assets_gltf.get(scene).expect("this gltf should have been loaded");
      let scene = &gltf.named_scenes["Library"];
      // println!("ANIMATIONS {:?}", gltf.named_animations);
     
      for _ in 0..amount {
        let child_scene = commands.spawn(
          (
              SceneBundle {
                  scene: scene.clone(),
                  transform: Transform::from_translation(position),
                  ..Default::default()
              },
              bevy::prelude::Name::from(["scene_wrapper", &what.clone()].join("_") ),
              // Parent(world) // FIXME/ would be good if this worked directly
              SpawnedRoot,
              AnimationHelper{ // TODO: insert this at the ENTITY level, not the scene level
                named_animations: gltf.named_animations.clone(),
                // animations: gltf.named_animations.values().clone()
              },
          )).id();
          commands.entity(world).add_child(child_scene);
      }
      
     
    }
  }

  
  #[derive(Component)]
  /// FlagComponent for dynamically spawned scenes
  pub struct SpawnedRoot;
  
  #[derive(Component)]
  /// FlagComponent for dynamically spawned scenes
  pub struct SpawnedRootProcessed;
  
  /// this system updates the first (and normally only) child of a scene flaged SpawnedRoot
  /// - adds a name based on parent component (spawned scene) which is named on the scene name/prefab to be instanciated
  /// - adds the initial physics impulse (FIXME: we would need to add a temporary physics component to those who do not have it)
  // FIXME: updating hierarchy does not work in all cases ! this is sadly dependant on the structure of the exported blend data, disablin for now
  // - blender root-> object with properties => WORKS
  // - scene instance -> does not work
  // it might be due to how we add components to the PARENT item in gltf to components
  pub(crate) fn update_spawned_root_first_child(
    added_spawned_roots :Query<(Entity, &Children, &Name), Added<SpawnedRoot>>,
    all_children: Query<(Entity, &Children)>,
  
    mut commands: Commands,
  
    test :Query<(Entity, &Children, &Name ), (With<SpawnedRoot>, Without<SpawnedRootProcessed>)>,
    // tmp: Query<(Entity, &Name, &Children), Added<SpawnedRoot>>, 
    mut game_world: Query<(Entity, &Children), With<GameWorldTag>>,

    // FIXME: should be done at a more generic gltf level
    animation_helpers: Query<&AnimationHelper>,
    added_animation_helpers : Query<(Entity, &AnimationPlayer), (Added<AnimationPlayer>)>
  ){
    /*
        currently we have
        - scene instance 
            - root node ?
                - the actual stuff
        we want to remove the root node 
        so we wend up with 
        - scene instance
            - the actual stuff
  
        so 
            - get root node
            - add its children to the scene instance
            - remove root node
        
        Another issue is, the scene instance become empty if we have a pickabke as the "actual stuff", so that would lead to a lot of
            empty scenes if we spawn pickables
        - perhaps another system that cleans up empty scene instances ?
  
        FIME: this is all highly dependent on the hierachy ;..
     */
   
    
    for (scene_instance, chidren, name) in test.iter()  {
        println!("children of scene {:?}", chidren);
        let root_node = chidren.first().unwrap(); //FIXME: and what about childless ones ??
        let root_node_data = all_children.get(*root_node).unwrap();
  
        // fixme : randomization should be controlled via parameters, perhaps even the seed could be specified ?
        // use this https://rust-random.github.io/book/guide-seeding.html#a-simple-number, blenders seeds are also uInts
        // also this is not something we want every time, this should be a settable parameter when requesting a spawn
        let mut rng = rand::thread_rng();
        let range = 0.8;
        let vel_x: f32 = rng.gen_range(-range..range);
        let vel_y: f32 = rng.gen_range(-range..range);
        let vel_z: f32 = rng.gen_range(2.0..2.5);
        // add missing name of entity, based on the wrapper's name
        let name= name.clone().replace("scene_wrapper_", "");
        //ItemType
        commands.entity(*root_node).insert((
            bevy::prelude::Name::from(name.clone()),
            ItemType {name},
            Velocity {
              linvel: Vec3::new(vel_x, vel_y, vel_z),
              angvel: Vec3::new(0.0, 0.0, 0.0),
            },
        ));
  
        // flag the spawned_root as being processed
        commands.entity(scene_instance).insert(SpawnedRootProcessed);
  
        // these are the things we want, move them one level up
        let actual_stuff = root_node_data.1; 
        let world = game_world.single_mut();
        let world = world.1[0]; // FIXME: dangerous hack because our gltf data have a single child like this, but might not always be the case
        // commands.entity(world).push_children(&actual_stuff);
        //
        // let original_transforms = 
        // commands.entity(world).add_child(*root_node);
        // commands.entity(*root_node).despawn_recursive();

        let matching_animation_helper = animation_helpers.get(scene_instance);
        
        // println!("WE HAVE SOME ADDED ANIMATION PLAYERS {:?}", matching_animation_helper);
        if let Ok(anim_helper) = matching_animation_helper{
          for (added, _) in added_animation_helpers.iter(){
            commands.entity(added).insert(
              AnimationHelper{ // TODO: insert this at the ENTITY level, not the scene level
                named_animations: anim_helper.named_animations.clone(),
                // animations: gltf.named_animations.values().clone()
              },
            );
          }
        }

    }
  }
  
  /// cleans up dynamically spawned scenes so that they get despawned if they have no more children
  // FIXME: this can run too early , and since withouth_children matches not yet completed scene instances, boom, it removes components/children before they can be used
  pub(crate) fn cleanup_scene_instances(
    scene_instances: Query<(Entity, &Children), With<SpawnedRootProcessed>>,
    without_children: Query<Entity, (With<SpawnedRootProcessed>, Without<Children>)>,// if there are not children left, bevy removes Children ?
    mut commands: Commands
  ){
    for (entity, children) in scene_instances.iter(){
        if children.len() == 0{ // it seems this does not happen ?
            println!("empty scene instance can be cleaned up");
            commands.entity(entity).despawn_recursive();
        }
    }
    for entity in without_children.iter() {
        println!("empty scene instance can be cleaned up");
        commands.entity(entity).despawn_recursive();
    }
  }