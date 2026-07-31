use std::time::Duration;

use aws_credential_types::provider::error::CredentialsError;
use warp_localization::LocaleId;

use super::{OidcRefreshError, user_facing_aws_credentials_error_message};

#[test]
fn maps_credentials_not_loaded_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::not_loaded_no_source(),
        "sandbox",
        LocaleId::EnUs,
    );

    assert_eq!(
        message,
        "AWS credentials were not found for the AWS profile `sandbox`. Log in with the AWS CLI or update your AWS credentials configuration, then refresh."
    );
}

#[test]
fn maps_invalid_configuration_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::invalid_configuration(std::io::Error::other("bad config")),
        "readonly",
        LocaleId::EnUs,
    );

    assert_eq!(
        message,
        "The AWS profile `readonly` is invalid or incomplete in your local AWS configuration. Update your AWS profile settings and credentials, then refresh."
    );
}

#[test]
fn maps_provider_timeout_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::provider_timed_out(Duration::from_secs(5)),
        "sandbox",
        LocaleId::EnUs,
    );

    assert_eq!(
        message,
        "Timed out while loading AWS credentials. Refresh and try again."
    );
}

#[test]
fn maps_provider_error_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::provider_error(std::io::Error::other("provider error")),
        "sandbox",
        LocaleId::EnUs,
    );

    assert_eq!(
        message,
        "Unable to load AWS credentials from your configured provider. Refresh your AWS login and try again."
    );
}

#[test]
fn maps_unhandled_error_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::unhandled(std::io::Error::other("unexpected")),
        "sandbox",
        LocaleId::EnUs,
    );

    assert_eq!(
        message,
        "Unexpected error while loading AWS credentials. Refresh your AWS login and try again."
    );
}

#[test]
fn maps_invalid_configuration_to_requested_locale() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::invalid_configuration(std::io::Error::other("bad config")),
        "readonly",
        LocaleId::ZhCn,
    );

    assert!(message.contains("AWS profile `readonly`"));
    assert!(message.contains("无效或不完整"));
    assert!(!message.contains("invalid or incomplete"));
}

#[test]
fn oidc_errors_localize_user_messages_without_localizing_diagnostics() {
    let errors = [
        OidcRefreshError::TaskIdRequired,
        OidcRefreshError::MintToken(anyhow::anyhow!("token source")),
        OidcRefreshError::AssumeRole {
            detail: "access denied".to_string(),
            source: anyhow::anyhow!("sts source"),
        },
        OidcRefreshError::MissingCredentials,
        OidcRefreshError::RefreshInterrupted,
    ];

    let expected_diagnostics = [
        "AWS Bedrock inference requires an ambient task ID before credentials can be minted",
        "Failed to mint AWS Bedrock task identity token",
        "STS AssumeRoleWithWebIdentity failed: access denied",
        "STS response did not include credentials",
        "Credential refresh was interrupted",
    ];
    let expected_chinese = [
        "AWS Bedrock 推理需要 Ambient 任务 ID 后才能签发凭证",
        "签发 AWS Bedrock 任务身份令牌失败",
        "STS AssumeRoleWithWebIdentity 失败：access denied",
        "STS 响应未包含凭证",
        "凭证刷新已中断",
    ];

    for ((error, diagnostic), user_message) in errors
        .into_iter()
        .zip(expected_diagnostics)
        .zip(expected_chinese)
    {
        assert_eq!(error.to_string(), diagnostic);
        assert_eq!(error.user_message(LocaleId::ZhCn), user_message);
    }
}
