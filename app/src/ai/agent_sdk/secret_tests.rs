use chrono::{TimeZone as _, Utc};
use serde_json::Value;
use warp_cli::agent::OutputFormat;
use warp_graphql::managed_secrets::ManagedSecretType;
use warp_localization::LocaleId;

use super::SecretInfo;
use crate::ai::agent_sdk::output::write_list_for_locale;

fn secret_info() -> SecretInfo {
    SecretInfo {
        name: "deploy-token".to_string(),
        scope: super::super::common::OWNER_SCOPE_TEAM.to_string(),
        secret_type: ManagedSecretType::RawValue,
        created_at: Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 1, 3, 4, 5, 6).unwrap(),
    }
}

#[test]
fn secret_list_json_keeps_scope_locale_neutral() {
    let localized_team =
        crate::localization::text_for_locale(LocaleId::ZhCn, "agent_sdk.common.owner.team");
    let mut output = Vec::new();

    write_list_for_locale(
        [secret_info()],
        OutputFormat::Json,
        &mut output,
        LocaleId::ZhCn,
    )
    .unwrap();

    let rendered = String::from_utf8(output).unwrap();
    let value: Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(
        value[0]["scope"].as_str(),
        Some(super::super::common::OWNER_SCOPE_TEAM)
    );
    assert!(!rendered.contains(&localized_team));
}

#[test]
fn secret_list_ndjson_keeps_scope_locale_neutral() {
    let localized_team =
        crate::localization::text_for_locale(LocaleId::ZhCn, "agent_sdk.common.owner.team");
    let mut output = Vec::new();

    write_list_for_locale(
        [secret_info()],
        OutputFormat::Ndjson,
        &mut output,
        LocaleId::ZhCn,
    )
    .unwrap();

    let rendered = String::from_utf8(output).unwrap();
    let value: Value = serde_json::from_str(rendered.trim_end()).unwrap();
    assert_eq!(
        value["scope"].as_str(),
        Some(super::super::common::OWNER_SCOPE_TEAM)
    );
    assert!(!rendered.contains(&localized_team));
}

#[test]
fn secret_list_text_localizes_scope_rows() {
    let localized_team =
        crate::localization::text_for_locale(LocaleId::ZhCn, "agent_sdk.common.owner.team");
    let mut output = Vec::new();

    write_list_for_locale(
        [secret_info()],
        OutputFormat::Text,
        &mut output,
        LocaleId::ZhCn,
    )
    .unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains(&localized_team));
}
