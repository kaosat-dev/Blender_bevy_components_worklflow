use bevy::{
    app::{App, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        reflect::ReflectComponent,
        schedule::{IntoSystemConfigs, IntoSystemSetConfigs, SystemSet},
        system::{Commands, Query, Res, ResMut, Resource, SystemParam},
    },
    hierarchy::{HierarchyQueryExt, Parent},
    log::warn,
    prelude::Update,
    reflect::{Reflect, TypePath},
    scene::SceneInstance,
    utils::HashMap,
};
use bevy_gltf_components::GltfComponentsSet;
use std::marker::PhantomData;

/// Plugin for keeping the GltfRefMap in sync.
pub struct GltfRefMapPlugin;

impl Plugin for GltfRefMapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GltfRefTarget>()
            .init_resource::<GltfRefMap>()
            .configure_sets(
                Update,
                (GltfRefsSet::CreateRefMap, GltfRefsSet::ResolveRefs)
                    .chain()
                    .after(GltfComponentsSet::Injection),
            )
            .add_systems(
                Update,
                GltfRefMap::create_ref_map.in_set(GltfRefsSet::CreateRefMap),
            );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
/// SystemSet to order your systems based on when references are resolved.
pub enum GltfRefsSet {
    /// Systems that create the [`GltfRefMap`] for that frame.
    CreateRefMap,
    /// Systems that resolve references using the [`GltfRefMap`]. For types that
    /// need to have references manually resolved, prefer to put their systems
    /// here.
    ResolveRefs,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GltfRefTarget(String);

/// A map of references for Gltf instances.
#[derive(Resource, Default)]
pub struct GltfRefMap(HashMap<Entity, HashMap<String, Entity>>);

impl GltfRefMap {
    /// Gets an entity given its `gltf_root` and `ref_name`. A reference is
    /// only valid on the same frame that the Gltf is spawned in. In other
    /// words, after that same frame, any references on `gltf_root` will return
    /// None.
    pub fn get_ref(&self, gltf_root: Entity, ref_name: &String) -> Option<Entity> {
        self.0.get(&gltf_root)?.get(ref_name).copied()
    }

    fn create_ref_map(
        ref_targets: Query<(Entity, &GltfRefTarget)>,
        gltf_for_entity: GltfForEntity,
        mut ref_map: ResMut<GltfRefMap>,
        mut commands: Commands,
    ) {
        ref_map.0.clear();

        for (entity, ref_target) in ref_targets.iter() {
            commands.entity(entity).remove::<GltfRefTarget>();
            let Some(gltf_root) = gltf_for_entity.find_gltf_root(entity) else {
                warn!("Entity {entity:?} is not part of a Gltf but contains a GltfRefTarget (target={}).", ref_target.0);
                continue;
            };
            ref_map
                .0
                .entry(gltf_root)
                .or_default()
                .insert(ref_target.0.clone(), entity);
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GltfRef<T: Component> {
    target: String,
    #[reflect(ignore)]
    _marker: PhantomData<T>,
}

impl<T: Component> GltfRef<T>
where
    Entity: Into<T>,
{
    fn system(
        refs: Query<(Entity, &Self)>,
        gltf_for_entity: GltfForEntity,
        ref_map: Res<GltfRefMap>,
        mut commands: Commands,
    ) {
        for (entity, gltf_ref) in refs.iter() {
            commands.entity(entity).remove::<Self>();
            let Some(gltf_root) = gltf_for_entity.find_gltf_root(entity) else {
                warn!("GltfRef should only be on descendants of a Gltf.");
                continue;
            };

            match ref_map.get_ref(gltf_root, &gltf_ref.target) {
                Some(target) => {
                    commands.entity(entity).insert(Into::<T>::into(target));
                }
                None => {
                    warn!(
                        "Entity {entity:?} attempted to reference '{}' in gltf {gltf_root:?}",
                        &gltf_ref.target
                    );
                }
            }
        }
    }
}

/// Plugin for automatically converting [`GltfRef`]s into their corresponding
/// `T`.
pub struct GltfRefPlugin<T: Component + TypePath>(PhantomData<T>);

impl<T: Component + TypePath> Plugin for GltfRefPlugin<T>
where
    Entity: Into<T>,
{
    fn build(&self, app: &mut App) {
        app.register_type::<GltfRef<T>>().add_systems(
            Update,
            GltfRef::<T>::system.in_set(GltfRefsSet::ResolveRefs),
        );
    }
}

// Manual implementation of Default for GltfRefPlugin to avoid `Default` trait
// bounds on `T`.
impl<T: Component + TypePath> Default for GltfRefPlugin<T>
where
    Entity: Into<T>,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

/// SystemParam to find the Gltf that an entity belongs to.
#[derive(SystemParam)]
pub struct GltfForEntity<'w, 's> {
    gltfs: Query<'w, 's, (), With<SceneInstance>>,
    hierarchy: Query<'w, 's, &'static Parent>,
}

impl<'w, 's> GltfForEntity<'w, 's> {
    /// Finds the Gltf root for `entity`. Returns None if `entity` does not
    /// belong to a Gltf.
    pub fn find_gltf_root(&self, entity: Entity) -> Option<Entity> {
        for entity in std::iter::once(entity).chain(self.hierarchy.iter_ancestors(entity)) {
            if self.gltfs.contains(entity) {
                return Some(entity);
            }
        }
        None
    }
}
