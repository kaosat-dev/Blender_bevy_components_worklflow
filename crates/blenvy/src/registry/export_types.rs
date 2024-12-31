use crate::{AssetRoot, BlenvyConfig};
use bevy::{
    log::info,
    prelude::{AppTypeRegistry, ReflectComponent, ReflectResource, World},
    reflect::{TypeInfo, TypeRegistration, VariantInfo},
};
use serde_json::{json, Map, Value};
use std::{fs::File, path::Path};

pub fn export_types(world: &mut World) {
    let config = world
        .get_resource::<BlenvyConfig>()
        .expect("ExportComponentsConfig should exist at this stage");

    let asset_root = world.resource::<AssetRoot>();
    let registry_save_path = Path::join(&asset_root.0, &config.registry_save_path);
    let writer = File::create(registry_save_path).expect("should have created schema file");

    let components_to_filter_out = &config.registry_component_filter.clone();
    let resources_to_filter_out = &config.registry_resource_filter.clone();

    let types = world.resource_mut::<AppTypeRegistry>();
    let types = types.read();
    let schemas = types
        .iter()
        .filter(|type_info| {
            let type_id = type_info.type_id();
            components_to_filter_out.is_allowed_by_id(type_id)
                && resources_to_filter_out.is_allowed_by_id(type_id)
        })
        .map(export_type)
        .collect::<Map<_, _>>();

    serde_json::to_writer_pretty(
        writer,
        &json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "long_name": "bevy component registry schema",
            "$defs": schemas,
        }),
    )
    .expect("valid json");

    info!("Done exporting registry schema");
}

pub fn export_type(reg: &TypeRegistration) -> (String, Value) {
    let t = reg.type_info();
    let binding = t.type_path_table();
    let short_name = binding.short_path();
    let mut schema = match t {
        TypeInfo::Struct(info) => {
            let properties = info
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    (
                        field.name().to_owned(),
                        add_min_max(json!({ "type": typ(field.type_path()) }), reg, idx, None),
                    )
                })
                .collect::<Map<_, _>>();

            json!({
                "type": "object",
                "typeInfo": "Struct",
                "long_name": t.type_path(),
                "properties": properties,
                "additionalProperties": false,
                "required": info
                    .iter()
                    .filter(|field| !field.type_path().starts_with("core::option::Option"))
                    .map(|field| field.name())
                    .collect::<Vec<_>>(),
            })
        }
        TypeInfo::Enum(info) => {
            let simple = info
                .iter()
                .all(|variant| matches!(variant, VariantInfo::Unit(_)));
            if simple {
                json!({
                    "type": "string",
                    "typeInfo": "Enum",
                    "long_name": t.type_path(),
                    "oneOf": info
                        .iter()
                        .map(|variant| match variant {
                            VariantInfo::Unit(v) => v.name(),
                            _ => unreachable!(),
                        })
                        .collect::<Vec<_>>(),
                })
            } else {
                let variants = info
                .iter()
                .enumerate()
                .map(|(field_idx, variant)| match variant {
                    //let binding = t.type_path_table();
                    //let short_name = binding.short_path();
                    VariantInfo::Struct(v) => json!({
                        "type": "object",
                        "typeInfo": "Struct",
                        "long_name": v.name(),
                        "short_name": v.name().split("::").last().unwrap_or(v.name()),
                        "properties": v
                            .iter()
                            .enumerate()
                            .map(|(variant_idx, field)| (field.name().to_owned(), add_min_max(json!({"type": typ(field.type_path()), "long_name": field.name()}), reg, field_idx, Some(variant_idx))))
                            .collect::<Map<_, _>>(),
                        "additionalProperties": false,
                        "required": v
                            .iter()
                            .filter(|field| !field.type_path().starts_with("core::option::Option"))
                            .map(|field| field.name())
                            .collect::<Vec<_>>(),
                    }),
                    VariantInfo::Tuple(v) => json!({
                        "type": "array",
                        "typeInfo": "Tuple",
                        "long_name": v.name(),
                        "short_name":v.name(),
                        "prefixItems": v
                            .iter()
                            .enumerate()
                            .map(|(variant_idx, field)| add_min_max(json!({"type": typ(field.type_path())}), reg, field_idx, Some(variant_idx)))
                            .collect::<Vec<_>>(),
                        "items": false,
                    }),
                    VariantInfo::Unit(v) => json!({
                        "long_name": v.name(),
                    }),
                })
                .collect::<Vec<_>>();

                json!({
                    "type": "object",
                    "typeInfo": "Enum",
                    "long_name": t.type_path(),
                    "oneOf": variants,
                })
            }
        }
        TypeInfo::TupleStruct(info) => json!({
            "long_name": t.type_path(),
            "type": "array",
            "typeInfo": "TupleStruct",
            "prefixItems": info
                .iter()
                .enumerate()
                .map(|(idx, field)| add_min_max(json!({"type": typ(field.type_path())}), reg, idx, None))
                .collect::<Vec<_>>(),
            "items": false,
        }),
        TypeInfo::List(info) => json!({
            "long_name": t.type_path(),
            "type": "array",
            "typeInfo": "List",
            "items": json!({
                "type": typ(info.item_ty().path())
            }),
        }),
        TypeInfo::Array(info) => json!({
            "long_name": t.type_path(),
            "type": "array",
            "typeInfo": "Array",
            "items": json!({"type": typ(info.item_ty().path())}),
        }),
        TypeInfo::Map(info) => json!({
            "long_name": t.type_path(),
            "type": "object",
            "typeInfo": "Map",
            "valueType": json!({"type": typ(info.value_ty().path())}),
            "keyType": json!({"type": typ(info.key_ty().path())}),
        }),
        TypeInfo::Tuple(info) => json!({
            "long_name": t.type_path(),
            "type": "array",
            "typeInfo": "Tuple",
            "prefixItems": info
                .iter()
                .enumerate()
                .map(|(idx, field)| add_min_max(json!({"type": typ(field.type_path())}), reg, idx, None))
                .collect::<Vec<_>>(),
            "items": false,
        }),
        TypeInfo::Set(info) => json!({
            "long_name": t.type_path(),
            "type": "set",
            "typeInfo": "Set",
            "items": json!({"type": typ(info.value_ty().path())}),
        }),
        TypeInfo::Opaque(info) => json!({
            "long_name": t.type_path(),
            "type": map_json_type(info.type_path()),
            "typeInfo": "Value",
        }),
    };
    schema.as_object_mut().unwrap().insert(
        "isComponent".to_owned(),
        reg.data::<ReflectComponent>().is_some().into(),
    );
    schema.as_object_mut().unwrap().insert(
        "isResource".to_owned(),
        reg.data::<ReflectResource>().is_some().into(),
    );

    schema
        .as_object_mut()
        .unwrap()
        .insert("short_name".to_owned(), short_name.into());

    (t.type_path().to_owned(), schema)
}

fn typ(t: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{t}") })
}

fn map_json_type(t: &str) -> Value {
    match t {
        "bool" => "boolean",
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => "uint",
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => "int",
        "f32" | "f64" => "float",
        "char" | "str" | "alloc::string::String" => "string",
        _ => "object",
    }
    .into()
}

fn add_min_max(
    mut val: Value,
    reg: &TypeRegistration,
    field_index: usize,
    variant_index: Option<usize>,
) -> Value {
    /*fn get_min_max(
        reg: &TypeRegistration,
        field_index: usize,
        variant_index: Option<usize>,
    ) -> Option<(Option<f32>, Option<f32>)> {
        use bevy_inspector_egui::inspector_options::{
            std_options::NumberOptions, ReflectInspectorOptions, Target,
        };

        reg.data::<ReflectInspectorOptions>()
            .and_then(|ReflectInspectorOptions(o)| {
                o.get(if let Some(variant_index) = variant_index {
                    Target::VariantField {
                        variant_index,
                        field_index,
                    }
                } else {
                    Target::Field(field_index)
                })
            })
            .and_then(|o| o.downcast_ref::<NumberOptions<f32>>())
            .map(|num| (num.min, num.max))
    }*/

    fn get_min_max(
        _reg: &TypeRegistration,
        _field_index: usize,
        _variant_index: Option<usize>,
    ) -> Option<(Option<f32>, Option<f32>)> {
        None
    }

    let Some((min, max)) = get_min_max(reg, field_index, variant_index) else {
        return val;
    };
    let obj = val.as_object_mut().unwrap();
    if let Some(min) = min {
        obj.insert("minimum".to_owned(), min.into());
    }
    if let Some(max) = max {
        obj.insert("maximum".to_owned(), max.into());
    }
    val
}
