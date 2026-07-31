use warp_cli::agent::OutputFormat;
use warp_localization::LocaleId;

use super::{ProfileInfo, UNSYNCED_PROFILE_ID};
use crate::ai::agent_sdk::output::write_list_for_locale;

#[test]
fn profile_list_localizes_text_without_localizing_json_values() {
    let profile = || ProfileInfo {
        id: UNSYNCED_PROFILE_ID.to_owned(),
        name: "Default".to_owned(),
        name_key: Some("settings.execution_profile.editor.default_profile_name"),
    };

    let mut text_output = Vec::new();
    write_list_for_locale(
        [profile()],
        OutputFormat::Text,
        &mut text_output,
        LocaleId::ZhCn,
    )
    .unwrap();
    let text_output = String::from_utf8(text_output).unwrap();
    assert!(text_output.contains("未同步"));
    assert!(text_output.contains("默认"));

    let mut json_output = Vec::new();
    write_list_for_locale(
        [profile()],
        OutputFormat::Json,
        &mut json_output,
        LocaleId::ZhCn,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json_output).unwrap();
    assert_eq!(value[0]["id"], UNSYNCED_PROFILE_ID);
    assert_eq!(value[0]["name"], "Default");
}

#[test]
fn profile_list_does_not_translate_user_defined_names() {
    let profile = ProfileInfo {
        id: "profile-id".to_owned(),
        name: "Default".to_owned(),
        name_key: None,
    };

    let mut output = Vec::new();
    write_list_for_locale([profile], OutputFormat::Text, &mut output, LocaleId::ZhCn).unwrap();

    assert!(String::from_utf8(output).unwrap().contains("Default"));
}
