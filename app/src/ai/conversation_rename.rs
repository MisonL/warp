use warpui::{SingletonEntity, View, ViewContext};

use crate::ai::agent::conversation::AIConversationId;
use crate::ai::blocklist::{BeginConversationRenameError, BlocklistAIHistoryModel};
use crate::localization;
use crate::server::server_api::ServerApiProvider;
use crate::view_components::DismissibleToast;
use crate::workspace::ToastStack;

const CONVERSATION_TITLE_MAX_CHARS: usize = 500;

/// Renames a conversation locally and triggers a conversation rename on the server.
///
/// Renaming is only exposed for open conversations, so the conversation is expected
/// to already be loaded in the history model.
pub(crate) fn rename_conversation<T: View>(
    conversation_id: AIConversationId,
    title: String,
    ctx: &mut ViewContext<T>,
) {
    let title = match validate_conversation_title(title, ctx) {
        Ok(title) => title,
        Err(message) => {
            let window_id = ctx.window_id();
            ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                toast_stack.add_ephemeral_toast(DismissibleToast::error(message), window_id, ctx);
            });
            return;
        }
    };
    if BlocklistAIHistoryModel::as_ref(ctx)
        .conversation(&conversation_id)
        .is_some_and(|conversation| conversation.is_empty())
    {
        let window_id = ctx.window_id();
        ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
            toast_stack.add_ephemeral_toast(
                DismissibleToast::error(localization::text_for_app(
                    ctx,
                    "ai.conversation_rename.error.empty_conversation",
                )),
                window_id,
                ctx,
            );
        });
        return;
    }
    if conversation_already_has_title(conversation_id, &title, ctx) {
        return;
    }

    let history = BlocklistAIHistoryModel::handle(ctx);
    let server_conversation_id = match history.update(ctx, |history, ctx| {
        history.begin_conversation_rename(conversation_id, title.clone(), ctx)
    }) {
        Ok(server_conversation_id) => server_conversation_id,
        Err(err) => {
            let key = match err {
                BeginConversationRenameError::MissingServerConversationToken => {
                    "ai.conversation_rename.error.not_synced"
                }
                BeginConversationRenameError::RenameInProgress => {
                    "ai.conversation_rename.error.rename_in_progress"
                }
                BeginConversationRenameError::ConversationNotFound => {
                    "ai.conversation_rename.error.not_found"
                }
                BeginConversationRenameError::ConversationNotReady => {
                    "ai.conversation_rename.error.not_ready"
                }
            };
            let window_id = ctx.window_id();
            ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                toast_stack.add_ephemeral_toast(
                    DismissibleToast::error(localization::text_for_app(ctx, key)),
                    window_id,
                    ctx,
                );
            });
            return;
        }
    };

    let server_api = ServerApiProvider::as_ref(ctx).get_ai_client();
    ctx.spawn(
        async move {
            server_api
                .rename_conversation(server_conversation_id, title)
                .await
        },
        move |_, result, ctx| {
            let window_id = ctx.window_id();
            match result {
                Ok(response) => {
                    let title = response.title;
                    BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                        history.complete_conversation_rename(conversation_id, title.clone(), ctx);
                    });
                    ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                        toast_stack.add_ephemeral_toast(
                            DismissibleToast::success(localization::text_for_app_with_args(
                                ctx,
                                "ai.conversation_rename.toast.success",
                                &[("title", title.as_str())],
                            )),
                            window_id,
                            ctx,
                        );
                    });
                }
                Err(e) => {
                    BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                        history.fail_conversation_rename(conversation_id, ctx);
                    });
                    ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                        let error = e.to_string();
                        toast_stack.add_ephemeral_toast(
                            DismissibleToast::error(localization::text_for_app_with_args(
                                ctx,
                                "ai.conversation_rename.toast.failed",
                                &[("error", error.as_str())],
                            )),
                            window_id,
                            ctx,
                        );
                    });
                }
            }
        },
    );
}

/// Returns whether the conversation's current local title already matches `title`,
/// making the rename a no-op.
fn conversation_already_has_title<T: View>(
    conversation_id: AIConversationId,
    title: &str,
    ctx: &ViewContext<T>,
) -> bool {
    BlocklistAIHistoryModel::as_ref(ctx)
        .conversation(&conversation_id)
        .and_then(|conversation| conversation.title())
        .is_some_and(|current_title| current_title == title)
}

/// Trims and validates a requested conversation title, returning a user-facing
/// error message when the title is invalid.
fn validate_conversation_title<T: View>(
    title: String,
    ctx: &ViewContext<T>,
) -> Result<String, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err(localization::text_for_app(
            ctx,
            "ai.conversation_rename.error.empty_title",
        ));
    }

    if title.chars().count() > CONVERSATION_TITLE_MAX_CHARS {
        let max = CONVERSATION_TITLE_MAX_CHARS.to_string();
        return Err(localization::text_for_app_with_args(
            ctx,
            "ai.conversation_rename.error.title_too_long",
            &[("max", max.as_str())],
        ));
    }

    Ok(title.to_owned())
}
