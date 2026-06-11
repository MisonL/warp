use std::path::PathBuf;

use warp_localization::LocaleId;
use warpui::{AppContext, ModelContext};

use crate::ai::agent::{AIAgentActionId, SearchCodebaseFailureReason, SearchCodebaseResult};
use crate::ai::blocklist::SessionContext;
use crate::ai::get_relevant_files::controller::GetRelevantFilesController;
use crate::localization;

pub(super) enum RemoteSearchRequest {
    Ready(SearchCodebaseResult),
}

pub(super) fn root_directory_for_search(
    _session_context: &SessionContext,
    _requested_codebase_path: Option<&str>,
    _app: &AppContext,
) -> Option<PathBuf> {
    None
}

pub(super) fn send_request(
    _query: String,
    _partial_paths: Option<Vec<String>>,
    _session_context: SessionContext,
    _requested_codebase_path: Option<String>,
    _action_id: AIAgentActionId,
    _ctx: &mut ModelContext<GetRelevantFilesController>,
) -> RemoteSearchRequest {
    RemoteSearchRequest::Ready(SearchCodebaseResult::Failed {
        reason: SearchCodebaseFailureReason::CodebaseNotIndexed,
        message: protocol_message("agent.search_codebase.error.remote_unavailable"),
    })
}

fn protocol_message(key: &str) -> String {
    localization::text_for_locale(LocaleId::EnUs, key)
}
