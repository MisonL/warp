//! Small presentation helpers for the `warp-tui` front-end's TUI views.
use std::time::Duration;

use warpui_core::elements::animation::AnimationClock;
use warpui_core::elements::tui::{
    Modifier, TuiConstrainedBox, TuiElement, TuiFlex, TuiStyle, TuiText,
};
use warpui_core::elements::CrossAxisAlignment;
use warpui_core::AppContext;

use crate::localization;
use crate::tui_builder::TuiUiBuilder;
use crate::warping_indicator::render_spinner;

/// Abbreviates a leading home-directory prefix of `path` to `~`.
pub(crate) fn abbreviate_home_prefix(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home = home.to_string_lossy();
        if let Some(rest) = path.strip_prefix(&*home) {
            if rest.is_empty() || rest.starts_with('/') || rest.starts_with('\\') {
                return format!("~{rest}");
            }
        }
    }
    path.to_owned()
}

/// Compacts a path for the one-line session footer while preserving its root
/// (or first relative component) and basename.
pub(crate) fn compact_footer_path(path: &str) -> String {
    let path = abbreviate_home_prefix(path);
    let separator = if path.contains('\\') && !path.contains('/') {
        '\\'
    } else {
        '/'
    };
    let components: Vec<_> = path
        .split(separator)
        .filter(|component| !component.is_empty())
        .collect();
    if components.len() <= 2 {
        return path;
    }

    let last = components
        .last()
        .expect("path has more than two components");
    if path.starts_with(separator) {
        format!("{separator}…{separator}{last}")
    } else {
        format!(
            "{}{separator}…{separator}{last}",
            components.first().expect("path has components")
        )
    }
}

/// Placeholder shown while a requested conversation is restored.
pub(crate) fn conversation_restoring(app: &AppContext) -> Box<dyn TuiElement> {
    let muted = TuiUiBuilder::from_app(app).muted_text_style();
    let label = localization::text("tui.session.loading");
    centered_in_viewport(
        TuiConstrainedBox::new(
            TuiFlex::column()
                .child(render_spinner(
                    AnimationClock::starting_at(Duration::ZERO),
                    muted,
                ))
                .child(
                    TuiText::new(label.clone())
                        .with_style(muted)
                        .truncate()
                        .finish(),
                )
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .finish(),
        )
        .with_max_cols(label.len() as u16)
        .finish(),
    )
}

/// Placeholder shown when a requested conversation cannot be restored.
pub(crate) fn conversation_restore_failed(message: &str) -> Box<dyn TuiElement> {
    let dim = TuiStyle::default().add_modifier(Modifier::DIM);
    let error = localization::text_with_args("tui.session.restore_failed", &[("message", message)]);
    vertically_centered(
        TuiFlex::column()
            .child(TuiText::new(error).truncate().finish())
            .child(
                TuiText::new(localization::text("tui.common.press_ctrl_c_to_exit"))
                    .with_style(dim)
                    .truncate()
                    .finish(),
            ),
    )
}

/// Vertically centers `content` with its existing horizontal alignment.
fn vertically_centered(content: TuiFlex) -> Box<dyn TuiElement> {
    TuiFlex::column()
        .flex_child(TuiFlex::column().finish())
        .child(content.finish())
        .flex_child(TuiFlex::column().finish())
        .finish()
}

/// Centers `content` horizontally and vertically within the viewport.
fn centered_in_viewport(content: Box<dyn TuiElement>) -> Box<dyn TuiElement> {
    let centered_row = TuiFlex::row()
        .flex_child(TuiFlex::row().finish())
        .child(content)
        .flex_child(TuiFlex::row().finish());
    TuiFlex::column()
        .flex_child(TuiFlex::column().finish())
        .child(centered_row.finish())
        .flex_child(TuiFlex::column().finish())
        .finish()
}

/// Placeholder shown while the user completes device-authorization login. The
/// verification URL/code are surfaced once known (the browser also auto-opens).
pub(crate) fn login_placeholder(
    verification_uri: Option<&str>,
    user_code: Option<&str>,
) -> Box<dyn TuiElement> {
    let dim = TuiStyle::default().add_modifier(Modifier::DIM);
    let mut content = TuiFlex::column().child(
        TuiText::new(localization::text("tui.auth.sign_in"))
            .truncate()
            .finish(),
    );
    match (verification_uri, user_code) {
        (Some(uri), Some(code)) => {
            content = content
                .child(
                    TuiText::new(localization::text_with_args(
                        "tui.auth.open_browser",
                        &[("uri", uri)],
                    ))
                    .with_style(dim)
                    .truncate()
                    .finish(),
                )
                .child(
                    TuiText::new(localization::text_with_args(
                        "tui.auth.enter_code",
                        &[("code", code)],
                    ))
                    .with_style(dim)
                    .truncate()
                    .finish(),
                );
        }
        (Some(uri), None) => {
            content = content.child(
                TuiText::new(localization::text_with_args(
                    "tui.auth.open_browser",
                    &[("uri", uri)],
                ))
                .with_style(dim)
                .truncate()
                .finish(),
            );
        }
        _ => {
            content = content.child(
                TuiText::new(localization::text("tui.auth.opening_browser"))
                    .with_style(dim)
                    .truncate()
                    .finish(),
            );
        }
    }
    vertically_centered(content)
}

/// Placeholder shown between login completion and terminal session creation.
pub(crate) fn terminal_starting() -> Box<dyn TuiElement> {
    let dim = TuiStyle::default().add_modifier(Modifier::DIM);
    vertically_centered(
        TuiFlex::column().child(
            TuiText::new(localization::text("tui.session.starting_terminal"))
                .with_style(dim)
                .truncate()
                .finish(),
        ),
    )
}

/// Placeholder shown when login fails; the user can quit with `Ctrl-C`.
pub(crate) fn login_failed(message: &str) -> Box<dyn TuiElement> {
    let dim = TuiStyle::default().add_modifier(Modifier::DIM);
    let content = TuiFlex::column()
        .child(
            TuiText::new(localization::text_with_args(
                "tui.auth.login_failed",
                &[("message", message)],
            ))
            .truncate()
            .finish(),
        )
        .child(
            TuiText::new(localization::text("tui.common.press_ctrl_c_to_exit"))
                .with_style(dim)
                .truncate()
                .finish(),
        );
    vertically_centered(content)
}

#[cfg(test)]
#[path = "ui_tests.rs"]
mod tests;
