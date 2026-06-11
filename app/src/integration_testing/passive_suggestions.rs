use warp_localization::LocaleId;

use crate::ai::blocklist::{
    apply_static_prompt_captures_for_integration_test, static_suggested_query_for_integration_test,
};
use crate::localization;
use crate::terminal::view::inline_banner::ZeroStatePromptSuggestionType;

pub fn assert_static_prompt_submission_uses_locale_independent_prompt() {
    let repository = "warp-terminal";
    let suggestion =
        static_suggested_query_for_integration_test("git clone warp-terminal", LocaleId::ZhCn)
            .expect("git clone should match a static prompt suggestion");

    let expected_label = localization::text_for_locale(
        LocaleId::ZhCn,
        "terminal.passive_suggestion.static.git_clone.label",
    )
    .replace("{1}", repository);
    let expected_prompt = localization::text_for_locale(
        LocaleId::EnUs,
        "terminal.passive_suggestion.static.git_clone.query",
    )
    .replace("{1}", repository);

    assert_eq!(suggestion.label.as_deref(), Some(expected_label.as_str()));
    assert_eq!(suggestion.prompt, expected_prompt);
}

pub fn assert_static_prompt_without_label_uses_locale_independent_prompt() {
    let suggestion = static_suggested_query_for_integration_test("git push", LocaleId::ZhCn)
        .expect("git push should match a static prompt suggestion");
    let expected_prompt = localization::text_for_locale(
        LocaleId::EnUs,
        "terminal.passive_suggestion.static.git_push.query",
    );

    assert_eq!(suggestion.label, None);
    assert_eq!(suggestion.prompt, expected_prompt);
}

pub fn assert_static_prompt_capture_replacement_is_single_pass() {
    let regex = regex::Regex::new(r"^cmd\s+(\S+)\s+(\S+)$").unwrap();
    let captures = regex
        .captures("cmd alpha{2} beta")
        .expect("command should match capture test regex");

    let result = apply_static_prompt_captures_for_integration_test("{0}: {1} then {2}", &captures);

    assert_eq!(result, "{0}: alpha{2} then beta");
}

pub fn assert_zero_state_prompt_submission_uses_locale_independent_prompt() {
    let suggestion = ZeroStatePromptSuggestionType::Fix;
    let key = suggestion.query_key();

    assert_eq!(
        suggestion.prompt(),
        localization::text_for_locale(LocaleId::EnUs, key)
    );
}
