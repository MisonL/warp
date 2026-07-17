use super::*;

/// Builds the per-tool label body; the awaiting-approval suffix is applied by
/// [`super::tool_call_label`]. `result` is the finished result, when there is one.
///
/// `Constructing` arms never interpolate argument fields (they may be empty
/// or partial while streaming); their copy is indexed on the GUI's loading
/// messages (`common.rs` `LOAD_OUTPUT_MESSAGE_*` and the requested-command
/// view's "Generating command...").
pub(super) fn label_for_action(
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
                    &[("name", name)],
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
            State::Constructing => localization::text("tui.tool.conversation.suggesting"),
            State::Pending | State::AwaitingApproval | State::Running | State::Failed => {
                localization::text("tui.tool.conversation.suggested")
            }
            State::Succeeded => match result {
                Some(AIAgentActionResultType::SuggestNewConversation(
                    SuggestNewConversationResult::Rejected,
                )) => localization::text("tui.tool.conversation.continuing_current"),
                _ => localization::text("tui.tool.conversation.started_new"),
            },
            State::Cancelled => localization::text("tui.tool.conversation.cancelled"),
        },
        AIAgentActionType::SuggestPrompt(_) => fallback_label(
            localization::text("agent.action.name.suggest_prompt"),
            state,
        ),
        AIAgentActionType::InitProject => {
            fallback_label(localization::text("agent.action.name.init_project"), state)
        }
        AIAgentActionType::OpenCodeReview => fallback_label(
            localization::text("agent.action.name.open_code_review"),
            state,
        ),
        AIAgentActionType::ReadDocuments(request) => {
            let documents = localized_count_label(
                request.document_ids.len(),
                "tui.count.document.one",
                "tui.count.document.many",
            );
            match state {
                State::Constructing => localization::text("tui.tool.documents.reading"),
                State::Pending | State::AwaitingApproval | State::Succeeded => {
                    localization::text_with_args(
                        "tui.tool.documents.read",
                        &[("documents", &documents)],
                    )
                }
                State::Running => localization::text_with_args(
                    "tui.tool.documents.reading_count",
                    &[("documents", &documents)],
                ),
                State::Failed => localization::text("tui.tool.documents.failed"),
                State::Cancelled => localization::text("tui.tool.documents.cancelled"),
            }
        }
        AIAgentActionType::EditDocuments(request) => match state {
            State::Pending | State::AwaitingApproval => localization::text("tui.tool.plan.update"),
            State::Constructing | State::Running => localization::text("tui.tool.plan.updating"),
            State::Succeeded => {
                let edits = localized_count_label(
                    request.diffs.len(),
                    "tui.count.edit.one",
                    "tui.count.edit.many",
                );
                localization::text_with_args("tui.tool.plan.updated", &[("edits", &edits)])
            }
            State::Failed => localization::text("tui.tool.plan.update_failed"),
            State::Cancelled => localization::text("tui.tool.plan.update_cancelled"),
        },
        AIAgentActionType::CreateDocuments(request) => match state {
            State::Pending | State::AwaitingApproval => localization::text("tui.tool.plan.create"),
            State::Constructing | State::Running => localization::text("tui.tool.plan.generating"),
            State::Succeeded => {
                let count = request.documents.len();
                if count > 1 {
                    localization::text_with_args(
                        "tui.tool.plan.created_documents",
                        &[("count", &count.to_string())],
                    )
                } else {
                    localization::text("tui.tool.plan.created")
                }
            }
            State::Failed => localization::text("tui.tool.plan.create_failed"),
            State::Cancelled => localization::text("tui.tool.plan.create_cancelled"),
        },
        AIAgentActionType::ReadShellCommandOutput { .. } => match state {
            State::Pending | State::AwaitingApproval | State::Succeeded => {
                localization::text("tui.tool.command_output.read")
            }
            State::Constructing | State::Running => {
                localization::text("tui.tool.command_output.reading")
            }
            State::Failed => localization::text("tui.tool.command_output.failed"),
            State::Cancelled => localization::text("tui.tool.command_output.cancelled"),
        },
        AIAgentActionType::UseComputer(request) => summary_label(&request.action_summary, state),
        AIAgentActionType::InsertCodeReviewComments { comments, .. } => {
            let comments = localized_count_label(
                comments.len(),
                "tui.count.review_comment.one",
                "tui.count.review_comment.many",
            );
            match state {
                State::Constructing => localization::text("tui.tool.review.preparing"),
                State::Pending | State::AwaitingApproval => localization::text_with_args(
                    "tui.tool.review.insert",
                    &[("comments", &comments)],
                ),
                State::Running => localization::text_with_args(
                    "tui.tool.review.inserting",
                    &[("comments", &comments)],
                ),
                State::Succeeded => localization::text_with_args(
                    "tui.tool.review.inserted",
                    &[("comments", &comments)],
                ),
                State::Failed => localization::text("tui.tool.review.failed"),
                State::Cancelled => localization::text("tui.tool.review.cancelled"),
            }
        }
        AIAgentActionType::RequestComputerUse(request) => {
            summary_label(&request.task_summary, state)
        }
        AIAgentActionType::StartRecording { .. } => match state {
            State::Pending | State::AwaitingApproval => {
                localization::text("tui.tool.recording.start")
            }
            State::Constructing | State::Running => {
                localization::text("tui.tool.recording.starting")
            }
            State::Succeeded => localization::text("tui.tool.recording.started"),
            State::Failed => localization::text("tui.tool.recording.start_failed"),
            State::Cancelled => localization::text("tui.tool.recording.start_cancelled"),
        },
        AIAgentActionType::StopRecording { .. } => match state {
            State::Pending | State::AwaitingApproval => {
                localization::text("tui.tool.recording.stop")
            }
            State::Constructing | State::Running => {
                localization::text("tui.tool.recording.stopping")
            }
            State::Succeeded => localization::text("tui.tool.recording.saved"),
            State::Failed => localization::text("tui.tool.recording.save_failed"),
            State::Cancelled => localization::text("tui.tool.recording.stop_cancelled"),
        },
        AIAgentActionType::ReadSkill(request) => {
            let skill = single_line(&request.skill.display_label());
            match state {
                State::Constructing => localization::text("tui.tool.skill.reading"),
                State::Pending | State::AwaitingApproval | State::Succeeded => {
                    localization::text_with_args("tui.tool.skill.read", &[("skill", &skill)])
                }
                State::Running => localization::text_with_args(
                    "tui.tool.skill.reading_named",
                    &[("skill", &skill)],
                ),
                State::Failed => {
                    localization::text_with_args("tui.tool.skill.failed", &[("skill", &skill)])
                }
                State::Cancelled => {
                    localization::text_with_args("tui.tool.skill.cancelled", &[("skill", &skill)])
                }
            }
        }
        AIAgentActionType::FetchConversation { .. } => match state {
            State::Pending | State::AwaitingApproval => {
                localization::text("tui.tool.conversation.fetch")
            }
            State::Constructing | State::Running => {
                localization::text("tui.tool.conversation.fetching")
            }
            State::Succeeded => localization::text("tui.tool.conversation.fetched"),
            State::Failed => localization::text("tui.tool.conversation.fetch_failed"),
            State::Cancelled => localization::text("tui.tool.conversation.fetch_cancelled"),
        },
        AIAgentActionType::StartAgent {
            name,
            execution_mode,
            ..
        } => {
            let agent = if matches!(execution_mode, StartAgentExecutionMode::Remote { .. }) {
                localization::text_with_args("tui.tool.agent.remote", &[("name", name)])
            } else {
                localization::text_with_args("tui.tool.agent.local", &[("name", name)])
            };
            match state {
                State::Constructing => localization::text("tui.tool.agent.configuring"),
                State::Pending | State::AwaitingApproval => {
                    localization::text_with_args("tui.tool.agent.start", &[("agent", &agent)])
                }
                State::Running => {
                    localization::text_with_args("tui.tool.agent.starting", &[("agent", &agent)])
                }
                State::Succeeded => {
                    localization::text_with_args("tui.tool.agent.started", &[("name", name)])
                }
                State::Failed => {
                    localization::text_with_args("tui.tool.agent.start_failed", &[("name", name)])
                }
                State::Cancelled => localization::text_with_args(
                    "tui.tool.agent.start_cancelled",
                    &[("name", name)],
                ),
            }
        }
        AIAgentActionType::SendMessageToAgent {
            addresses, subject, ..
        } => {
            let subject = single_line(subject);
            match state {
                State::Constructing => localization::text("tui.tool.message.composing"),
                State::Pending | State::AwaitingApproval => {
                    localization::text_with_args("tui.tool.message.send", &[("subject", &subject)])
                }
                State::Running => {
                    let agents = localized_count_label(
                        addresses.len(),
                        "tui.count.agent.one",
                        "tui.count.agent.many",
                    );
                    localization::text_with_args(
                        "tui.tool.message.sending",
                        &[("agents", &agents), ("subject", &subject)],
                    )
                }
                State::Succeeded => {
                    localization::text_with_args("tui.tool.message.sent", &[("subject", &subject)])
                }
                State::Failed => localization::text_with_args(
                    "tui.tool.message.failed",
                    &[("subject", &subject)],
                ),
                State::Cancelled => localization::text("tui.tool.message.cancelled"),
            }
        }
        AIAgentActionType::TransferShellCommandControlToUser { reason } => match state {
            State::Constructing => localization::text("tui.tool.control.handing"),
            State::Pending | State::AwaitingApproval | State::Running => {
                let reason = single_line(reason);
                localization::text_with_args(
                    "tui.tool.control.handing_reason",
                    &[("reason", &reason)],
                )
            }
            State::Succeeded => localization::text("tui.tool.control.in_control"),
            State::Failed => localization::text("tui.tool.control.failed"),
            State::Cancelled => localization::text("tui.tool.control.cancelled"),
        },
        AIAgentActionType::AskUserQuestion { questions } => match state {
            State::Constructing => localization::text("tui.tool.question.preparing"),
            State::Pending | State::AwaitingApproval | State::Running => {
                let questions = localized_count_label(
                    questions.len(),
                    "tui.count.question.one",
                    "tui.count.question.many",
                );
                localization::text_with_args(
                    "tui.tool.question.asking",
                    &[("questions", &questions)],
                )
            }
            State::Succeeded => match result {
                Some(AIAgentActionResultType::AskUserQuestion(
                    AskUserQuestionResult::Success { answers },
                )) => {
                    let total = answers.len();
                    let answered = answers.iter().filter(|answer| !answer.is_skipped()).count();
                    if answered == 0 {
                        localization::text("tui.tool.question.skipped")
                    } else if answered == total && total == 1 {
                        localization::text("tui.tool.question.answered_one")
                    } else if answered == total {
                        localization::text_with_args(
                            "tui.tool.question.answered_all",
                            &[("count", &total.to_string())],
                        )
                    } else {
                        localization::text_with_args(
                            "tui.tool.question.answered_some",
                            &[
                                ("answered", &answered.to_string()),
                                ("total", &total.to_string()),
                            ],
                        )
                    }
                }
                Some(AIAgentActionResultType::AskUserQuestion(
                    AskUserQuestionResult::SkippedByAutoApprove { .. },
                )) => localization::text("tui.tool.question.skipped"),
                _ => localization::text("tui.tool.question.answered"),
            },
            State::Failed => localization::text("tui.tool.question.failed"),
            State::Cancelled => localization::text("tui.tool.question.cancelled"),
        },
        AIAgentActionType::RunAgents(request) => {
            let total = request.agent_run_configs.len();
            match state {
                State::Constructing | State::Pending | State::AwaitingApproval => {
                    localization::text("tui.tool.orchestration.configuring")
                }
                State::Running => {
                    let agents =
                        localized_count_label(total, "tui.count.agent.one", "tui.count.agent.many");
                    localization::text_with_args(
                        "tui.tool.orchestration.spawning",
                        &[("agents", &agents)],
                    )
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
                        let agents = localized_count_label(
                            total,
                            "tui.count.agent.one",
                            "tui.count.agent.many",
                        );
                        if launched == total {
                            localization::text_with_args(
                                "tui.tool.orchestration.spawned",
                                &[("agents", &agents)],
                            )
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
                    _ => {
                        let agents = localized_count_label(
                            total,
                            "tui.count.agent.one",
                            "tui.count.agent.many",
                        );
                        localization::text_with_args(
                            "tui.tool.orchestration.spawned",
                            &[("agents", &agents)],
                        )
                    }
                },
                State::Failed => match result {
                    Some(AIAgentActionResultType::RunAgents(RunAgentsResult::Denied {
                        ..
                    })) => localization::text("tui.tool.orchestration.disabled"),
                    Some(AIAgentActionResultType::RunAgents(RunAgentsResult::Failure {
                        error,
                    })) if !error.is_empty() => {
                        let error = single_line(error);
                        localization::text_with_args(
                            "tui.tool.orchestration.failed_with_error",
                            &[("error", &error)],
                        )
                    }
                    _ => localization::text("tui.tool.orchestration.failed"),
                },
                State::Cancelled => localization::text("tui.tool.orchestration.cancelled"),
            }
        }
        AIAgentActionType::WaitForEvents { .. } => match state {
            State::Constructing | State::Pending | State::AwaitingApproval | State::Running => {
                localization::text("tui.tool.events.waiting")
            }
            State::Succeeded => localization::text("tui.tool.events.done"),
            State::Failed => localization::text("tui.tool.events.failed"),
            State::Cancelled => localization::text("tui.tool.events.cancelled"),
        },
    }
}
