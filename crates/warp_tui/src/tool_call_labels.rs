//! Per-tool, per-state one-line labels for tool-call rows in the TUI
//! transcript, modeled on the GUI's inline action text.

use std::path::Path;

use ai::agent::action_result::RunAgentsAgentOutcome;
use warp::tui_export::{
    AIActionStatus, AIAgentAction, AIAgentActionResultType, AIAgentActionType,
    AskUserQuestionResult, FileGlobV2Result, GrepResult, RequestCommandOutputResult,
    RunAgentsAgentOutcomeKind, RunAgentsResult, SearchCodebaseFailureReason, SearchCodebaseResult,
    StartAgentExecutionMode, SuggestNewConversationResult,
};
use warp_core::command::ExitCode;
use warpui_core::elements::tui::TuiStyle;

use self::ToolCallDisplayState as State;
#[path = "tool_call_labels_actions.rs"]
mod actions;
use crate::localization;
use crate::tui_builder::TuiUiBuilder;

/// Ground-truth state of the terminal block backing a shell-command tool
/// call, resolved by the caller. When a block exists, its state supersedes
/// the stored action status/result for execution states (mirroring the GUI's
/// `RequestedCommandView`, which derives icon and expandability from the
/// block whenever one exists). Notably, an agent-monitored command's stored
/// result stays a `LongRunningCommandSnapshot` forever, so without the block
/// its row could never leave the "still running" state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommandBlockState {
    Running,
    Finished { exit_code: ExitCode },
}

/// A shell-command tool call's terminal block as resolved by the caller: its
/// execution state plus the command it actually ran. The block's command
/// supersedes the streamed one, which the user may have edited before
/// accepting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedCommandBlock {
    /// The block's command, when it has one; `None` while the block's
    /// command grid is still empty.
    pub(crate) command: Option<String>,
    pub(crate) state: CommandBlockState,
}

/// Longest rendered length for interpolated values (commands, queries, paths)
/// so tool-call rows stay scannable one-liners.
const MAX_INLINE_LEN: usize = 80;

/// Coarse presentation state for a tool call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ToolCallDisplayState {
    /// The tool call's arguments are still streaming and may be incomplete.
    Constructing,
    /// The tool call is waiting to begin execution.
    Pending,
    /// The tool call is blocked on user confirmation.
    Blocked,
    /// The tool call is executing asynchronously.
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl ToolCallDisplayState {
    /// The compact leading glyph for this state.
    pub(crate) fn glyph(self) -> &'static str {
        match self {
            Self::Constructing | Self::Pending => "○",
            Self::Blocked | Self::Cancelled => "■",
            Self::Running => "●",
            Self::Succeeded => "✓",
            Self::Failed => "×",
        }
    }

    /// The semantic theme style for this state's glyph.
    pub(crate) fn glyph_style(self, builder: &TuiUiBuilder) -> TuiStyle {
        match self {
            Self::Constructing | Self::Pending => builder.dim_text_style(),
            Self::Blocked | Self::Running => builder.attention_glyph_style(),
            Self::Succeeded => builder.success_glyph_style(),
            Self::Failed => builder.error_text_style(),
            Self::Cancelled => builder.muted_text_style(),
        }
    }

    /// The semantic text style paired with this state.
    pub(crate) fn label_style(self, builder: &TuiUiBuilder) -> TuiStyle {
        match self {
            Self::Constructing | Self::Pending => builder.dim_text_style(),
            Self::Blocked | Self::Running | Self::Succeeded | Self::Failed | Self::Cancelled => {
                builder.primary_text_style()
            }
        }
    }
}

/// Collapses an optional action status into the coarse display state.
/// `output_streaming` is whether the exchange output is still streaming;
/// a status-less action in a streaming output is still being constructed
/// (mirroring the GUI's `status.is_none() && is_streaming()` gating).
/// A resolved `block_state` supersedes the status for execution states
/// (see [`CommandBlockState`]).
pub(crate) fn tool_call_display_state(
    status: Option<&AIActionStatus>,
    output_streaming: bool,
    block_state: Option<CommandBlockState>,
) -> ToolCallDisplayState {
    // A block existing means the command actually started executing, so its
    // state is authoritative over the action status/result.
    match block_state {
        Some(CommandBlockState::Running) => return State::Running,
        Some(CommandBlockState::Finished { exit_code }) => {
            return if exit_code.is_sigint() {
                State::Cancelled
            } else if exit_code.was_successful() {
                State::Succeeded
            } else {
                State::Failed
            };
        }
        None => {}
    }

    match status {
        None if output_streaming => State::Constructing,
        None | Some(AIActionStatus::Preprocessing | AIActionStatus::Queued) => State::Pending,
        Some(AIActionStatus::Blocked) => State::Blocked,
        Some(AIActionStatus::RunningAsync) => State::Running,
        Some(finished @ AIActionStatus::Finished(_)) => {
            if finished.is_cancelled() {
                State::Cancelled
            } else if finished.is_failed() {
                State::Failed
            } else {
                State::Succeeded
            }
        }
    }
}

/// Returns the one-line transcript label for a tool call in its current state.
pub(crate) fn tool_call_label(
    action: &AIAgentAction,
    status: Option<&AIActionStatus>,
    output_streaming: bool,
    block: Option<&ResolvedCommandBlock>,
) -> String {
    let state = tool_call_display_state(status, output_streaming, block.map(|block| block.state));
    let result = status
        .and_then(AIActionStatus::finished_result)
        .map(|result| &result.result);
    let label = actions::label_for_action(&action.action, state, result, block);

    match state {
        State::Blocked => {
            localization::text_with_args("tui.tool.awaiting_approval", &[("label", &label)])
        }
        State::Constructing
        | State::Pending
        | State::Running
        | State::Succeeded
        | State::Failed
        | State::Cancelled => label,
    }
}

pub(crate) fn blocked_tool_call_label(action: &AIAgentActionType) -> String {
    actions::label_for_action(action, State::Blocked, None, None)
}

/// Summarizes the outcomes of an orchestration launch using the same catalog
/// entries as the rest of the TUI tool-call labels.
fn launched_agents_label(agents: &[RunAgentsAgentOutcome]) -> String {
    let launched = agents
        .iter()
        .filter(|agent| matches!(agent.kind, RunAgentsAgentOutcomeKind::Launched { .. }))
        .count();
    let total = agents.len();
    let agents = localized_count_label(total, "tui.count.agent.one", "tui.count.agent.many");

    if launched == total {
        localization::text_with_args("tui.tool.orchestration.spawned", &[("agents", &agents)])
    } else if launched == 0 {
        localization::text_with_args(
            "tui.tool.orchestration.spawn_failed",
            &[("agents", &agents)],
        )
    } else {
        localization::text_with_args(
            "tui.tool.orchestration.spawned_some",
            &[
                ("launched", &launched.to_string()),
                ("total", &total.to_string()),
            ],
        )
    }
}

/// Shared label body for both file-glob action versions; only V2 results
/// carry a match count.
fn file_glob_label(
    patterns: &[String],
    path: Option<&str>,
    state: ToolCallDisplayState,
    matched_count: Option<usize>,
) -> String {
    let patterns = single_line(&patterns.join(", "));
    let path = display_path(path.unwrap_or("."));

    match state {
        State::Constructing => localization::text("tui.tool.file_glob.preparing"),
        State::Pending | State::Blocked => localization::text_with_args(
            "tui.tool.file_glob.start",
            &[("patterns", &patterns), ("path", &path)],
        ),
        State::Running => localization::text_with_args(
            "tui.tool.file_glob.running",
            &[("patterns", &patterns), ("path", &path)],
        ),
        State::Succeeded => match matched_count {
            Some(count) => {
                let files =
                    localized_count_label(count, "tui.count.file.one", "tui.count.file.many");
                localization::text_with_args(
                    "tui.tool.file_glob.succeeded_with_count",
                    &[("files", &files), ("patterns", &patterns)],
                )
            }
            None => localization::text_with_args(
                "tui.tool.file_glob.succeeded",
                &[("patterns", &patterns)],
            ),
        },
        State::Failed => {
            localization::text_with_args("tui.tool.file_glob.failed", &[("patterns", &patterns)])
        }
        State::Cancelled => {
            localization::text_with_args("tui.tool.file_glob.cancelled", &[("patterns", &patterns)])
        }
    }
}

fn command_exit_label(command: &str, exit_code: ExitCode) -> String {
    let exit_code = exit_code.value().to_string();
    localization::text_with_args(
        "tui.tool.command.exited",
        &[("command", command), ("exit_code", &exit_code)],
    )
}

/// Labels computer-use calls with their agent-supplied summary, marking only
/// terminal non-success states (matching the GUI, which shows the summary
/// verbatim).
fn summary_label(summary: &str, state: ToolCallDisplayState) -> String {
    let summary = single_line(summary);
    match state {
        State::Constructing => localization::text("tui.tool.computer.preparing"),
        State::Pending | State::Blocked | State::Running | State::Succeeded => summary,
        State::Failed => {
            localization::text_with_args("tui.tool.generic.failed", &[("label", &summary)])
        }
        State::Cancelled => {
            localization::text_with_args("tui.tool.generic.cancelled", &[("label", &summary)])
        }
    }
}

/// Generic label for action types without bespoke text, derived from the
/// action's user-friendly name.
fn fallback_label(name: String, state: ToolCallDisplayState) -> String {
    match state {
        State::Pending | State::Blocked => name,
        State::Constructing | State::Running => {
            localization::text_with_args("tui.tool.generic.running", &[("label", &name)])
        }
        State::Succeeded => {
            localization::text_with_args("tui.tool.generic.done", &[("label", &name)])
        }
        State::Failed => {
            localization::text_with_args("tui.tool.generic.failed", &[("label", &name)])
        }
        State::Cancelled => {
            localization::text_with_args("tui.tool.generic.cancelled", &[("label", &name)])
        }
    }
}

/// Collapses text to its first line, capped at [`MAX_INLINE_LEN`] chars, with
/// a trailing `…` when anything was trimmed.
fn single_line(text: &str) -> String {
    let first_line = text.lines().next().unwrap_or_default().trim_end();
    let mut out: String = first_line.chars().take(MAX_INLINE_LEN).collect();
    if first_line.chars().count() > MAX_INLINE_LEN || text.lines().count() > 1 {
        out.push('…');
    }
    out
}

/// Renders a search path for display, mirroring the GUI's treatment of `.`.
fn display_path(path: &str) -> String {
    if path == "." {
        localization::text("tui.tool.path.current_directory")
    } else {
        single_line(path)
    }
}

/// Returns the final path component, falling back to the input when there is none.
fn base_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_owned())
}

/// Summarizes file paths as comma-joined base names for up to 3 files, else a count.
fn files_summary<'a>(paths: impl ExactSizeIterator<Item = &'a String>) -> String {
    if paths.len() > 3 {
        return localized_count_label(paths.len(), "tui.count.file.one", "tui.count.file.many");
    }
    let names: Vec<String> = paths.map(|path| base_name(path)).collect();
    if names.is_empty() {
        localization::text("tui.tool.files.generic")
    } else {
        names.join(", ")
    }
}

fn localized_count_label(count: usize, singular_key: &str, plural_key: &str) -> String {
    localization::text_with_args(
        if count == 1 { singular_key } else { plural_key },
        &[("count", &count.to_string())],
    )
}

#[cfg(test)]
#[path = "tool_call_labels_tests.rs"]
mod tests;
