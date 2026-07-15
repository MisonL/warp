use warp_cli::agent::OutputFormat;
use warp_localization::LocaleId;

use super::{ProviderInfo, PROVIDER_STATUS_NOT_CONNECTED};
use crate::ai::agent_sdk::output::write_list_for_locale;

#[test]
fn provider_list_localizes_text_without_localizing_json_values() {
    let provider = || ProviderInfo {
        name: "Linear".to_owned(),
        slug: "linear".to_owned(),
        allowed_for: "personal, team".to_owned(),
        status: PROVIDER_STATUS_NOT_CONNECTED.to_owned(),
    };

    let mut text_output = Vec::new();
    write_list_for_locale(
        [provider()],
        OutputFormat::Text,
        &mut text_output,
        LocaleId::ZhCn,
    )
    .unwrap();
    let text_output = String::from_utf8(text_output).unwrap();
    assert!(text_output.contains("个人, 团队"));
    assert!(text_output.contains("未连接"));

    let mut json_output = Vec::new();
    write_list_for_locale(
        [provider()],
        OutputFormat::Json,
        &mut json_output,
        LocaleId::ZhCn,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json_output).unwrap();
    assert_eq!(value[0]["allowed_for"], "personal, team");
    assert_eq!(value[0]["status"], PROVIDER_STATUS_NOT_CONNECTED);
}
