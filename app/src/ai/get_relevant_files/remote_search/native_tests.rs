use std::path::PathBuf;

use remote_server::proto::{file_context_proto, FileContextProto, ReadFileContextResponse};
use warp_localization::LocaleId;

use super::{file_contents_from_response, protocol_message, protocol_message_with_args};

#[test]
fn file_contents_from_response_keeps_only_whole_text_files() {
    let response = ReadFileContextResponse {
        file_contexts: vec![
            FileContextProto {
                file_name: "/repo/src/lib.rs".to_string(),
                content: Some(file_context_proto::Content::TextContent(
                    "content".to_string(),
                )),
                line_range_start: None,
                line_range_end: None,
                last_modified_epoch_millis: None,
                line_count: 1,
            },
            FileContextProto {
                file_name: "/repo/src/fragment.rs".to_string(),
                content: Some(file_context_proto::Content::TextContent(
                    "fragment".to_string(),
                )),
                line_range_start: Some(1),
                line_range_end: Some(2),
                last_modified_epoch_millis: None,
                line_count: 1,
            },
        ],
        failed_files: vec![],
    };

    let file_contents = file_contents_from_response(response);

    assert_eq!(file_contents.len(), 1);
    assert_eq!(
        file_contents.get(&PathBuf::from("/repo/src/lib.rs")),
        Some(&"content".to_string())
    );
}

#[test]
fn remote_search_protocol_errors_use_en_us_text() {
    assert_eq!(
        protocol_message("agent.search_codebase.error.remote_not_enabled"),
        crate::localization::text_for_locale(
            LocaleId::EnUs,
            "agent.search_codebase.error.remote_not_enabled"
        )
    );
    assert_ne!(
        protocol_message("agent.search_codebase.error.remote_not_enabled"),
        crate::localization::text_for_locale(
            LocaleId::ZhCn,
            "agent.search_codebase.error.remote_not_enabled"
        )
    );
}

#[test]
fn remote_search_protocol_errors_keep_en_us_templates_with_arguments() {
    let message = protocol_message_with_args(
        "agent.search_codebase.error.remote_read_failed",
        &[("failed", "/repo/src/main.rs: denied")],
    );

    assert_eq!(
        message,
        "Failed to read remote search result files: /repo/src/main.rs: denied"
    );
}
