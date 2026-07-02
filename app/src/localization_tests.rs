use super::*;

#[test]
fn environment_locale_candidates_reads_posix_locale_variables_in_order() {
    let candidates = environment_locale_candidates_from(|key| match key {
        "LANGUAGE" => Some("zh-CN:en-US".to_owned()),
        "LC_ALL" => Some("fr-FR".to_owned()),
        "LC_MESSAGES" => Some("de-DE".to_owned()),
        "LANG" => Some("es-ES.UTF-8".to_owned()),
        _ => None,
    });

    assert_eq!(
        candidates,
        ["zh-CN", "en-US", "fr-FR", "de-DE", "es-ES.UTF-8"]
    );
}

#[test]
fn environment_locale_candidates_ignores_empty_values() {
    let candidates = environment_locale_candidates_from(|key| match key {
        "LANGUAGE" => Some(" :zh-CN:: ".to_owned()),
        "LC_ALL" => Some(" ".to_owned()),
        "LC_MESSAGES" => None,
        "LANG" => Some("en-US".to_owned()),
        _ => None,
    });

    assert_eq!(candidates, ["zh-CN", "en-US"]);
}
