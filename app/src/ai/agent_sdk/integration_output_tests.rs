use warp_cli::agent::OutputFormat;
use warp_graphql::queries::get_simple_integrations::SimpleIntegrationConnectionStatus;
use warp_localization::LocaleId;

use super::{IntegrationInfo, canonical_status_explanation, empty_integrations_output};
use crate::ai::agent_sdk::output::write_list_for_locale;

#[test]
fn integration_list_localizes_text_without_localizing_json_status() {
    let integration = || IntegrationInfo {
        provider: "GitHub".to_owned(),
        description: "Source control".to_owned(),
        status: canonical_status_explanation(SimpleIntegrationConnectionStatus::NotConnected)
            .to_owned(),
        connection_status: SimpleIntegrationConnectionStatus::NotConnected,
        environment_uid: None,
        base_prompt: None,
        created_at: None,
        updated_at: None,
        created_at_formatted: "Unknown".to_owned(),
        updated_at_formatted: "Unknown".to_owned(),
    };

    let mut text_output = Vec::new();
    write_list_for_locale(
        [integration()],
        OutputFormat::Text,
        &mut text_output,
        LocaleId::ZhCn,
    )
    .unwrap();
    let text_output = String::from_utf8(text_output).unwrap();
    assert!(text_output.contains("此集成未连接"));

    let mut json_output = Vec::new();
    write_list_for_locale(
        [integration()],
        OutputFormat::Json,
        &mut json_output,
        LocaleId::ZhCn,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json_output).unwrap();
    assert_eq!(
        value[0]["status"],
        canonical_status_explanation(SimpleIntegrationConnectionStatus::NotConnected)
    );
}

#[test]
fn empty_integration_machine_output_remains_structured() {
    assert_eq!(
        empty_integrations_output(OutputFormat::Json, LocaleId::ZhCn).as_deref(),
        Some("[]")
    );
    assert_eq!(
        empty_integrations_output(OutputFormat::Ndjson, LocaleId::ZhCn),
        None
    );
    assert_eq!(
        empty_integrations_output(OutputFormat::Text, LocaleId::ZhCn).as_deref(),
        Some("未找到集成。")
    );
}
