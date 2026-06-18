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
            .unwrap_or_else(|| message.to_owned())
    }
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
    if let ConversationStatus::Blocked { blocked_action } = conversation.status() {
        return Some(AmbientConversationStatus::Blocked {
            blocked_action: blocked_action.clone(),
        });
    }
    if let ConversationStatus::Error = conversation.status() {
        if let Some(error_message) = conversation.status_error_message() {
            return Some(AmbientConversationStatus::Error {
                error: RenderableAIError::Other {
                    error_message: error_message.to_string(),
                    will_attempt_resume: false,
                    waiting_for_network: false,
                },
            });
        }
    }

    if let Some(last_exchange) = conversation.root_task_exchanges().last() {
        if let AIAgentOutputStatus::Finished { finished_output } = &last_exchange.output_status {
            let status = match finished_output {
                FinishedAIAgentOutput::Cancelled { output: _, reason } => {
                    AmbientConversationStatus::Cancelled { reason: *reason }
                }
                FinishedAIAgentOutput::Error { output: _, error } => {
                    AmbientConversationStatus::Error {
                        error: error.clone(),
                    }
                }
                FinishedAIAgentOutput::Success { output: _ } => AmbientConversationStatus::Success,
            };
            return Some(status);
        }
    }

    None
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
