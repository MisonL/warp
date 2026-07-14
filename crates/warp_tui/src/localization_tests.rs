use super::*;

#[test]
fn environment_locale_candidates_prioritize_language_override() {
    let candidates = environment_locale_candidates_from(|key| match key {
        "LANGUAGE" => Some("zh_CN:fr_FR::en_US".to_owned()),
        "LC_ALL" => Some("de_DE.UTF-8".to_owned()),
        "LC_MESSAGES" => Some("it_IT.UTF-8".to_owned()),
        "LANG" => Some("en_GB.UTF-8".to_owned()),
        _ => None,
    });

    assert_eq!(
        candidates,
        vec![
            "zh_CN",
            "fr_FR",
            "en_US",
            "de_DE.UTF-8",
            "it_IT.UTF-8",
            "en_GB.UTF-8",
        ]
    );
}

#[test]
fn bundled_simplified_chinese_templates_are_available() {
    assert_eq!(
        &*CATALOGS.text(LocaleId::ZhCn, "tui.auth.sign_in"),
        "登录后继续"
    );
    assert_eq!(
        replace_placeholders(
            &CATALOGS.text(LocaleId::ZhCn, "tui.auth.open_browser"),
            &[("uri", "https://example.com")],
        )
        .expect("localized TUI test template arguments should match the catalog"),
        "在浏览器中打开 https://example.com"
    );
}
