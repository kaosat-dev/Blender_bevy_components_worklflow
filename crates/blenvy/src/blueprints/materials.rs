use bevy::prelude::*;

use crate::BlenvyConfig;

#[derive(Reflect, Default, Debug)]
/// struct containing the name & path of the material to apply
pub struct MaterialInfo {
    pub name: String,
    pub path: String,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// component containing the full list of `MaterialInfos` for a given entity/object
pub struct MaterialInfos(Vec<MaterialInfo>);

#[derive(Component, Default, Debug)]
pub struct MaterialProcessed;

/// system that injects / replaces materials from materials library
pub(crate) fn inject_materials(
    mut blenvy_config: ResMut<BlenvyConfig>,
    material_infos_query: Query<
        (Entity, &MaterialInfos, &Children),
        Without<MaterialProcessed>, // (With<BlueprintReadyForPostProcess>)
                                    /*(
                                        Added<BlueprintMaterialAssetsLoaded>,
                                        With<BlueprintMaterialAssetsLoaded>,
                                    ),*/
    >,
    with_materials_and_meshes: Query<
        (),
        (
            With<Parent>,
            With<MeshMaterial3d<StandardMaterial>>,
            With<Mesh3d>,
        ),
    >,
    assets_gltf: Res<Assets<Gltf>>,
    asset_server: Res<AssetServer>,

    mut commands: Commands,
) {
    for (entity, material_infos, children) in material_infos_query.iter() {
        for (material_index, material_info) in material_infos.0.iter().enumerate() {
            let material_full_path = format!("{}#{}", material_info.path, material_info.name);
            let mut material_found: Option<&Handle<StandardMaterial>> = None;
            if blenvy_config
                .materials_cache
                .contains_key(&material_full_path)
            {
                debug!("material is cached, retrieving");
                let material = blenvy_config
                    .materials_cache
                    .get(&material_full_path)
                    .expect("we should have the material available");
                material_found = Some(material);
            } else {
                let model_handle: Handle<Gltf> = asset_server.load(material_info.path.clone()); // FIXME: kinda weird now
                let Some(mat_gltf) = assets_gltf.get(model_handle.id()) else {
                    warn!(
                        "materials file {} should have been preloaded skipping",
                        material_info.path
                    );
                    continue;
                };
                /*let mat_gltf = assets_gltf.get(model_handle.id()).unwrap_or_else(|| {
                    panic!(
                        "materials file {} should have been preloaded",
                        material_info.path
                    )
                });*/
                if mat_gltf
                    .named_materials
                    .contains_key(&material_info.name as &str)
                {
                    let material = mat_gltf
                        .named_materials
                        .get(&material_info.name as &str)
                        .expect("this material should have been loaded at this stage, please make sure you are correctly preloading them");
                    blenvy_config
                        .materials_cache
                        .insert(material_full_path, material.clone());
                    material_found = Some(material);
                }
            }

            if let Some(material) = material_found {
                info!("Step 6: injecting/replacing materials");
                for (child_index, child) in children.iter().enumerate() {
                    if child_index == material_index && with_materials_and_meshes.contains(*child) {
                        info!(
                            "injecting material {}, path: {:?}",
                            material_info.name,
                            material_info.path.clone()
                        );

                        commands.entity(*child).insert(MeshMaterial3d(material.clone()));
                    }
                }
            }
        }

        commands.entity(entity).insert(MaterialProcessed);
    }
}
