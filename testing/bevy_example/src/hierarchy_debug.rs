use bevy::{
    gltf::{GltfMaterialExtras, GltfMeshExtras, GltfSceneExtras},
    prelude::*,
};

use crate::{EnumTest, RedirectPropHitImpulse};

#[derive(Component)]
pub struct HiearchyDebugTag;

pub fn setup_hierarchy_debug(mut commands: Commands) {
    // a place to display the extras on screen
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        Text("".to_string()),
        TextFont {
            font_size: 15.,
            ..default()
        },
        TextColor(LinearRgba {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        }.into()),
        TextLayout {
            ..default()
        },
        HiearchyDebugTag,
    ));
}

pub fn get_descendants(
    all_children: &Query<&Children>,
    all_names: &Query<&Name>,
    root: &Entity,
    __all_transforms: &Query<&Transform>,
    __all_global_transforms: &Query<&GlobalTransform>,
    nesting: usize,
    to_check: &Query<&EnumTest>, //&Query<(&BlueprintInstanceReady, &BlueprintAssets)>,
) -> String {
    let mut hierarchy_display: Vec<String> = vec![];
    let root_name = all_names.get(*root);
    let name;
    if let Ok(root_name) = root_name {
        name = root_name.to_string();
    } else {
        name = "no_name".to_string();
    }

    let mut component_display: String = "".into();
    let __components_to_check = to_check.get(*root);

    if let Ok(compo) = to_check.get(*root) {
        component_display = format!("{:?}", compo);
    }

    hierarchy_display.push(format!(
        "{}{} ====> {} ",
        " ".repeat(nesting),
        name,
        component_display // all_transforms.get(*root),
                          //all_global_transforms.get(*root)
    )); //  ({:?}) // ({:?}) ({:?})

    if let Ok(children) = all_children.get(*root) {
        for child in children.iter() {
            let child_descendants_display = get_descendants(
                all_children,
                all_names,
                child,
                __all_transforms,
                __all_global_transforms,
                nesting + 4,
                to_check,
            );
            hierarchy_display.push(child_descendants_display);
        }
    }
    hierarchy_display.join("\n")
}

pub fn draw_hierarchy_debug(
    root: Query<Entity, Without<Parent>>,
    all_children: Query<&Children>,
    all_names: Query<&Name>,
    all_transforms: Query<&Transform>,
    all_global_transforms: Query<&GlobalTransform>,

    to_check: Query<&EnumTest>, //Query<(&BlueprintInstanceReady, &BlueprintAssets)>,
    display: Query<Entity, With<HiearchyDebugTag>>,
    mut writer: TextUiWriter,
) {
    let mut hierarchy_display: Vec<String> = vec![];

    for root_entity in root.iter() {
        // hierarchy_display.push( format!("Hierarchy root{:?}", name) );

        hierarchy_display.push(get_descendants(
            &all_children,
            &all_names,
            &root_entity,
            &all_transforms,
            &all_global_transforms,
            0,
            &to_check,
        ));
        // let mut children = all_children.get(root_entity);
        /*for child in children.iter() {
            // hierarchy_display
            let name = all_names.get(*child); //.unwrap_or(&Name::new("no name"));
            hierarchy_display.push(format!("  {:?}", name))
        }*/

        //
    }
    let display = display.single();
    *writer.text(display, 0) = hierarchy_display.join("\n");
}

////////:just some testing for gltf extras
#[allow(clippy::type_complexity)]
fn __check_for_gltf_extras(
    gltf_extras_per_entity: Query<(
        Entity,
        Option<&Name>,
        Option<&GltfSceneExtras>,
        Option<&GltfExtras>,
        Option<&GltfMeshExtras>,
        Option<&GltfMaterialExtras>,
    )>,
    display: Query<Entity, With<HiearchyDebugTag>>,
    mut writer: TextUiWriter,
) {
    let mut gltf_extra_infos_lines: Vec<String> = vec![];

    for (id, name, scene_extras, __extras, mesh_extras, material_extras) in
        gltf_extras_per_entity.iter()
    {
        if scene_extras.is_some()
        //|| extras.is_some()
        || mesh_extras.is_some()
        || material_extras.is_some()
        {
            let formatted_extras = format!(
                "Extras per entity {} ('Name: {}'):
- scene extras:     {:?}
- mesh extras:      {:?}
- material extras:  {:?}
            ",
                id,
                name.unwrap_or(&Name::default()),
                scene_extras,
                //extras,
                mesh_extras,
                material_extras
            );
            gltf_extra_infos_lines.push(formatted_extras);
        }
        let display = display.single();
        *writer.text(display, 0) = gltf_extra_infos_lines.join("\n");
    }
}

fn __check_for_component(
    specific_components: Query<(Entity, Option<&Name>, &RedirectPropHitImpulse)>,
    display: Query<Entity, With<HiearchyDebugTag>>,
    mut writer: TextUiWriter,
) {
    let mut info_lines: Vec<String> = vec![];
    for (__entiity, name, enum_complex) in specific_components.iter() {
        let data = format!(
            " We have a matching component: {:?} (on {:?})",
            enum_complex, name
        );
        info_lines.push(data);
        println!("yoho component");
    }

    let display = display.single();
    *writer.text(display, 0) = info_lines.join("\n");
}

pub struct HiearchyDebugPlugin;
impl Plugin for HiearchyDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_hierarchy_debug)
            //.add_systems(Update, check_for_component)
            .add_systems(Update, draw_hierarchy_debug)
            //.add_systems(Update, check_for_gltf_extras)
           ;
    }
}
