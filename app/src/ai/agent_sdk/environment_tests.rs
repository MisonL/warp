use warp_cli::agent::OutputFormat;
use warp_localization::LocaleId;

use super::EnvironmentInfo;
use crate::ai::agent_sdk::common::{CANONICAL_UNKNOWN, CANONICAL_UNSYNCED};
use crate::ai::agent_sdk::output::write_list_for_locale;

#[test]
fn environment_list_localizes_text_without_localizing_json_sentinels() {
    let environment = || EnvironmentInfo {
        id: CANONICAL_UNSYNCED.to_owned(),
        name: "Development".to_owned(),
        description: None,
        base_image: None,
        github_repos: Vec::new(),
        setup_commands: Vec::new(),
        creator_email: CANONICAL_UNKNOWN.to_owned(),
        last_edited: "未知".to_owned(),
        last_edited_utc: None,
        scope: "Personal".to_owned(),
    };

    let mut text_output = Vec::new();
    write_list_for_locale(
        [environment()],
        OutputFormat::Text,
        &mut text_output,
        LocaleId::ZhCn,
    )
    .unwrap();
    let text_output = String::from_utf8(text_output).unwrap();
    assert!(text_output.contains("未同步"));
    assert!(text_output.contains("未知"));
    assert!(text_output.contains("个人"));

    let mut json_output = Vec::new();
    write_list_for_locale(
        [environment()],
        OutputFormat::Json,
        &mut json_output,
        LocaleId::ZhCn,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json_output).unwrap();
    assert_eq!(value[0]["id"], CANONICAL_UNSYNCED);
    assert_eq!(value[0]["creator_email"], CANONICAL_UNKNOWN);
    assert_eq!(value[0]["scope"], "Personal");
    assert!(value[0]["last_edited"].is_null());
}
