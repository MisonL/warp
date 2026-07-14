//! Per-tool, per-state one-line labels for tool-call rows in the TUI
//! transcript, modeled on the GUI's inline action text.

use std::path::Path;

use warp::tui_export::{
    AIActionStatus, AIAgentAction, AIAgentActionResultType, AIAgentActionType,
    AskUserQuestionResult, FileGlobV2Result, GrepResult, RequestCommandOutputResult,
    RunAgentsAgentOutcomeKind, RunAgentsResult, SearchCodebaseFailureReason, SearchCodebaseResult,
    StartAgentExecutionMode, SuggestNewConversationResult,
};
use warp_core::command::ExitCode;

use self::ToolCallDisplayState as State;
use crate::localization;

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

/// The coarse display state of a tool call, derived from its action status.
///
/// TUI-local presentation collapse of the shared [`AIActionStatus`]; the GUI
/// has no equivalent enum — its per-tool views consume `AIActionStatus`
/// directly and re-derive per-site booleans (queued/cancelled/streaming).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ToolCallDisplayState {
    /// The tool call's arguments are still streaming in: it has no action
    /// status yet and the exchange output is still streaming, so argument
    /// fields may be empty or partial and must not be interpolated.
    Constructing,
    /// No status yet (stream finished), preprocessing, or queued behind
    /// other actions.
    Pending,
    /// Blocked on user confirmation.
    AwaitingApproval,
    /// Executing asynchronously.
    Running,
    Succeeded,
    Failed,
    Cancelled,
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
        Some(AIActionStatus::Blocked) => State::AwaitingApproval,
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

/// The leading status glyph for a tool-call row; the caller colors it to
/// mirror the GUI's inline action icons (`action_icon` in the GUI's
/// `output.rs`): grey circle while pending, yellow block awaiting approval,
/// yellow dot running, green check on success, red x on failure, grey block
/// on cancellation.
pub(crate) fn tool_call_glyph(state: ToolCallDisplayState) -> &'static str {
    match state {
        State::Constructing | State::Pending => "○",
        State::AwaitingApproval | State::Cancelled => "■",
        State::Running => "●",
        State::Succeeded => "✓",
        State::Failed => "✗",
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
    let label = label_for_action(&action.action, state, result, block);
    match state {
        State::AwaitingApproval => {
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

/// Builds the per-tool label body; the awaiting-approval suffix is applied by
/// [`tool_call_label`]. `result` is the finished result, when there is one.
///
/// `Constructing` arms never interpolate argument fields (they may be empty
/// or partial while streaming); their copy is indexed on the GUI's loading
/// messages (`common.rs` `LOAD_OUTPUT_MESSAGE_*` and the requested-command
/// view's "Generating command...").
fn label_for_action(
    action: &AIAgentActionType,
    state: ToolCallDisplayState,
    result: Option<&AIAgentActionResultType>,
    block: Option<&ResolvedCommandBlock>,
) -> String {
    let block_state = block.map(|block| block.state);
    match action {
        AIAgentActionType::RequestCommandOutput { command, .. } => {
            // The streamed command can be edited before acceptance, so
            // prefer the executed command from the finished result or the
            // resolved block over the original suggestion.
            let executed = result
                .and_then(AIAgentActionResultType::command_str)
                .or_else(|| block.and_then(|block| block.command.as_deref()));
            let cmd = single_line(executed.unwrap_or(command));
            match state {
                State::Constructing => localization::text("tui.tool.command.generating"),
                State::Pending | State::AwaitingApproval => {
                    localization::text_with_args("tui.tool.command.run", &[("command", &cmd)])
                }
                State::Running => {
                    localization::text_with_args("tui.tool.command.running", &[("command", &cmd)])
                }
                State::Succeeded => match block_state {
                    Some(CommandBlockState::Finished { .. }) => {
                        localization::text_with_args("tui.tool.command.ran", &[("command", &cmd)])
                    }
                    // No local block: fall back to the stored result. A
                    // snapshot result means the command was still running at
                    // the last point we could observe it.
                    Some(CommandBlockState::Running) | None => match result {
                        Some(AIAgentActionResultType::RequestCommandOutput(
                            RequestCommandOutputResult::LongRunningCommandSnapshot { .. },
                        )) => localization::text_with_args(
                            "tui.tool.command.still_running",
                            &[("command", &cmd)],
                        ),
                        _ => localization::text_with_args(
                            "tui.tool.command.ran",
                            &[("command", &cmd)],
                        ),
                    },
                },
                State::Failed => match block_state {
                    Some(CommandBlockState::Finished { exit_code }) => {
                        command_exit_label(&cmd, exit_code)
                    }
                    Some(CommandBlockState::Running) | None => match result {
                        Some(AIAgentActionResultType::RequestCommandOutput(
                            RequestCommandOutputResult::Completed { exit_code, .. },
                        )) => command_exit_label(&cmd, *exit_code),
                        Some(AIAgentActionResultType::RequestCommandOutput(
                            RequestCommandOutputResult::Denylisted { .. },
                        )) => localization::text_with_args(
                            "tui.tool.command.denied",
                            &[("command", &cmd)],
                        ),
                        _ => localization::text_with_args(
                            "tui.tool.command.failed",
                            &[("command", &cmd)],
                        ),
                    },
                },
                State::Cancelled => {
                    localization::text_with_args("tui.tool.command.cancelled", &[("command", &cmd)])
                }
            }
        }
        AIAgentActionType::WriteToLongRunningShellCommand { .. } => match state {
            State::Constructing => localization::text("tui.tool.command_input.preparing"),
            State::Pending | State::AwaitingApproval => {
                localization::text("tui.tool.command_input.write")
            }
            State::Running => localization::text("tui.tool.command_input.writing"),
            State::Succeeded => localization::text("tui.tool.command_input.wrote"),
            State::Failed => localization::text("tui.tool.command_input.failed"),
            State::Cancelled => localization::text("tui.tool.command_input.cancelled"),
        },
        AIAgentActionType::ReadFiles(request) => {
            let files = files_summary(request.locations.iter().map(|location| &location.name));
            match state {
                State::Constructing => localization::text("tui.tool.files.reading"),
                State::Pending | State::AwaitingApproval | State::Succeeded => {
                    localization::text_with_args("tui.tool.files.read", &[("files", &files)])
                }
                State::Running => localization::text_with_args(
                    "tui.tool.files.reading_named",
                    &[("files", &files)],
                ),
                State::Failed => {
                    localization::text_with_args("tui.tool.files.read_failed", &[("files", &files)])
                }
                State::Cancelled => localization::text_with_args(
                    "tui.tool.files.read_cancelled",
                    &[("files", &files)],
                ),
            }
        }
        AIAgentActionType::UploadArtifact(request) => {
            let file = single_line(&request.file_path);
            match state {
                State::Constructing => localization::text("tui.tool.upload.preparing"),
                State::Pending | State::AwaitingApproval => {
                    localization::text_with_args("tui.tool.upload.start", &[("file", &file)])
                }
                State::Running => {
                    localization::text_with_args("tui.tool.upload.running", &[("file", &file)])
                }
                State::Succeeded => {
                    localization::text_with_args("tui.tool.upload.succeeded", &[("file", &file)])
                }
                State::Failed => {
                    localization::text_with_args("tui.tool.upload.failed", &[("file", &file)])
                }
                State::Cancelled => {
                    localization::text_with_args("tui.tool.upload.cancelled", &[("file", &file)])
                }
            }
        }
        AIAgentActionType::SearchCodebase(request) => {
            let query = single_line(&request.query);
            let scope = request
                .codebase_path
                .as_deref()
                .map(|path| {
                    let path = base_name(path);
                    localization::text_with_args("tui.tool.scope.in", &[("path", &path)])
                })
                .unwrap_or_default();
            match state {
                State::Constructing => localization::text("tui.tool.search.preparing"),
                State::Pending | State::AwaitingApproval => localization::text_with_args(
                    "tui.tool.search.start",
                    &[("query", &query), ("scope", &scope)],
                ),
                State::Running => localization::text_with_args(
                    "tui.tool.search.running",
                    &[("query", &query), ("scope", &scope)],
                ),
                State::Succeeded => match result {
                    Some(AIAgentActionResultType::SearchCodebase(
                        SearchCodebaseResult::Success { files },
                    )) if files.is_empty() => localization::text_with_args(
                        "tui.tool.search.no_results",
                        &[("query", &query), ("scope", &scope)],
                    ),
                    Some(AIAgentActionResultType::SearchCodebase(
                        SearchCodebaseResult::Success { files },
                    )) => {
                        let results = localized_count_label(
                            files.len(),
                            "tui.count.result.one",
                            "tui.count.result.many",
                        );
                        localization::text_with_args(
                            "tui.tool.search.succeeded_with_count",
                            &[("query", &query), ("scope", &scope), ("results", &results)],
                        )
                    }
                    _ => localization::text_with_args(
                        "tui.tool.search.succeeded",
                        &[("query", &query), ("scope", &scope)],
                    ),
                },
                State::Failed => match result {
                    Some(AIAgentActionResultType::SearchCodebase(
                        SearchCodebaseResult::Failed {
                            reason: SearchCodebaseFailureReason::CodebaseNotIndexed,
                            ..
                        },
                    )) => localization::text_with_args(
                        "tui.tool.search.not_indexed",
                        &[("query", &query), ("scope", &scope)],
                    ),
                    _ => localization::text_with_args(
                        "tui.tool.search.failed",
                        &[("query", &query), ("scope", &scope)],
                    ),
                },
                State::Cancelled => localization::text_with_args(
                    "tui.tool.search.cancelled",
                    &[("query", &query), ("scope", &scope)],
                ),
            }
        }
        // Rendered by its own stateful child view (`TuiFileEditsView`); the
        // label path should never be reached for it.
        AIAgentActionType::RequestFileEdits { .. } => {
            log::warn!("tool_call_label called for RequestFileEdits, which has custom rendering");
            String::new()
        }
        AIAgentActionType::Grep { queries, path } => {
            let queries = single_line(&queries.join(", "));
            let path = display_path(path);
            match state {
                State::Constructing => localization::text("tui.tool.grep.preparing"),
                State::Pending | State::AwaitingApproval => localization::text_with_args(
                    "tui.tool.grep.start",
                    &[("queries", &queries), ("path", &path)],
                ),
                State::Running => localization::text_with_args(
                    "tui.tool.grep.running",
                    &[("queries", &queries), ("path", &path)],
                ),
                State::Succeeded => match result {
                    Some(AIAgentActionResultType::Grep(GrepResult::Success { matched_files })) => {
                        let files = localized_count_label(
                            matched_files.len(),
                            "tui.count.matching_file.one",
                            "tui.count.matching_file.many",
                        );
                        localization::text_with_args(
                            "tui.tool.grep.succeeded",
                            &[("queries", &queries), ("path", &path), ("files", &files)],
                        )
                    }
                    _ => localization::text_with_args(
                        "tui.tool.grep.succeeded_without_count",
                        &[("queries", &queries), ("path", &path)],
                    ),
                },
                State::Failed => {
                    localization::text_with_args("tui.tool.grep.failed", &[("queries", &queries)])
                }
                State::Cancelled => localization::text_with_args(
                    "tui.tool.grep.cancelled",
                    &[("queries", &queries)],
                ),
            }
        }
        AIAgentActionType::FileGlob { patterns, path } => {
            file_glob_label(patterns, path.as_deref(), state, None)
        }
        AIAgentActionType::FileGlobV2 {
            patterns,
            search_dir,
        } => {
            let matched_count = match result {
                Some(AIAgentActionResultType::FileGlobV2(FileGlobV2Result::Success {
                    matched_files,
                    ..
                })) => Some(matched_files.len()),
                _ => None,
            };
            file_glob_label(patterns, search_dir.as_deref(), state, matched_count)
        }
        AIAgentActionType::ReadMCPResource { name, uri, .. } => {
            let resource = single_line(uri.as_deref().unwrap_or(name));
            match state {
                // The resource name arrives with the tool-call header (not
                // the streamed args), so include it when present, like the
                // GUI's "Reading \"{name}\" MCP resource..." loading text.
                State::Constructing if name.is_empty() => {
                    localization::text("tui.tool.mcp_resource.preparing")
                }
                State::Constructing => localization::text_with_args(
                    "tui.tool.mcp_resource.preparing_named",
                    &[("name", &name)],
                ),
                State::Pending | State::AwaitingApproval | State::Succeeded => {
                    localization::text_with_args(
                        "tui.tool.mcp_resource.read",
                        &[("resource", &resource)],
                    )
                }
                State::Running => localization::text_with_args(
                    "tui.tool.mcp_resource.reading",
                    &[("resource", &resource)],
                ),
                State::Failed => localization::text_with_args(
                    "tui.tool.mcp_resource.failed",
                    &[("resource", &resource)],
                ),
                State::Cancelled => localization::text_with_args(
                    "tui.tool.mcp_resource.cancelled",
                    &[("resource", &resource)],
                ),
            }
        }
        AIAgentActionType::CallMCPTool { name, .. } => {
            let name = single_line(name);
            match state {
                // Like the GUI's "Calling \"{name}\" MCP tool..." loading
                // text; the tool name is available before its args finish.
                State::Constructing if name.is_empty() => {
                    localization::text("tui.tool.mcp_tool.preparing")
                }
                State::Constructing => localization::text_with_args(
                    "tui.tool.mcp_tool.preparing_named",
                    &[("name", &name)],
                ),
                State::Pending | State::AwaitingApproval => {
                    localization::text_with_args("tui.tool.mcp_tool.start", &[("name", &name)])
                }
                State::Running => {
                    localization::text_with_args("tui.tool.mcp_tool.running", &[("name", &name)])
                }
                State::Succeeded => {
                    localization::text_with_args("tui.tool.mcp_tool.succeeded", &[("name", &name)])
                }
                State::Failed => {
                    localization::text_with_args("tui.tool.mcp_tool.failed", &[("name", &name)])
                }
                State::Cancelled => {
                    localization::text_with_args("tui.tool.mcp_tool.cancelled", &[("name", &name)])
                }
            }
        }
        AIAgentActionType::SuggestNewConversation { .. } => match state {
            State::Constructing => "Suggesting a new conversation…".to_owned(),
            State::Pending | State::AwaitingApproval | State::Running | State::Failed => {
                "Suggested starting a new conversation".to_owned()
            }
            State::Succeeded => match result {
                Some(AIAgentActionResultType::SuggestNewConversation(
                    SuggestNewConversationResult::Rejected,
                )) => "Continuing current conversation".to_owned(),
                _ => "New conversation started".to_owned(),
            },
            State::Cancelled => "New conversation suggestion cancelled".to_owned(),
        },
        AIAgentActionType::SuggestPrompt(_)
        | AIAgentActionType::InitProject
        | AIAgentActionType::OpenCodeReview => fallback_label(action, state),
        AIAgentActionType::ReadDocuments(request) => {
            let documents = count_label(request.document_ids.len(), "document", "documents");
            match state {
                State::Constructing => "Reading documents…".to_owned(),
                State::Pending | State::AwaitingApproval | State::Succeeded => {
                    format!("Read {documents}")
                }
                State::Running => format!("Reading {documents}"),
                State::Failed => "Failed to read documents".to_owned(),
                State::Cancelled => "Cancelled reading documents".to_owned(),
            }
        }
        AIAgentActionType::EditDocuments(request) => match state {
            State::Pending | State::AwaitingApproval => "Update plan".to_owned(),
            State::Constructing | State::Running => "Updating plan…".to_owned(),
            State::Succeeded => format!(
                "Updated plan ({})",
                count_label(request.diffs.len(), "edit", "edits")
            ),
            State::Failed => "Failed to update plan".to_owned(),
            State::Cancelled => "Update plan cancelled".to_owned(),
        },
        AIAgentActionType::CreateDocuments(request) => match state {
            State::Pending | State::AwaitingApproval => "Create plan".to_owned(),
            State::Constructing | State::Running => "Generating plan…".to_owned(),
            State::Succeeded => {
                let count = request.documents.len();
                if count > 1 {
                    format!("Created {count} documents")
                } else {
                    "Created plan".to_owned()
                }
            }
            State::Failed => "Failed to create plan".to_owned(),
            State::Cancelled => "Create plan cancelled".to_owned(),
        },
        AIAgentActionType::ReadShellCommandOutput { .. } => match state {
            State::Pending | State::AwaitingApproval | State::Succeeded => {
                "Read command output".to_owned()
            }
            State::Constructing | State::Running => "Reading command output…".to_owned(),
            State::Failed => "Failed to read command output".to_owned(),
            State::Cancelled => "Read command output cancelled".to_owned(),
        },
        AIAgentActionType::UseComputer(request) => summary_label(&request.action_summary, state),
        AIAgentActionType::InsertCodeReviewComments { comments, .. } => {
            let comments = count_label(comments.len(), "review comment", "review comments");
            match state {
                State::Constructing => "Preparing review comments…".to_owned(),
                State::Pending | State::AwaitingApproval => format!("Insert {comments}"),
                State::Running => format!("Inserting {comments}…"),
                State::Succeeded => format!("Inserted {comments}"),
                State::Failed => "Failed to insert review comments".to_owned(),
                State::Cancelled => "Insert review comments cancelled".to_owned(),
            }
        }
        AIAgentActionType::RequestComputerUse(request) => {
            summary_label(&request.task_summary, state)
        }
        AIAgentActionType::StartRecording { .. } => match state {
            State::Pending | State::AwaitingApproval => "Start recording".to_owned(),
            State::Constructing | State::Running => "Starting recording…".to_owned(),
            State::Succeeded => "Started screen recording".to_owned(),
            State::Failed => "Recording failed to start".to_owned(),
            State::Cancelled => "Start recording cancelled".to_owned(),
        },
        AIAgentActionType::StopRecording { .. } => match state {
            State::Pending | State::AwaitingApproval => "Stop recording".to_owned(),
            State::Constructing | State::Running => "Stopping recording…".to_owned(),
            State::Succeeded => "Saved screen recording".to_owned(),
            State::Failed => "Failed to save recording".to_owned(),
            State::Cancelled => "Stop recording cancelled".to_owned(),
        },
        AIAgentActionType::ReadSkill(request) => {
            let skill = single_line(&request.skill.display_label());
            match state {
                State::Constructing => "Reading skill…".to_owned(),
                State::Pending | State::AwaitingApproval | State::Succeeded => {
                    format!("Read skill {skill}")
                }
                State::Running => format!("Reading skill {skill}"),
                State::Failed => format!("Failed to read skill {skill}"),
                State::Cancelled => format!("Cancelled reading skill {skill}"),
            }
        }
        AIAgentActionType::FetchConversation { .. } => match state {
            State::Pending | State::AwaitingApproval => "Fetch conversation".to_owned(),
            State::Constructing | State::Running => "Fetching conversation…".to_owned(),
            State::Succeeded => "Fetched conversation".to_owned(),
            State::Failed => "Fetch conversation failed".to_owned(),
            State::Cancelled => "Fetch conversation cancelled".to_owned(),
        },
        AIAgentActionType::StartAgent {
            name,
            execution_mode,
            ..
        } => {
            let agent = if matches!(execution_mode, StartAgentExecutionMode::Remote { .. }) {
                format!("remote agent {name}")
            } else {
                format!("agent {name}")
            };
            match state {
                State::Constructing => "Configuring agent…".to_owned(),
                State::Pending | State::AwaitingApproval => format!("Start {agent}"),
                State::Running => format!("Starting {agent}…"),
                State::Succeeded => format!("Started agent {name}"),
                State::Failed => format!("Failed to start agent {name}"),
                State::Cancelled => format!("Start agent {name} cancelled"),
            }
        }
        AIAgentActionType::SendMessageToAgent {
            addresses, subject, ..
        } => {
            let subject = single_line(subject);
            match state {
                State::Constructing => "Composing message…".to_owned(),
                State::Pending | State::AwaitingApproval => format!("Send message: {subject}"),
                State::Running => format!(
                    "Sending message to {}: {subject}",
                    count_label(addresses.len(), "agent", "agents")
                ),
                State::Succeeded => format!("Sent message: {subject}"),
                State::Failed => format!("Failed to send message: {subject}"),
                State::Cancelled => "Send message cancelled".to_owned(),
            }
        }
        AIAgentActionType::TransferShellCommandControlToUser { reason } => match state {
            State::Constructing => "Handing control to you…".to_owned(),
            State::Pending | State::AwaitingApproval | State::Running => {
                format!("Handing control to you: {}", single_line(reason))
            }
            State::Succeeded => "You are in control".to_owned(),
            State::Failed => "Control transfer failed".to_owned(),
            State::Cancelled => "Control transfer cancelled".to_owned(),
        },
        AIAgentActionType::AskUserQuestion { questions } => match state {
            State::Constructing => "Preparing question…".to_owned(),
            State::Pending | State::AwaitingApproval | State::Running => format!(
                "Asking {}",
                count_label(questions.len(), "question", "questions")
            ),
            State::Succeeded => match result {
                Some(AIAgentActionResultType::AskUserQuestion(
                    AskUserQuestionResult::Success { answers },
                )) => {
                    let total = answers.len();
                    let answered = answers.iter().filter(|answer| !answer.is_skipped()).count();
                    if answered == 0 {
                        "Questions skipped".to_owned()
                    } else if answered == total && total == 1 {
                        "Answered question".to_owned()
                    } else if answered == total {
                        format!("Answered all {total} questions")
                    } else {
                        format!("Answered {answered} of {total} questions")
                    }
                }
                Some(AIAgentActionResultType::AskUserQuestion(
                    AskUserQuestionResult::SkippedByAutoApprove { .. },
                )) => "Questions skipped".to_owned(),
                _ => "Answered questions".to_owned(),
            },
            State::Failed => "Questions failed".to_owned(),
            State::Cancelled => "Questions cancelled".to_owned(),
        },
        AIAgentActionType::RunAgents(request) => {
            let total = request.agent_run_configs.len();
            match state {
                State::Constructing | State::Pending | State::AwaitingApproval => {
                    "Configuring agents…".to_owned()
                }
                State::Running => {
                    format!("Spawning {}…", count_label(total, "agent", "agents"))
                }
                State::Succeeded => match result {
                    Some(AIAgentActionResultType::RunAgents(RunAgentsResult::Launched {
                        agents,
                        ..
                    })) => {
                        let launched = agents
                            .iter()
                            .filter(|agent| {
                                matches!(agent.kind, RunAgentsAgentOutcomeKind::Launched { .. })
                            })
                            .count();
                        let total = agents.len();
                        if launched == total {
                            format!("Spawned {}", count_label(total, "agent", "agents"))
                        } else if launched == 0 {
                            format!("Failed to spawn {}", count_label(total, "agent", "agents"))
                        } else {
                            format!("Spawned {launched} of {total} agents")
                        }
                    }
                    _ => format!("Spawned {}", count_label(total, "agent", "agents")),
                },
                State::Failed => match result {
                    Some(AIAgentActionResultType::RunAgents(RunAgentsResult::Denied {
                        ..
                    })) => "Orchestration disabled — agents not launched".to_owned(),
                    Some(AIAgentActionResultType::RunAgents(RunAgentsResult::Failure {
                        error,
                    })) if !error.is_empty() => {
                        format!("Failed to start orchestration: {}", single_line(error))
                    }
                    _ => "Failed to start orchestration".to_owned(),
                },
                State::Cancelled => "Spawn agents cancelled".to_owned(),
            }
        }
        AIAgentActionType::WaitForEvents { .. } => match state {
            State::Constructing | State::Pending | State::AwaitingApproval | State::Running => {
                "Waiting for agent events…".to_owned()
            }
            State::Succeeded => "Done waiting for agent events".to_owned(),
            State::Failed => "Waiting for agent events failed".to_owned(),
            State::Cancelled => "Wait for events cancelled".to_owned(),
        },
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
        State::Pending | State::AwaitingApproval => localization::text_with_args(
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
        State::Constructing => "Preparing computer use…".to_owned(),
        State::Pending | State::AwaitingApproval | State::Running | State::Succeeded => summary,
        State::Failed => format!("{summary} — failed"),
        State::Cancelled => format!("{summary} — cancelled"),
    }
}

/// Generic label for action types without bespoke text, derived from the
/// action's user-friendly name.
fn fallback_label(action: &AIAgentActionType, state: ToolCallDisplayState) -> String {
    let name = action.user_friendly_name();
    match state {
        State::Pending | State::AwaitingApproval => name,
        State::Constructing | State::Running => format!("{name}…"),
        State::Succeeded => format!("{name} — done"),
        State::Failed => format!("{name} — failed"),
        State::Cancelled => format!("{name} — cancelled"),
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

/// Pluralizes a counted noun, e.g. `count_label(2, "file", "files")` → "2 files".
fn count_label(count: usize, singular: &str, plural: &str) -> String {
    let noun = if count == 1 { singular } else { plural };
    format!("{count} {noun}")
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
