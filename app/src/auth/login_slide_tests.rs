use super::LoginPurpose;

#[test]
fn login_copy_uses_localization_keys() {
    assert_eq!(
        LoginPurpose::AccountFirst.copy(),
        (
            "auth.onboarding.title.account_first",
            "auth.onboarding.subtitle.account_first",
        )
    );
    assert_eq!(
        LoginPurpose::AccountFirst.work_email_callout_copy(),
        Some((
            "auth.onboarding.work_email.title",
            "auth.onboarding.work_email.description",
        ))
    );
}
