use warp_graphql::ai::{AgentTaskState, PlatformErrorCode};
use warp_localization::LocaleId;

use super::AgentDriverError;
use super::terminal::{BootstrapError, ShareSessionError};
use crate::ai::blocklist::local_agent_task_sync_model::classify_renderable_error;
use crate::localization;
use crate::server::server_api::ai::TaskStatusUpdate;

const KEY_PREFIX: &str = "agent_sdk.driver.error_classification";

fn text(locale: LocaleId, key_suffix: &str) -> String {
    let key = format!("{KEY_PREFIX}.{key_suffix}");
    localization::text_for_locale(locale, &key)
}

fn text_with_args(locale: LocaleId, key_suffix: &str, args: &[(&str, &str)]) -> String {
    let key = format!("{KEY_PREFIX}.{key_suffix}");
    localization::text_for_locale_with_args(locale, &key, args)
}

fn bootstrap_error_text(error: &BootstrapError, locale: LocaleId) -> String {
    match error {
        BootstrapError::PtySpawnFailed {
            reason: Some(reason),
        } => localization::text_for_locale_with_args(
            locale,
            "agent_sdk.driver.error_classification.bootstrap_error.pty_spawn_failed_with_reason",
            &[("reason", reason)],
        ),
        BootstrapError::PtySpawnFailed { reason: None } => localization::text_for_locale(
            locale,
            "agent_sdk.driver.error_classification.bootstrap_error.pty_spawn_failed",
        ),
        BootstrapError::TimedOut => localization::text_for_locale(
            locale,
            "agent_sdk.driver.error_classification.bootstrap_error.timed_out",
        ),
        BootstrapError::InternalError => localization::text_for_locale(
            locale,
            "agent_sdk.driver.error_classification.bootstrap_error.internal",
        ),
    }
}

/// Classify an `AgentDriverError` into a task state and a `TaskStatusUpdate`
/// suitable for reporting via `update_agent_task`.
fn classify_driver_error_for_locale(
    error: &AgentDriverError,
    locale: LocaleId,
) -> (AgentTaskState, TaskStatusUpdate) {
    match error {
        // Warp-side errors (task -> ERROR).
        AgentDriverError::TerminalUnavailable | AgentDriverError::InvalidRuntimeState => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                text(locale, "internal_error"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::BootstrapFailed { error } => {
            let error = bootstrap_error_text(error, locale);
            (
                AgentTaskState::Error,
                TaskStatusUpdate::with_error_code(
                    text_with_args(locale, "bootstrap_failed", &[("error", &error)]),
                    PlatformErrorCode::InternalError,
                ),
            )
        }
        AgentDriverError::ShareSessionFailed { error: share_err } => {
            let message = match share_err {
                ShareSessionError::Internal(_) => text(locale, "share_internal"),
                ShareSessionError::Failed(reason) => {
                    // The reason string comes from the session-sharing layer and is aimed at
                    // interactive users (e.g. "try sharing again"). Provide a cloud-agent-
                    // appropriate message instead of wrapping it, which would produce
                    // repetitive "try again" text.
                    text_with_args(locale, "share_failed", &[("reason", reason)])
                }
                ShareSessionError::Disabled => text(locale, "share_disabled"),
                ShareSessionError::Timeout => text(locale, "share_timeout"),
                ShareSessionError::Interrupted => text(locale, "share_interrupted"),
            };
            (
                AgentTaskState::Error,
                TaskStatusUpdate::with_error_code(
                    message,
                    match share_err {
                        ShareSessionError::Disabled => PlatformErrorCode::FeatureNotAvailable,
                        _ => PlatformErrorCode::InternalError,
                    },
                ),
            )
        }
        AgentDriverError::WarpDriveSyncFailed => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                text(locale, "warp_drive_sync_failed"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::NotLoggedIn => {
            let bin = warp_cli::binary_name().unwrap_or_else(|| "warp".to_string());
            (
                AgentTaskState::Error,
                TaskStatusUpdate::with_error_code(
                    text_with_args(locale, "not_logged_in", &[("bin", &bin)]),
                    PlatformErrorCode::AuthenticationRequired,
                ),
            )
        }
        AgentDriverError::CloudProviderSetupFailed(err) => {
            let error = format!("{err:#}");
            (
                AgentTaskState::Error,
                TaskStatusUpdate::with_error_code(
                    text_with_args(locale, "cloud_provider_setup_failed", &[("error", &error)]),
                    PlatformErrorCode::InternalError,
                ),
            )
        }

        // User-side errors (task -> FAILED).
        AgentDriverError::MCPServerNotFound(uuid) => {
            let uuid = uuid.to_string();
            (
                AgentTaskState::Failed,
                TaskStatusUpdate::with_error_code(
                    text_with_args(locale, "mcp_server_not_found", &[("uuid", &uuid)]),
                    PlatformErrorCode::EnvironmentSetupFailed,
                ),
            )
        }
        AgentDriverError::ManagedMcpResolutionFailed { uid, message } => {
            let uid = uid.to_string();
            (
                AgentTaskState::Failed,
                TaskStatusUpdate::with_error_code(
                    text_with_args(
                        locale,
                        "managed_mcp_resolution_failed",
                        &[("uid", &uid), ("message", message)],
                    ),
                    PlatformErrorCode::EnvironmentSetupFailed,
                ),
            )
        }
        AgentDriverError::MCPStartupFailed { details } => {
            let server_lines = details
                .iter()
                .map(|detail| format!("- {detail}"))
                .collect::<Vec<_>>()
                .join("\n");
            (
                AgentTaskState::Failed,
                TaskStatusUpdate::with_error_code(
                    text_with_args(
                        locale,
                        "mcp_startup_failed",
                        &[("server_lines", &server_lines)],
                    ),
                    PlatformErrorCode::EnvironmentSetupFailed,
                ),
            )
        }
        AgentDriverError::MCPJsonParseError(msg) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(locale, "mcp_json_parse_error", &[("message", msg)]),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::MCPMissingVariables => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text(locale, "mcp_missing_variables"),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::ProfileError(name) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(locale, "profile_not_found", &[("name", name)]),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::AIWorkflowNotFound(id) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(locale, "saved_prompt_not_found", &[("id", id)]),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::EnvironmentNotFound(id) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(locale, "environment_not_found", &[("id", id)]),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::EnvironmentSetupFailed(msg) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(locale, "environment_setup_failed", &[("message", msg)]),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        // The shell died while an environment setup command was running
        // (e.g. the command ran `exit`). This is a user-side environment
        // configuration problem, so classify as FAILED.
        AgentDriverError::SetupCommandExitedShell { .. } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                error.to_string(),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::InvalidWorkingDirectory { path, .. } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "Working directory '{}' does not exist or is not a directory. Verify the path in your environment configuration.",
                    path.display()
                ),
            )
        }

        // Conversation errors.
        // Delegate to classify_renderable_error for proper ERROR vs FAILED
        // distinction and PlatformErrorCode. This is a belt-and-suspenders
        // fallback - LocalAgentTaskSyncModel handles most conversation errors,
        // but the driver catches them too if the conversation ends with an error.
        AgentDriverError::ConversationError { error } => {
            let (state, update) = classify_renderable_error(error);
            (
                state,
                update.unwrap_or_else(|| {
                    TaskStatusUpdate::with_error_code(
                        error.to_string(),
                        PlatformErrorCode::InternalError,
                    )
                }),
            )
        }

        // Cancellation / Blocked (no error code).
        AgentDriverError::ConversationCancelled { .. } => (
            AgentTaskState::Cancelled,
            TaskStatusUpdate::message(text(locale, "conversation_cancelled")),
        ),
        AgentDriverError::ConversationBlocked { blocked_action } => (
            AgentTaskState::Blocked,
            TaskStatusUpdate::message(text_with_args(
                locale,
                "conversation_blocked",
                &[("blocked_action", blocked_action)],
            )),
        ),

        // Setup errors.
        AgentDriverError::TeamMetadataRefreshTimeout => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                text(locale, "team_metadata_refresh_timeout"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::SkillResolutionFailed(msg) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(locale, "skill_resolution_failed", &[("message", msg)]),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::ConfigBuildFailed(err) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!("Failed to build agent configuration: {err}"),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::PromptResolutionFailed(err) => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                format!("Failed to resolve prompt for the run: {err}"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::SecretsFetchFailed(err) => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                format!("Failed to fetch task secrets: {err}"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::TaskMetadataFetchFailed(err) => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                format!("Failed to fetch task metadata: {err}"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::AwsBedrockCredentialsFailed(msg) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(
                    locale,
                    "aws_bedrock_credentials_failed",
                    &[("message", msg)],
                ),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::ConversationLoadFailed(msg) => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                text_with_args(locale, "conversation_load_failed", &[("message", msg)]),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::ConversationHarnessMismatch {
            conversation_id,
            expected,
            got,
        } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(
                    locale,
                    "conversation_harness_mismatch",
                    &[
                        ("conversation_id", conversation_id),
                        ("expected", expected),
                        ("got", got),
                    ],
                ),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::TaskHarnessMismatch {
            task_id,
            expected,
            got,
        } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(
                    locale,
                    "task_harness_mismatch",
                    &[("task_id", task_id), ("expected", expected), ("got", got)],
                ),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::ConversationResumeStateMissing {
            harness,
            conversation_id,
        } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(
                    locale,
                    "conversation_resume_state_missing",
                    &[("conversation_id", conversation_id), ("harness", harness)],
                ),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::HarnessCommandFailed { exit_code } => {
            let exit_code = exit_code.to_string();
            (
                AgentTaskState::Failed,
                TaskStatusUpdate::with_error_code(
                    text_with_args(
                        locale,
                        "harness_command_failed",
                        &[("exit_code", &exit_code)],
                    ),
                    PlatformErrorCode::InternalError,
                ),
            )
        }
        AgentDriverError::HarnessSetupFailed { harness, reason } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                text_with_args(
                    locale,
                    "harness_setup_failed",
                    &[("harness", harness), ("reason", reason)],
                ),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::HarnessConfigSetupFailed { harness, error } => {
            let error = error.to_string();
            (
                AgentTaskState::Failed,
                TaskStatusUpdate::with_error_code(
                    text_with_args(
                        locale,
                        "harness_config_setup_failed",
                        &[("harness", harness), ("error", &error)],
                    ),
                    PlatformErrorCode::EnvironmentSetupFailed,
                ),
            )
        }
        AgentDriverError::HarnessAuthCheckFailed { harness, detail } => {
            log::error!("Preflight detail for {harness}: {detail}");
            (
                AgentTaskState::Failed,
                TaskStatusUpdate::with_error_code(
                    text_with_args(locale, "harness_auth_check_failed", &[("harness", harness)]),
                    PlatformErrorCode::AuthenticationRequired,
                ),
            )
        }
        AgentDriverError::HarnessRuntimeFailureDetected {
            harness,
            pattern,
            excerpt,
        } => {
            log::error!("Runtime failure for {harness}: pattern={pattern}, excerpt={excerpt}");
            (
                AgentTaskState::Failed,
                TaskStatusUpdate::with_error_code(
                    text_with_args(
                        locale,
                        "harness_runtime_failure_detected",
                        &[
                            ("harness", harness),
                            ("pattern", pattern),
                            ("excerpt", excerpt),
                        ],
                    ),
                    PlatformErrorCode::AuthenticationRequired,
                ),
            )
        }
    }
}

/// Classify an error for persistence in an agent task status.
///
/// Persisted messages use the canonical English catalog so clients can localize them when
/// rendering the task in a different locale.
pub fn classify_driver_error(error: &AgentDriverError) -> (AgentTaskState, TaskStatusUpdate) {
    classify_driver_error_for_locale(error, LocaleId::EnUs)
}

pub(super) fn localized_driver_error_message(error: &AgentDriverError, locale: LocaleId) -> String {
    classify_driver_error_for_locale(error, locale).1.message
}

#[cfg(test)]
#[path = "error_classification_tests.rs"]
mod tests;
