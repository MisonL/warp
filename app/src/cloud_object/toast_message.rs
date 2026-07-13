use warpui::AppContext;

use super::{CloudObject, GenericStringObjectFormat, JsonObjectType, ObjectType};
use crate::localization;
use crate::server::cloud_objects::update_manager::{
    InitiatedBy, ObjectOperation, OperationSuccessType,
};

pub struct CloudObjectToastMessage;

impl CloudObjectToastMessage {
    pub fn toast_message(
        object: &dyn CloudObject,
        operation: &ObjectOperation,
        success_type: &OperationSuccessType,
        app: &AppContext,
    ) -> Option<String> {
        let object_name = object.model_type_name().to_owned();
        let object_name_lowercase = object_name.to_ascii_lowercase();

        match (object.object_type(), operation, success_type) {
            // We should only show toasts for creates initiated by the user, not by the system
            (
                _,
                ObjectOperation::Create {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Success,
            ) => {
                let containing_object_name = object.containing_object_name(app);
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.saved_to",
                    &[
                        ("object", &object_name),
                        ("location", &containing_object_name),
                    ],
                ))
            }
            // notebooks intentionally do not have an update message, as they are updated
            // as the user types and so toasts would be VERY noisy
            (ObjectType::Notebook, ObjectOperation::Update, OperationSuccessType::Success) => None,
            (_, ObjectOperation::Update, OperationSuccessType::Success) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.updated",
                    &[("object", &object_name)],
                ))
            }
            (_, ObjectOperation::MoveToFolder, OperationSuccessType::Success)
            | (_, ObjectOperation::MoveToDrive, OperationSuccessType::Success) => {
                let containing_object_name = object.containing_object_name(app);
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.moved_to",
                    &[
                        ("object", &object_name),
                        ("location", &containing_object_name),
                    ],
                ))
            }
            (_, ObjectOperation::Trash, OperationSuccessType::Success) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.trashed",
                    &[("object", &object_name)],
                ))
            }
            (_, ObjectOperation::Untrash, OperationSuccessType::Success) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.restored",
                    &[("object", &object_name)],
                ))
            }
            (_, ObjectOperation::Leave, OperationSuccessType::Success) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.left",
                    &[("object", &object_name)],
                ))
            }
            (
                _,
                ObjectOperation::Create {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Failure,
            ) => Some(localization::text_for_app_with_args(
                app,
                "cloud_object.toast.failed_create",
                &[("object", &object_name_lowercase)],
            )),
            (
                _,
                ObjectOperation::Create {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Denied(message),
            ) => Some(message.to_string()),
            (_, ObjectOperation::Update, OperationSuccessType::Failure) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.failed_update",
                    &[("object", &object_name_lowercase)],
                ))
            }
            (_, ObjectOperation::MoveToFolder, OperationSuccessType::Failure)
            | (_, ObjectOperation::MoveToDrive, OperationSuccessType::Failure) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.failed_move",
                    &[("object", &object_name_lowercase)],
                ))
            }
            (_, ObjectOperation::Trash, OperationSuccessType::Failure) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.failed_trash",
                    &[("object", &object_name_lowercase)],
                ))
            }
            (_, ObjectOperation::Untrash, OperationSuccessType::Failure) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.failed_restore",
                    &[("object", &object_name_lowercase)],
                ))
            }
            // We should only show deletion failure toasts for user-initiated deletions.
            (
                _,
                ObjectOperation::Delete {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Failure,
            ) => Some(localization::text_for_app_with_args(
                app,
                "cloud_object.toast.failed_delete",
                &[("object", &object_name_lowercase)],
            )),
            (_, ObjectOperation::Leave, OperationSuccessType::Failure) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.failed_leave",
                    &[("object", &object_name)],
                ))
            }
            (ObjectType::Workflow, ObjectOperation::Update, OperationSuccessType::Rejection) => {
                Some(localization::text_for_app(
                    app,
                    "cloud_object.toast.rejection.workflow",
                ))
            }
            (
                ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
                    JsonObjectType::EnvVarCollection,
                )),
                ObjectOperation::Update,
                OperationSuccessType::Rejection,
            ) => Some(localization::text_for_app(
                app,
                "cloud_object.toast.rejection.env_vars",
            )),
            (
                ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
                    JsonObjectType::AIFact,
                )),
                ObjectOperation::Update,
                OperationSuccessType::Rejection,
            ) => Some(localization::text_for_app(
                app,
                "cloud_object.toast.rejection.rule",
            )),
            (_, ObjectOperation::TakeEditAccess, OperationSuccessType::Failure) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.failed_start_editing",
                    &[("object", &object_name_lowercase)],
                ))
            }
            (_, ObjectOperation::UpdatePermissions, OperationSuccessType::Success) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.updated_permissions",
                    &[("object", &object_name_lowercase)],
                ))
            }
            (_, ObjectOperation::UpdatePermissions, OperationSuccessType::Failure) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.failed_update_permissions",
                    &[("object", &object_name_lowercase)],
                ))
            }
            _ => None,
        }
    }

    pub fn toast_deletion_confirm_message(
        num_objects: i32,
        operation: &ObjectOperation,
        success_type: &OperationSuccessType,
        app: &AppContext,
    ) -> Option<String> {
        let count_objects_message = match num_objects {
            1 => localization::text_for_app(app, "cloud_object.toast.object_count.singular"),
            n => localization::text_for_app_with_args(
                app,
                "cloud_object.toast.object_count.plural",
                &[("count", &n.to_string())],
            ),
        };
        match (operation, success_type) {
            // We should only show deletion failure toasts for user-initiated deletions.
            (
                ObjectOperation::Delete {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Success,
            ) => Some(localization::text_for_app_with_args(
                app,
                "cloud_object.toast.deleted_forever",
                &[("count_objects", &count_objects_message)],
            )),
            (ObjectOperation::EmptyTrash, OperationSuccessType::Success) => {
                Some(localization::text_for_app_with_args(
                    app,
                    "cloud_object.toast.trash_emptied",
                    &[("count_objects", &count_objects_message)],
                ))
            }
            (ObjectOperation::EmptyTrash, OperationSuccessType::Failure) => Some(
                localization::text_for_app(app, "cloud_object.toast.failed_empty_trash"),
            ),
            (ObjectOperation::EmptyTrash, OperationSuccessType::Rejection) => Some(
                localization::text_for_app(app, "cloud_object.toast.no_objects_to_empty"),
            ),
            _ => None,
        }
    }
}
