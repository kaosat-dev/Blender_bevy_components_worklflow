use bevy::log::{debug, warn};
use bevy::reflect::serde::ReflectDeserializer;
use bevy::reflect::{PartialReflect, TypeRegistration, TypeRegistry};
use bevy::utils::HashMap;
use ron::Value;
use serde::de::DeserializeSeed;

use super::capitalize_first_letter;

pub fn ronstring_to_reflect_component(
    ron_string: &str,
    type_registry: &TypeRegistry,
) -> Vec<(Box<dyn PartialReflect>, TypeRegistration)> {
    let lookup: HashMap<String, Value> = ron::from_str(ron_string).unwrap();
    let mut components: Vec<(Box<dyn PartialReflect>, TypeRegistration)> = Vec::new();
    // println!("ron_string {:?}", ron_string);
    for (name, value) in lookup.into_iter() {
        let parsed_value: String = match value.clone() {
            Value::String(str) => str,
            _ => ron::to_string(&value).unwrap().to_string(),
        };

        if name.as_str() == "bevy_components" {
            bevy_components_string_to_components(parsed_value, type_registry, &mut components);
        } else {
            components_string_to_components(
                name,
                value,
                parsed_value,
                type_registry,
                &mut components,
            );
        }
    }
    components
}

fn components_string_to_components(
    name: String,
    value: Value,
    parsed_value: String,
    type_registry: &TypeRegistry,
    components: &mut Vec<(Box<dyn PartialReflect>, TypeRegistration)>,
) {
    let type_string = name.replace("component: ", "").trim().to_string();
    let capitalized_type_name = capitalize_first_letter(type_string.as_str());

    if let Some(type_registration) =
        type_registry.get_with_short_type_path(capitalized_type_name.as_str())
    {
        debug!("TYPE INFO {:?}", type_registration.type_info());

        let ron_string = format!(
            "{{ \"{}\":{} }}",
            type_registration.type_info().type_path(),
            parsed_value
        );

        /*
        // usefull to determine what an entity looks like Serialized
        let test_struct =  Color::Srgba(Srgba { red: 0.2, green: 0.2, blue: 0.2, alpha: 0.2 });
        //CameraRenderGraph::new("name");
        let serializer = ReflectSerializer::new(&test_struct, &type_registry);
        let serialized =
            ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default()).unwrap();
        println!("serialized Component {}", serialized);
        */
        debug!("component data ron string {}", ron_string);
        let mut deserializer = ron::Deserializer::from_str(ron_string.as_str())
            .expect("deserialzer should have been generated from string");
        let reflect_deserializer = ReflectDeserializer::new(type_registry);
        /*let component = reflect_deserializer
        .deserialize(&mut deserializer)
        .unwrap_or_else(|_| {
            panic!(
                "failed to deserialize component {} with value: {:?}",
                name, value
            )
        });*/
        let Ok(component) = reflect_deserializer.deserialize(&mut deserializer) else {
            warn!(
                "failed to deserialize component {} with value: {:?}",
                name, value
            );
            return;
        };

        debug!("component {:?}", component);
        debug!("real type {:?}", component.get_represented_type_info());
        components.push((component, type_registration.clone()));
        debug!("found type registration for {}", capitalized_type_name);
    } else {
        warn!("no type registration for {}", capitalized_type_name);
    }
}

fn bevy_components_string_to_components(
    parsed_value: String,
    type_registry: &TypeRegistry,
    components: &mut Vec<(Box<dyn PartialReflect>, TypeRegistration)>,
) {
    let lookup: HashMap<String, Value> = ron::from_str(&parsed_value).unwrap();
    for (key, value) in lookup.into_iter() {
        let parsed_value: String = match value.clone() {
            Value::String(str) => str,
            _ => ron::to_string(&value).unwrap().to_string(),
        };

        if let Some(type_registration) = type_registry.get_with_type_path(key.as_str()) {
            debug!("TYPE INFO {:?}", type_registration.type_info());

            let ron_string = format!(
                "{{ \"{}\":{} }}",
                type_registration.type_info().type_path(),
                parsed_value
            );

            debug!("component data ron string {}", ron_string);
            let mut deserializer = ron::Deserializer::from_str(ron_string.as_str())
                .expect("deserialzer should have been generated from string");
            let reflect_deserializer = ReflectDeserializer::new(type_registry);
            let component = reflect_deserializer
                .deserialize(&mut deserializer)
                .unwrap_or_else(|_| {
                    panic!(
                        "failed to deserialize component {} with value: {:?}",
                        key, value
                    )
                });

            debug!("component {:?}", component);
            debug!("real type {:?}", component.get_represented_type_info());
            components.push((component, type_registration.clone()));
            debug!("found type registration for {}", key);
        } else {
            warn!("no type registration for {}", key);
        }
    }
}
