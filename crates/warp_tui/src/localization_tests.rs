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
fn replacing_current_locale_reports_only_changes() {
    let locale = RwLock::new(LocaleId::EnUs);

    assert!(!replace_current_locale(&locale, LocaleId::EnUs));
    assert!(replace_current_locale(&locale, LocaleId::ZhCn));
    assert_eq!(*locale.read(), LocaleId::ZhCn);
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
    for (key, expected) in [
        ("tui.markdown.empty_table", "[空表格]"),
        ("tui.markdown.table_has_no_rows", "[表格没有行]"),
        (
            "tui.markdown.unsupported_embedded_content",
            "[不支持的嵌入内容]",
        ),
    ] {
        assert_eq!(text_for_locale(LocaleId::ZhCn, key), expected);
    }
}
