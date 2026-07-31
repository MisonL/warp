use warp_localization::LocaleId;

use super::{
    AttachmentModeTransition, ImageAttachmentValidationError, MAX_IMAGE_COUNT_FOR_QUERY,
    attachment_mode_transition, default_image_name_for_locale,
    image_attachment_validation_error_for_locale, reconciled_selected_index,
};

#[test]
fn selection_tracks_newest_and_clamps_after_removal() {
    assert_eq!(reconciled_selected_index(0, 2, None), Some(1));
    assert_eq!(reconciled_selected_index(2, 1, Some(1)), Some(0));
    assert_eq!(reconciled_selected_index(1, 0, Some(0)), None);
}

#[test]
fn attachment_transitions_lock_and_restore_nld() {
    assert_eq!(
        attachment_mode_transition(false, true, true, false),
        AttachmentModeTransition::LockAgent
    );
    assert_eq!(
        attachment_mode_transition(true, true, true, false),
        AttachmentModeTransition::None
    );
    assert_eq!(
        attachment_mode_transition(true, false, true, false),
        AttachmentModeTransition::RestoreAgent {
            request_detection: true
        }
    );
    assert_eq!(
        attachment_mode_transition(true, false, true, true),
        AttachmentModeTransition::RestoreAgent {
            request_detection: false
        }
    );
    assert_eq!(
        attachment_mode_transition(true, false, false, false),
        AttachmentModeTransition::RestoreAgent {
            request_detection: false
        }
    );
}

#[test]
fn image_attachment_validation_errors_are_localized() {
    assert_eq!(
        image_attachment_validation_error_for_locale(
            LocaleId::ZhCn,
            ImageAttachmentValidationError::Unavailable,
        ),
        "图像附件不可用。"
    );
    assert_eq!(
        image_attachment_validation_error_for_locale(
            LocaleId::ZhCn,
            ImageAttachmentValidationError::ProcessingInProgress,
        ),
        "请等待当前图像附件处理完成。"
    );
    assert_eq!(
        image_attachment_validation_error_for_locale(
            LocaleId::ZhCn,
            ImageAttachmentValidationError::LimitPerQuery,
        ),
        format!("每个查询最多可附加 {MAX_IMAGE_COUNT_FOR_QUERY} 张图像。")
    );
    assert_eq!(
        image_attachment_validation_error_for_locale(
            LocaleId::ZhCn,
            ImageAttachmentValidationError::ModelUnsupported,
        ),
        "所选模型不支持图像附件。"
    );
}

#[test]
fn default_image_name_is_localized() {
    assert_eq!(
        default_image_name_for_locale(LocaleId::ZhCn),
        "\u{56fe}\u{50cf}"
    );
}
