use warp::tui_export::AttachmentType;
use warp_localization::LocaleId;
use warpui_core::elements::MouseStateHandle;
use warpui_core::elements::tui::{TuiElement, TuiFlex, TuiHoverable, TuiText};
use warpui_core::keymap::macros::*;
use warpui_core::keymap::{BindingDescription, EditableBinding};
use warpui_core::{
    AppContext, BlurContext, Entity, FocusContext, ModelHandle, TuiView, TypedActionView,
    ViewContext,
};

use super::model::{TuiAttachmentModel, TuiAttachmentModelEvent, TuiAttachmentPasteDisposition};
use crate::keybindings::TUI_BINDING_GROUP;
use crate::localization;
use crate::tui_builder::TuiUiBuilder;

pub(crate) const FOCUS_ATTACHMENTS_BINDING_NAME: &str = "tui:session:focus_attachments";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TuiAttachmentBarEvent {
    AbortInputDetection,
    RequestInputDetection,
    RestorePastedText(String),
    ShowHint(String),
    ReturnFocus,
}

fn binding_description(fallback: &'static str, key: &'static str) -> BindingDescription {
    BindingDescription::new(fallback).with_dynamic_override(move |_| {
        Some(attachment_binding_description_for_locale(
            localization::current_locale(),
            key,
        ))
    })
}

fn attachment_binding_description_for_locale(locale: LocaleId, key: &str) -> String {
    localization::text_for_locale(locale, key)
}

fn attachment_control(
    text: &'static str,
    mouse: MouseStateHandle,
    action: TuiAttachmentBarAction,
    ctx: &AppContext,
) -> Box<dyn TuiElement> {
    let builder = TuiUiBuilder::from_app(ctx);
    let style = if mouse.lock().is_ok_and(|state| state.is_hovered()) {
        builder.primary_text_style()
    } else {
        builder.muted_text_style()
    };
    TuiHoverable::new(
        mouse,
        TuiText::new(text).with_style(style).truncate().finish(),
    )
    .on_click(move |event_ctx, _| {
        event_ctx.dispatch_typed_action(action.clone());
    })
    .finish()
}

#[derive(Clone, Debug)]
pub(crate) enum TuiAttachmentBarAction {
    Next,
    Previous,
    RemoveSelected,
    ReturnFocus,
}

pub(crate) struct TuiAttachmentBar {
    model: ModelHandle<TuiAttachmentModel>,
    focused: bool,
    previous_mouse: MouseStateHandle,
    next_mouse: MouseStateHandle,
    remove_mouse: MouseStateHandle,
}

pub(crate) fn init(app: &mut AppContext) {
    let context = id!(TuiAttachmentBar::ui_name());
    app.register_editable_bindings([
        EditableBinding::new(
            "tui:attachments:next",
            binding_description("Select the next attachment", "tui.attachments.binding.next"),
            TuiAttachmentBarAction::Next,
        )
        .with_context_predicate(context.clone())
        .with_group(TUI_BINDING_GROUP)
        .with_key_binding("tab"),
        EditableBinding::new(
            "tui:attachments:next",
            binding_description("Select the next attachment", "tui.attachments.binding.next"),
            TuiAttachmentBarAction::Next,
        )
        .with_context_predicate(context.clone())
        .with_group(TUI_BINDING_GROUP)
        .with_key_binding("right"),
        EditableBinding::new(
            "tui:attachments:previous",
            binding_description(
                "Select the previous attachment",
                "tui.attachments.binding.previous",
            ),
            TuiAttachmentBarAction::Previous,
        )
        .with_context_predicate(context.clone())
        .with_group(TUI_BINDING_GROUP)
        .with_key_binding("shift-tab"),
        EditableBinding::new(
            "tui:attachments:previous",
            binding_description(
                "Select the previous attachment",
                "tui.attachments.binding.previous",
            ),
            TuiAttachmentBarAction::Previous,
        )
        .with_context_predicate(context.clone())
        .with_group(TUI_BINDING_GROUP)
        .with_key_binding("left"),
        EditableBinding::new(
            "tui:attachments:remove",
            binding_description(
                "Remove the selected attachment",
                "tui.attachments.binding.remove",
            ),
            TuiAttachmentBarAction::RemoveSelected,
        )
        .with_context_predicate(context.clone())
        .with_group(TUI_BINDING_GROUP)
        .with_key_binding("backspace"),
        EditableBinding::new(
            "tui:attachments:remove",
            binding_description(
                "Remove the selected attachment",
                "tui.attachments.binding.remove",
            ),
            TuiAttachmentBarAction::RemoveSelected,
        )
        .with_context_predicate(context.clone())
        .with_group(TUI_BINDING_GROUP)
        .with_key_binding("delete"),
        EditableBinding::new(
            "tui:attachments:return_focus",
            binding_description(
                "Return focus to the input",
                "tui.attachments.binding.return_focus",
            ),
            TuiAttachmentBarAction::ReturnFocus,
        )
        .with_context_predicate(context.clone())
        .with_group(TUI_BINDING_GROUP)
        .with_key_binding("escape"),
        EditableBinding::new(
            "tui:attachments:return_focus",
            binding_description(
                "Return focus to the input",
                "tui.attachments.binding.return_focus",
            ),
            TuiAttachmentBarAction::ReturnFocus,
        )
        .with_context_predicate(context)
        .with_group(TUI_BINDING_GROUP)
        .with_key_binding("enter"),
    ]);
}

impl TuiAttachmentBar {
    pub(crate) fn new(model: ModelHandle<TuiAttachmentModel>, ctx: &mut ViewContext<Self>) -> Self {
        let model_for_subscription = model.clone();
        ctx.subscribe_to_model(&model, move |view, _, event, ctx| match event {
            TuiAttachmentModelEvent::Updated => {
                // TUI notifications invalidate the whole window, including the
                // parent that conditionally renders this attachment bar.
                if view.focused && !model_for_subscription.as_ref(ctx).should_render(ctx) {
                    ctx.emit(TuiAttachmentBarEvent::ReturnFocus);
                }
                ctx.notify();
            }
            TuiAttachmentModelEvent::AbortInputDetection => {
                ctx.emit(TuiAttachmentBarEvent::AbortInputDetection);
            }
            TuiAttachmentModelEvent::RequestInputDetection => {
                ctx.emit(TuiAttachmentBarEvent::RequestInputDetection);
            }
            TuiAttachmentModelEvent::RestorePastedText(text) => {
                ctx.emit(TuiAttachmentBarEvent::RestorePastedText(text.clone()));
            }
            TuiAttachmentModelEvent::ShowHint(text) => {
                ctx.emit(TuiAttachmentBarEvent::ShowHint(text.clone()));
            }
        });
        Self {
            model,
            focused: false,
            previous_mouse: MouseStateHandle::default(),
            next_mouse: MouseStateHandle::default(),
            remove_mouse: MouseStateHandle::default(),
        }
    }

    pub(crate) fn should_render(&self, ctx: &AppContext) -> bool {
        self.model.as_ref(ctx).should_render(ctx)
    }

    pub(crate) fn try_attach_paste(
        &mut self,
        text: String,
        ctx: &mut ViewContext<Self>,
    ) -> TuiAttachmentPasteDisposition {
        self.model
            .update(ctx, |model, ctx| model.try_attach_paste(text, ctx))
    }

    pub(crate) fn paste_image_from_clipboard(&mut self, ctx: &mut ViewContext<Self>) {
        self.model
            .update(ctx, |model, ctx| model.paste_image_from_clipboard(ctx));
    }

    pub(crate) fn remove_selected(&mut self, ctx: &mut ViewContext<Self>) {
        self.model
            .update(ctx, |model, ctx| model.remove_selected(ctx));
    }
}

fn render_attachment_snapshot(
    snapshot: super::model::TuiAttachmentSnapshot,
    focused: bool,
    previous_mouse: MouseStateHandle,
    next_mouse: MouseStateHandle,
    remove_mouse: MouseStateHandle,
    ctx: &AppContext,
) -> Box<dyn TuiElement> {
    render_attachment_snapshot_for_locale(
        snapshot,
        focused,
        previous_mouse,
        next_mouse,
        remove_mouse,
        localization::current_locale(),
        ctx,
    )
}

fn render_attachment_snapshot_for_locale(
    snapshot: super::model::TuiAttachmentSnapshot,
    focused: bool,
    previous_mouse: MouseStateHandle,
    next_mouse: MouseStateHandle,
    remove_mouse: MouseStateHandle,
    locale: LocaleId,
    ctx: &AppContext,
) -> Box<dyn TuiElement> {
    let builder = TuiUiBuilder::from_app(ctx);
    let Some(selected) = snapshot.selected else {
        return TuiText::new(localization::text_for_locale(
            locale,
            "tui.attachments.loading_image",
        ))
        .with_style(builder.muted_text_style())
        .truncate()
        .finish();
    };
    let kind = match selected.attachment_type {
        AttachmentType::Image => {
            localization::text_for_locale(locale, "tui.attachments.kind.image")
        }
        AttachmentType::File => localization::text_for_locale(locale, "tui.attachments.kind.file"),
    };
    let mut row = TuiFlex::row();
    if snapshot.count > 1 {
        row = row
            .child(attachment_control(
                "< ",
                previous_mouse,
                TuiAttachmentBarAction::Previous,
                ctx,
            ))
            .child(
                TuiText::new(" ")
                    .with_style(builder.muted_text_style())
                    .finish(),
            );
    }
    row = row
        .child(
            TuiText::new(format!("{kind} "))
                .with_style(builder.accent_text_style())
                .truncate()
                .finish(),
        )
        .flex_child(
            TuiText::new(selected.file_name)
                .with_style(if snapshot.selected_is_processing {
                    builder.muted_text_style()
                } else if focused {
                    builder.primary_text_style()
                } else {
                    builder.muted_text_style()
                })
                .truncate()
                .finish(),
        )
        .child(
            TuiText::new(format!(
                "  {}/{}  ",
                snapshot.position.unwrap_or(1),
                snapshot.count
            ))
            .with_style(builder.muted_text_style())
            .truncate()
            .finish(),
        )
        .child(attachment_control(
            "x",
            remove_mouse,
            TuiAttachmentBarAction::RemoveSelected,
            ctx,
        ));
    if snapshot.count > 1 {
        row = row
            .child(
                TuiText::new(" ")
                    .with_style(builder.muted_text_style())
                    .finish(),
            )
            .child(attachment_control(
                " >",
                next_mouse,
                TuiAttachmentBarAction::Next,
                ctx,
            ));
    }
    if snapshot.is_processing {
        row = row.child(
            TuiText::new(format!(
                " - {}",
                localization::text_for_locale(locale, "tui.attachments.loading")
            ))
            .with_style(builder.dim_text_style())
            .truncate()
            .finish(),
        );
    }
    row.finish()
}

impl Entity for TuiAttachmentBar {
    type Event = TuiAttachmentBarEvent;
}

impl TuiView for TuiAttachmentBar {
    fn ui_name() -> &'static str {
        "TuiAttachmentBar"
    }

    fn render(&self, ctx: &AppContext) -> Box<dyn TuiElement> {
        render_attachment_snapshot(
            self.model.as_ref(ctx).snapshot(ctx),
            self.focused,
            self.previous_mouse.clone(),
            self.next_mouse.clone(),
            self.remove_mouse.clone(),
            ctx,
        )
    }

    fn on_focus(&mut self, focus_ctx: &FocusContext, ctx: &mut ViewContext<Self>) {
        if focus_ctx.is_self_focused() {
            self.focused = true;
            ctx.notify();
        }
    }

    fn on_blur(&mut self, blur_ctx: &BlurContext, ctx: &mut ViewContext<Self>) {
        if blur_ctx.is_self_blurred() {
            self.focused = false;
            ctx.notify();
        }
    }
}

impl TypedActionView for TuiAttachmentBar {
    type Action = TuiAttachmentBarAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            TuiAttachmentBarAction::Next => {
                self.model.update(ctx, |model, ctx| model.select_next(ctx));
            }
            TuiAttachmentBarAction::Previous => {
                self.model
                    .update(ctx, |model, ctx| model.select_previous(ctx));
            }
            TuiAttachmentBarAction::RemoveSelected => {
                self.model
                    .update(ctx, |model, ctx| model.remove_selected(ctx));
            }
            TuiAttachmentBarAction::ReturnFocus => {
                ctx.emit(TuiAttachmentBarEvent::ReturnFocus);
            }
        }
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
