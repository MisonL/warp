use warp_cli::agent::OutputFormat;
use warp_localization::LocaleId;

use super::{ProfileInfo, ProfileNameFallback};
use crate::ai::agent_sdk::output::write_list_for_locale;

#[test]
fn profile_json_names_stay_locale_neutral_for_default_fallbacks() {
    let localized_default = crate::localization::text_for_locale(
        LocaleId::ZhCn,
        "settings.execution_profile.editor.default_profile_name",
    );
    let item = ProfileInfo {
        id: "profile-1".to_string(),
        name: "Default".to_string(),
        name_fallback: Some(ProfileNameFallback::Default),
    };

    let mut output = Vec::new();
    write_list_for_locale([item], OutputFormat::Json, &mut output, LocaleId::ZhCn).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(rendered, r#"[{"id":"profile-1","name":"Default"}]"#);
    assert!(!rendered.contains(&localized_default));
}

#[test]
fn profile_table_rows_localize_default_fallback_names() {
    let localized_default = crate::localization::text_for_locale(
        LocaleId::ZhCn,
        "settings.execution_profile.editor.default_profile_name",
    );
    let item = ProfileInfo {
        id: "profile-1".to_string(),
        name: "Default".to_string(),
        name_fallback: Some(ProfileNameFallback::Default),
    };

    let mut output = Vec::new();
    write_list_for_locale([item], OutputFormat::Text, &mut output, LocaleId::ZhCn).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains(&localized_default));
}

#[test]
fn profile_table_rows_localize_untitled_fallback_names() {
    let localized_untitled = crate::localization::text_for_locale(
        LocaleId::ZhCn,
        "settings.execution_profile.untitled_profile_name",
    );
    let item = ProfileInfo {
        id: "profile-1".to_string(),
        name: "Untitled".to_string(),
        name_fallback: Some(ProfileNameFallback::Untitled),
    };

    let mut output = Vec::new();
    write_list_for_locale([item], OutputFormat::Text, &mut output, LocaleId::ZhCn).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains(&localized_untitled));
}
