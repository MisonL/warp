use warp_localization::LocaleId;

use super::{
    localized_auth_secret_row_label_for_locale, localized_auth_secret_status_message_for_locale,
};
use crate::ai::orchestration::OptionRow;

fn row(id: &str, label: &str) -> OptionRow {
    OptionRow {
        id: id.to_string(),
        label: label.to_string(),
        harness: None,
        badge: None,
        disabled_reason: None,
    }
}

#[test]
fn api_key_menu_localizes_inherit_without_changing_secret_names() {
    assert_eq!(
        localized_auth_secret_row_label_for_locale(&row("", "Skip (advanced)"), LocaleId::ZhCn,),
        "跳过（高级）",
    );
    assert_eq!(
        localized_auth_secret_row_label_for_locale(&row("team-key", "Team key"), LocaleId::ZhCn),
        "Team key",
    );
}

#[test]
fn api_key_menu_only_localizes_the_canonical_load_failure() {
    assert_eq!(
        localized_auth_secret_status_message_for_locale("Unable to load secrets", LocaleId::ZhCn),
        "无法加载密钥",
    );
    assert_eq!(
        localized_auth_secret_status_message_for_locale(
            "Server rejected the request",
            LocaleId::ZhCn,
        ),
        "Server rejected the request",
    );
}
