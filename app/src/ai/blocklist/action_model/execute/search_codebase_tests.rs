use warp_localization::LocaleId;

use super::{codebase_unavailable_message, protocol_message_with_args, search_failed_message};

#[test]
fn search_codebase_protocol_errors_use_en_us_text() {
    assert_eq!(
        search_failed_message(),
        crate::localization::text_for_locale(
            LocaleId::EnUs,
            "agent.search_codebase.error.search_failed"
        )
    );
    assert_ne!(
        search_failed_message(),
        crate::localization::text_for_locale(
            LocaleId::ZhCn,
            "agent.search_codebase.error.search_failed"
        )
    );

    assert_eq!(
        codebase_unavailable_message(),
        crate::localization::text_for_locale(
            LocaleId::EnUs,
            "agent.search_codebase.error.codebase_unavailable"
        )
    );
    assert_ne!(
        codebase_unavailable_message(),
        crate::localization::text_for_locale(
            LocaleId::ZhCn,
            "agent.search_codebase.error.codebase_unavailable"
        )
    );
}

#[test]
fn search_codebase_protocol_errors_keep_arguments() {
    let message = protocol_message_with_args(
        "agent.search_codebase.error.missing_files",
        &[("files", "src/main.rs, src/lib.rs")],
    );

    assert_eq!(message, "These files do not exist: src/main.rs, src/lib.rs");
}
