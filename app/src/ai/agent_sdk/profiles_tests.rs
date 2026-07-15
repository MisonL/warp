use warp_cli::agent::OutputFormat;
use warp_localization::LocaleId;

use super::{ProfileInfo, UNSYNCED_PROFILE_ID};
use crate::ai::agent_sdk::output::write_list_for_locale;

#[test]
fn profile_list_localizes_text_without_localizing_json_values() {
    let profile = || ProfileInfo {
        id: UNSYNCED_PROFILE_ID.to_owned(),
        name: "Default".to_owned(),
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
}
