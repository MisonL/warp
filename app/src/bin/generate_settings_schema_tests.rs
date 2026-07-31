use std::collections::{BTreeMap, BTreeSet};

use super::*;

#[test]
fn parses_only_supported_schema_locales() {
    assert_eq!(parse_locale("en-US"), Ok(LocaleId::EnUs));
    assert_eq!(parse_locale("zh-CN"), Ok(LocaleId::ZhCn));
    assert!(parse_locale("zh").is_err());
}

#[test]
fn localized_root_metadata_uses_catalog_placeholders() {
    let catalogs = localization_catalogs();

    assert_eq!(root_title(&catalogs, LocaleId::ZhCn), "Warp 设置");
    assert_eq!(
        root_description(&catalogs, LocaleId::ZhCn, "stable", 42),
        "Warp 设置的 JSON Schema（stable 渠道，42 项设置）"
    );
}

#[test]
fn schema_description_localization_avoids_ambiguous_source_text() {
    let mut schema = serde_json::json!({
        "description": "shared source text",
        "title": "shared source text",
        "nested": {
            "description": "unique source text"
        }
    });
    let translations = HashMap::from([(
        "unique source text".to_string(),
        "唯一的中文文本".to_string(),
    )]);

    localize_schema_descriptions(&mut schema, &translations);

    assert_eq!(schema["description"], "shared source text");
    assert_eq!(schema["title"], "shared source text");
    assert_eq!(schema["nested"]["description"], "唯一的中文文本");
}

fn collect_descriptions<'a>(
    value: &'a Value,
    path: &mut Vec<String>,
    descriptions: &mut BTreeMap<String, &'a str>,
) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                path.push(key.clone());
                if key == "description"
                    && let Some(description) = value.as_str()
                {
                    descriptions.insert(path.join("."), description);
                }
                collect_descriptions(value, path, descriptions);
                path.pop();
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                path.push(index.to_string());
                collect_descriptions(value, path, descriptions);
                path.pop();
            }
        }
        _ => {}
    }
}

fn descriptions_by_path(value: &Value) -> BTreeMap<String, &str> {
    let mut descriptions = BTreeMap::new();
    collect_descriptions(value, &mut Vec::new(), &mut descriptions);
    descriptions
}

fn strip_localized_metadata(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("description");
            map.remove("title");
            for value in map.values_mut() {
                strip_localized_metadata(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                strip_localized_metadata(value);
            }
        }
        _ => {}
    }
}

#[test]
fn zh_cn_schema_localizes_every_description_except_proper_names() {
    let english = generate_settings_schema("stable", LocaleId::EnUs);
    let chinese = generate_settings_schema("stable", LocaleId::ZhCn);

    assert_eq!(english.entry_count, chinese.entry_count);
    let english_descriptions = descriptions_by_path(&english.value);
    let chinese_descriptions = descriptions_by_path(&chinese.value);
    assert_eq!(
        english_descriptions.keys().collect::<Vec<_>>(),
        chinese_descriptions.keys().collect::<Vec<_>>()
    );

    let untranslated_paths: BTreeSet<&str> = english_descriptions
        .iter()
        .filter_map(|(path, english)| {
            (chinese_descriptions.get(path) == Some(english)).then_some(path.as_str())
        })
        .collect();
    let proper_name_paths = BTreeSet::from([
        "$defs.AppIcon.oneOf.16.description",
        "$defs.GraphicsBackend.oneOf.1.description",
        "$defs.GraphicsBackend.oneOf.2.description",
        "$defs.GraphicsBackend.oneOf.3.description",
        "$defs.GraphicsBackend.oneOf.4.description",
        "$defs.ThemeKind.oneOf.0.description",
        "$defs.ThemeKind.oneOf.1.description",
        "$defs.ThemeKind.oneOf.3.description",
        "$defs.ThemeKind.oneOf.4.description",
        "$defs.ThemeKind.oneOf.5.description",
        "$defs.ThemeKind.oneOf.6.description",
        "$defs.ThemeKind.oneOf.7.description",
        "$defs.ThemeKind.oneOf.8.description",
        "$defs.ThemeKind.oneOf.10.description",
        "$defs.ThemeKind.oneOf.11.description",
        "$defs.ThemeKind.oneOf.12.description",
        "$defs.ThemeKind.oneOf.13.description",
        "$defs.ThemeKind.oneOf.14.description",
        "$defs.ThemeKind.oneOf.15.description",
        "$defs.ThemeKind.oneOf.16.description",
        "$defs.ThemeKind.oneOf.17.description",
        "$defs.ThemeKind.oneOf.18.description",
        "$defs.ThemeKind.oneOf.19.description",
        "$defs.ThemeKind.oneOf.20.description",
    ]);
    assert_eq!(untranslated_paths, proper_name_paths);

    let mut english_structure = english.value;
    let mut chinese_structure = chinese.value;
    strip_localized_metadata(&mut english_structure);
    strip_localized_metadata(&mut chinese_structure);
    assert_eq!(english_structure, chinese_structure);
}

#[test]
fn surface_annotation_matches_setting_schema_entry_metadata() {
    ensure_settings_linked();

    for entry in inventory::iter::<SettingSchemaEntry> {
        let surfaces = (entry.surfaces_fn)();
        let annotation = setting_surface_names(surfaces);
        let annotation_names: HashSet<&str> = annotation.iter().filter_map(Value::as_str).collect();

        assert_eq!(
            annotation_names.contains("gui"),
            surfaces.includes(SettingsMode::Gui),
            "GUI surface mismatch for {}",
            entry.storage_key
        );
        assert_eq!(
            annotation_names.contains("tui"),
            surfaces.includes(SettingsMode::Tui),
            "TUI surface mismatch for {}",
            entry.storage_key
        );
        assert_eq!(
            annotation_names.len(),
            usize::from(surfaces.includes(SettingsMode::Gui))
                + usize::from(surfaces.includes(SettingsMode::Tui)),
            "unexpected surface annotation for {}",
            entry.storage_key
        );
    }
}
