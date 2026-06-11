#[test]
fn static_prompt_submission_uses_locale_independent_prompt() {
    warp::integration_testing::passive_suggestions::assert_static_prompt_submission_uses_locale_independent_prompt();
}

#[test]
fn static_prompt_without_label_uses_locale_independent_prompt() {
    warp::integration_testing::passive_suggestions::assert_static_prompt_without_label_uses_locale_independent_prompt();
}

#[test]
fn static_prompt_capture_replacement_is_single_pass() {
    warp::integration_testing::passive_suggestions::assert_static_prompt_capture_replacement_is_single_pass();
}

#[test]
fn zero_state_prompt_submission_uses_locale_independent_prompt() {
    warp::integration_testing::passive_suggestions::assert_zero_state_prompt_submission_uses_locale_independent_prompt();
}
