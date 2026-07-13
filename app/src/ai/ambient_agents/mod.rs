use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::{NonNilUuid, Uuid};
use warp_localization::LocaleId;
use warpui::AppContext;

use crate::ai::agent::conversation::{AIConversation, ConversationStatus};
use crate::ai::agent::{
    AIAgentOutputStatus, CancellationReason, FinishedAIAgentOutput, RenderableAIError,
};
use crate::localization;
use crate::terminal::cli_agent_sessions::CLI_AGENT_WAITING_FOR_ANSWER_BLOCKED_ACTION;

pub mod github_auth_notifier;
pub mod github_auth_url;
pub mod scheduled;
pub mod spawn;
pub mod task;
pub mod telemetry;

pub use task::{
    cancel_task_silently, cancel_task_with_toast, AgentConfigSnapshot, AgentSource,
    AmbientAgentLiveSessionState, AmbientAgentTask, AmbientAgentTaskState, TaskStatusMessage,
};
pub const OUT_OF_CREDITS_TASK_FAILURE_MESSAGE: &str =
    "Out of credits. Upgrade your Warp plan to continue running cloud agents.";
pub const SERVER_OVERLOADED_TASK_FAILURE_MESSAGE: &str =
    "Warp is temporarily overloaded. Please try again shortly.";
const LOCALIZABLE_TASK_STATUS_KEYS: &[&str] = &[
    "agent.task_status.quota_limit",
    "agent.task_status.server_overloaded",
    "agent.task_status.internal_error",
];
const LOCALIZABLE_TASK_STATUS_PATTERNS: &[(&str, &str)] = &[
    ("agent.task_status.blocked", "action"),
    ("agent.task_status.context_window_exceeded", "message"),
    ("agent.task_status.invalid_api_key", "provider"),
    (
        "agent.task_status.aws_bedrock_credentials_expired_or_invalid",
        "model_name",
    ),
];
const LOCALIZABLE_DRIVER_ERROR_KEYS: &[&str] = &[
    "agent_sdk.driver.error_classification.aws_bedrock_credentials_failed",
    "agent_sdk.driver.error_classification.bootstrap_error.internal",
    "agent_sdk.driver.error_classification.bootstrap_error.pty_spawn_failed",
    "agent_sdk.driver.error_classification.bootstrap_error.pty_spawn_failed_with_reason",
    "agent_sdk.driver.error_classification.bootstrap_error.timed_out",
    "agent_sdk.driver.error_classification.bootstrap_failed",
    "agent_sdk.driver.error_classification.cloud_provider_setup_failed",
    "agent_sdk.driver.error_classification.config_build_failed",
    "agent_sdk.driver.error_classification.conversation_blocked",
    "agent_sdk.driver.error_classification.conversation_cancelled",
    "agent_sdk.driver.error_classification.conversation_harness_mismatch",
    "agent_sdk.driver.error_classification.conversation_load_failed",
    "agent_sdk.driver.error_classification.conversation_resume_state_missing",
    "agent_sdk.driver.error_classification.environment_not_found",
    "agent_sdk.driver.error_classification.environment_setup_failed",
    "agent_sdk.driver.error_classification.harness_auth_check_failed",
    "agent_sdk.driver.error_classification.harness_command_failed",
    "agent_sdk.driver.error_classification.harness_config_setup_failed",
    "agent_sdk.driver.error_classification.harness_runtime_failure_detected",
    "agent_sdk.driver.error_classification.harness_setup_failed",
    "agent_sdk.driver.error_classification.internal_error",
    "agent_sdk.driver.error_classification.invalid_working_directory",
    "agent_sdk.driver.error_classification.managed_mcp_resolution_failed",
    "agent_sdk.driver.error_classification.mcp_json_parse_error",
    "agent_sdk.driver.error_classification.mcp_missing_variables",
    "agent_sdk.driver.error_classification.mcp_server_not_found",
    "agent_sdk.driver.error_classification.mcp_startup_failed",
    "agent_sdk.driver.error_classification.not_logged_in",
    "agent_sdk.driver.error_classification.profile_not_found",
    "agent_sdk.driver.error_classification.prompt_resolution_failed",
    "agent_sdk.driver.error_classification.saved_prompt_not_found",
    "agent_sdk.driver.error_classification.secrets_fetch_failed",
    "agent_sdk.driver.error_classification.share_disabled",
    "agent_sdk.driver.error_classification.share_failed",
    "agent_sdk.driver.error_classification.share_internal",
    "agent_sdk.driver.error_classification.share_interrupted",
    "agent_sdk.driver.error_classification.share_timeout",
    "agent_sdk.driver.error_classification.skill_resolution_failed",
    "agent_sdk.driver.error_classification.task_harness_mismatch",
    "agent_sdk.driver.error_classification.team_metadata_refresh_timeout",
    "agent_sdk.driver.error_classification.warp_drive_sync_failed",
];
const LOCALIZABLE_AGENT_ACTION_NAME_KEYS: &[&str] = &[
    "agent.action.name.ask_user_question",
    "agent.action.name.call_mcp_tool",
    "agent.action.name.create_documents",
    "agent.action.name.edit_documents",
    "agent.action.name.edit_files",
    "agent.action.name.fetch_conversation",
    "agent.action.name.file_glob",
    "agent.action.name.grep",
    "agent.action.name.init_project",
    "agent.action.name.insert_code_review_comments",
    "agent.action.name.open_code_review",
    "agent.action.name.read_documents",
    "agent.action.name.read_files",
    "agent.action.name.read_mcp_resource",
    "agent.action.name.read_shell_command_output",
    "agent.action.name.read_skill",
    "agent.action.name.request_computer_use",
    "agent.action.name.run_agents",
    "agent.action.name.run_command",
    "agent.action.name.search_codebase",
    "agent.action.name.send_message_to_agent",
    "agent.action.name.start_agent",
    "agent.action.name.suggest_new_conversation",
    "agent.action.name.suggest_prompt",
    "agent.action.name.transfer_shell_command_control_to_user",
    "agent.action.name.upload_artifact",
    "agent.action.name.use_computer",
    "agent.action.name.write_to_long_running_shell_command",
];
const CANONICAL_AGENT_ACTION_NAME_ALIASES: &[(&str, &str)] = &[
    (
        CLI_AGENT_WAITING_FOR_ANSWER_BLOCKED_ACTION,
        "agent.action.name.waiting_for_your_answer",
    ),
    ("Call mcp tool", "agent.action.name.call_mcp_tool"),
    ("Read mcp resource", "agent.action.name.read_mcp_resource"),
    (
        "Write to long running shell command",
        "agent.action.name.write_to_long_running_shell_command",
    ),
];

pub fn localized_task_status_message(app: &AppContext, message: &str) -> String {
    localized_task_status_message_for_locale(localization::current_locale(app), message)
}

pub fn localized_task_status_message_for_locale(locale: LocaleId, message: &str) -> String {
    if message == canonical_task_status_message("agent.task_status.error") {
        localization::text_for_locale(locale, "agent.task_status.error")
    } else if message == canonical_task_status_message("agent.task_status.cancelled") {
        localization::text_for_locale(locale, "agent.task_status.cancelled")
    } else {
        localized_canonical_task_status_message(locale, message)
            .or_else(|| localized_canonical_driver_error_message(locale, message))
            .unwrap_or_else(|| message.to_owned())
    }
}

fn localized_canonical_driver_error_message(locale: LocaleId, message: &str) -> Option<String> {
    LOCALIZABLE_DRIVER_ERROR_KEYS.iter().find_map(|key| {
        let canonical = localization::text_for_locale(LocaleId::EnUs, key);
        if canonical == message {
            return Some(localization::text_for_locale(locale, key));
        }

        let args = extract_template_args(&canonical, message)?;
        let localized_args = args
            .into_iter()
            .map(|(name, value)| {
                let value = match (*key, name.as_str()) {
                    ("agent_sdk.driver.error_classification.bootstrap_failed", "error") => {
                        localized_canonical_driver_error_message(locale, &value).unwrap_or(value)
                    }
                    _ => value,
                };
                (name, value)
            })
            .collect::<Vec<_>>();
        let arg_refs = localized_args
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect::<Vec<_>>();
        Some(localization::text_for_locale_with_args(
            locale, key, &arg_refs,
        ))
    })
}

fn extract_template_args(pattern: &str, message: &str) -> Option<Vec<(String, String)>> {
    let mut args = Vec::<(String, String)>::new();
    let mut pattern_cursor = 0;
    let mut message_cursor = 0;

    while let Some(relative_start) = pattern[pattern_cursor..].find('{') {
        let placeholder_start = pattern_cursor + relative_start;
        let literal = &pattern[pattern_cursor..placeholder_start];
        let remaining_message = message.get(message_cursor..)?;
        if !remaining_message.starts_with(literal) {
            return None;
        }
        message_cursor += literal.len();

        let placeholder_end = pattern[placeholder_start..].find('}')? + placeholder_start;
        let name = &pattern[placeholder_start + 1..placeholder_end];
        let remaining_pattern = &pattern[placeholder_end + 1..];
        let next_placeholder = remaining_pattern
            .find('{')
            .unwrap_or(remaining_pattern.len());
        let next_literal = &remaining_pattern[..next_placeholder];
        let remaining_message = message.get(message_cursor..)?;
        let value_len = if next_literal.is_empty() {
            if next_placeholder < remaining_pattern.len() {
                return None;
            }
            remaining_message.len()
        } else {
            remaining_message.find(next_literal)?
        };
        let value = remaining_message[..value_len].to_owned();

        if let Some((_, existing)) = args.iter().find(|(existing_name, _)| existing_name == name) {
            if existing != &value {
                return None;
            }
        } else {
            args.push((name.to_owned(), value));
        }
        message_cursor += value_len;
        pattern_cursor = placeholder_end + 1;
    }

    (pattern[pattern_cursor..] == message[message_cursor..]).then_some(args)
}

fn localized_canonical_task_status_message(locale: LocaleId, message: &str) -> Option<String> {
    LOCALIZABLE_TASK_STATUS_KEYS
        .iter()
        .find(|key| localization::text_for_locale(LocaleId::EnUs, key) == message)
        .map(|key| localization::text_for_locale(locale, key))
        .or_else(|| localized_canonical_task_status_message_with_arg(locale, message))
}

fn localized_canonical_task_status_message_with_arg(
    locale: LocaleId,
    message: &str,
) -> Option<String> {
    LOCALIZABLE_TASK_STATUS_PATTERNS
        .iter()
        .find_map(|(key, arg_name)| {
            let pattern = localization::text_for_locale(LocaleId::EnUs, key);
            let arg_value = extract_single_placeholder_value(&pattern, message)?;
            let localized_arg_value = localized_task_status_arg(locale, key, arg_name, arg_value);
            Some(localization::text_for_locale_with_args(
                locale,
                key,
                &[(arg_name, localized_arg_value.as_str())],
            ))
        })
}

fn canonical_task_status_message(key: &str) -> String {
    localization::text_for_locale(LocaleId::EnUs, key)
}

fn localized_task_status_arg(
    locale: LocaleId,
    status_key: &str,
    arg_name: &str,
    arg_value: &str,
) -> String {
    match (status_key, arg_name) {
        ("agent.task_status.blocked", "action") => {
            localized_canonical_agent_action_name(locale, arg_value)
                .unwrap_or_else(|| arg_value.to_owned())
        }
        _ => arg_value.to_owned(),
    }
}

fn localized_canonical_agent_action_name(locale: LocaleId, action_name: &str) -> Option<String> {
    if let Some((_, key)) = CANONICAL_AGENT_ACTION_NAME_ALIASES
        .iter()
        .find(|(canonical_action_name, _)| *canonical_action_name == action_name)
    {
        return Some(localization::text_for_locale(locale, key));
    }

    LOCALIZABLE_AGENT_ACTION_NAME_KEYS
        .iter()
        .find_map(|key| localized_canonical_text_for_key(locale, key, action_name))
}

fn localized_canonical_text_for_key(
    locale: LocaleId,
    key: &str,
    canonical_text: &str,
) -> Option<String> {
    let pattern = localization::text_for_locale(LocaleId::EnUs, key);
    if pattern == canonical_text {
        return Some(localization::text_for_locale(locale, key));
    }

    let arg_value = extract_single_placeholder_value(&pattern, canonical_text)?;
    let arg_name = single_placeholder_name(&pattern)?;
    Some(localization::text_for_locale_with_args(
        locale,
        key,
        &[(arg_name, arg_value)],
    ))
}

fn single_placeholder_name(pattern: &str) -> Option<&str> {
    let placeholder_start = pattern.find('{')?;
    let placeholder_end = pattern[placeholder_start..].find('}')? + placeholder_start;
    Some(&pattern[placeholder_start + 1..placeholder_end])
}

fn extract_single_placeholder_value<'a>(pattern: &str, message: &'a str) -> Option<&'a str> {
    let placeholder_start = pattern.find('{')?;
    let placeholder_end = pattern[placeholder_start..].find('}')? + placeholder_start;
    let prefix = &pattern[..placeholder_start];
    let suffix = &pattern[placeholder_end + 1..];

    let value = message.strip_prefix(prefix)?.strip_suffix(suffix)?;
    (!value.is_empty()).then_some(value)
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid task ID: {0}")]
pub struct ParseAmbientAgentTaskIdError(#[from] uuid::Error);

/// A globally unique ID for an ambient agent task.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AmbientAgentTaskId(NonNilUuid);

impl Display for AmbientAgentTaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for AmbientAgentTaskId {
    type Err = ParseAmbientAgentTaskIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::try_parse(s)?;
        Ok(Self(NonNilUuid::try_from(uuid)?))
    }
}

impl From<AmbientAgentTaskId> for cynic::Id {
    fn from(id: AmbientAgentTaskId) -> Self {
        Self::new(id.to_string())
    }
}

/// High-level outcome of an ambient agent conversation.
#[derive(Clone, Debug)]
pub enum AmbientConversationStatus {
    Success,
    Error {
        error: RenderableAIError,
    },
    #[allow(dead_code)]
    Cancelled {
        reason: CancellationReason,
    },
    #[allow(dead_code)]
    Blocked {
        blocked_action: String,
    },
}

/// Derive an [`AmbientConversationStatus`] from the given conversation, if it has
/// reached a terminal state that we care about for ambient agents.
pub fn conversation_output_status_from_conversation(
    conversation: &AIConversation,
) -> Option<AmbientConversationStatus> {
    match conversation.status() {
        // A pending recovery is not a terminal outcome.
        ConversationStatus::TransientError => None,

        ConversationStatus::Blocked { blocked_action } => {
            Some(AmbientConversationStatus::Blocked {
                blocked_action: blocked_action.clone(),
            })
        }

        ConversationStatus::Error => {
            // Prefer the structured error on the last exchange: it carries the precise
            // error variant and rendering hints that the string-only `status_error_message`
            // cannot.
            if let Some(AIAgentOutputStatus::Finished {
                finished_output: FinishedAIAgentOutput::Error { error, .. },
            }) = conversation
                .root_task_exchanges()
                .last()
                .map(|exchange| &exchange.output_status)
            {
                return Some(AmbientConversationStatus::Error {
                    error: error.clone(),
                });
            }
            if let Some(error_message) = conversation.status_error_message() {
                return Some(AmbientConversationStatus::Error {
                    error: RenderableAIError::Other {
                        error_message: error_message.to_string(),
                        will_attempt_resume: false,
                        waiting_for_network: false,
                        is_user_error: false,
                    },
                });
            }
            // Neither a structured exchange error nor a status message is available;
            // fall back to whatever terminal outcome the last exchange carries.
            terminal_status_from_last_exchange(conversation)
        }

        // `InProgress` and `WaitingForEvents` are not terminal, but we preserve the
        // existing behavior of reporting a terminal outcome whenever the last exchange
        // has already finished.
        ConversationStatus::InProgress
        | ConversationStatus::Success
        | ConversationStatus::Cancelled
        | ConversationStatus::WaitingForEvents => terminal_status_from_last_exchange(conversation),
    }
}

/// Derive a terminal [`AmbientConversationStatus`] from the conversation's last
/// exchange, if that exchange has finished.
fn terminal_status_from_last_exchange(
    conversation: &AIConversation,
) -> Option<AmbientConversationStatus> {
    let AIAgentOutputStatus::Finished { finished_output } =
        &conversation.root_task_exchanges().last()?.output_status
    else {
        return None;
    };
    Some(match finished_output {
        FinishedAIAgentOutput::Cancelled { reason, .. } => {
            AmbientConversationStatus::Cancelled { reason: *reason }
        }
        FinishedAIAgentOutput::Error { error, .. } => AmbientConversationStatus::Error {
            error: error.clone(),
        },
        FinishedAIAgentOutput::Success { .. } => AmbientConversationStatus::Success,
    })
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
