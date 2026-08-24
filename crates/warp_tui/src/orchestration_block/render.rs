//! Element construction for the orchestration card.

use warp::tui_export::{
    AIActionStatus, AuthSecretSelection, Harness, HarnessAvailabilityModel,
    ORCHESTRATION_WARP_WORKER_HOST, OptionSnapshot, RunAgentsExecutionMode,
    empty_env_recommendation_message, should_show_auth_secret_picker,
};
use warp_core::features::FeatureFlag;
use warpui::SingletonEntity;
use warpui_core::AppContext;
use warpui_core::elements::CrossAxisAlignment;
use warpui_core::elements::tui::{
    Modifier, TuiChildView, TuiContainer, TuiElement, TuiFlex, TuiParentElement, TuiText,
};

use super::configuration::ConfigPage;
use super::{CardMode, TuiOrchestrationBlock};
use crate::agent_block_sections::render_fallback_tool_call_section;
use crate::localization;
use crate::orchestrated_agent_identity_styling::{AgentIdentity, assign_agent_identity_indices};
use crate::tui_builder::TuiUiBuilder;

impl TuiOrchestrationBlock {
    /// Returns deterministic identities for the proposed agents.
    fn agent_identities(&self) -> Vec<&AgentIdentity> {
        let names = self
            .request_fields
            .agent_run_configs
            .iter()
            .map(|config| config.name.as_str());
        assign_agent_identity_indices(names, self.identity_palette.len())
            .into_iter()
            .filter_map(|index| self.identity_palette.get(index))
            .collect()
    }

    /// Returns the harness display label for the current selection.
    fn harness_label(&self, ctx: &AppContext) -> String {
        match Harness::parse_orchestration_harness(
            &self
                .orchestration_edit_state
                .orchestration_config_state
                .harness_type,
        ) {
            Some(harness) => HarnessAvailabilityModel::as_ref(ctx)
                .display_name_for(harness)
                .to_string(),
            None => "Warp".to_string(),
        }
    }

    /// Resolves an id to its display label, falling back to the id.
    fn label_for_id(snapshot: &OptionSnapshot, id: &str, fallback: &str) -> String {
        snapshot
            .rows
            .iter()
            .find(|row| row.id == id)
            .map(|row| row.label.clone())
            .unwrap_or_else(|| {
                if id.is_empty() {
                    fallback.to_string()
                } else {
                    id.to_string()
                }
            })
    }

    /// Renders every proposed agent with its stable identity.
    fn render_agent_identity_line(&self, builder: &TuiUiBuilder) -> Box<dyn TuiElement> {
        let mut spans: Vec<(String, _)> = Vec::new();
        for (index, (config, identity)) in self
            .request_fields
            .agent_run_configs
            .iter()
            .zip(self.agent_identities())
            .enumerate()
        {
            if index > 0 {
                spans.push(("  •  ".to_string(), builder.muted_text_style()));
            }
            spans.push((format!("{} ", identity.glyph), identity.style));
            spans.push((
                config.name.clone(),
                identity.style.add_modifier(Modifier::BOLD),
            ));
        }
        TuiText::from_spans(spans).finish()
    }

    /// Renders the inline run-wide configuration values.
    fn render_metadata_line(
        &self,
        app: &AppContext,
        builder: &TuiUiBuilder,
    ) -> Box<dyn TuiElement> {
        let state = &self.orchestration_edit_state.orchestration_config_state;
        let is_remote = state.execution_mode.is_remote();
        let mut entries = vec![(
            localization::text("tui.orchestration.label.location"),
            localization::text(if is_remote {
                "agent.orchestration.controls.cloud"
            } else {
                "agent.orchestration.controls.local"
            }),
        )];
        entries.push((
            localization::text("tui.orchestration.label.harness"),
            self.harness_label(app),
        ));
        if is_remote {
            if should_show_auth_secret_picker(state) {
                let api_key = match &state.auth_secret_selection {
                    AuthSecretSelection::Named(name) => name.clone(),
                    AuthSecretSelection::Inherit => {
                        localization::text("agent.orchestration.controls.auth_secret_inherit")
                    }
                    AuthSecretSelection::Unset | AuthSecretSelection::CreatingNew => {
                        localization::text("tui.orchestration.select_api_key")
                    }
                };
                entries.push((
                    localization::text("tui.orchestration.label.api_key"),
                    api_key,
                ));
            }
            let host = match &state.execution_mode {
                RunAgentsExecutionMode::Remote { worker_host, .. }
                    if !worker_host.trim().is_empty() =>
                {
                    worker_host.clone()
                }
                RunAgentsExecutionMode::Remote { .. } | RunAgentsExecutionMode::Local => {
                    ORCHESTRATION_WARP_WORKER_HOST.to_string()
                }
            };
            entries.push((localization::text("tui.orchestration.label.host"), host));
            let environment_id = match &state.execution_mode {
                RunAgentsExecutionMode::Remote { environment_id, .. } => environment_id.clone(),
                RunAgentsExecutionMode::Local => String::new(),
            };
            entries.push((
                localization::text("tui.orchestration.label.environment"),
                Self::label_for_id(
                    &self.snapshot_for_page(ConfigPage::Environment, app),
                    &environment_id,
                    &localization::text("tui.orchestration.empty_environment"),
                ),
            ));
        }
        entries.push((
            localization::text("tui.orchestration.label.model"),
            Self::label_for_id(
                &self.snapshot_for_page(ConfigPage::Model, app),
                &state.model_id,
                &localization::text("agent.orchestration.controls.default_model"),
            ),
        ));

        let mut spans: Vec<(String, _)> = Vec::new();
        for (index, (label, value)) in entries.into_iter().enumerate() {
            if index > 0 {
                spans.push(("  •  ".to_string(), builder.muted_text_style()));
            }
            spans.push((format!("{label}: "), builder.primary_text_style()));
            spans.push((value, builder.orchestration_selected_value_style()));
        }
        TuiText::from_spans(spans).finish()
    }

    /// Renders the acceptance card body.
    fn render_acceptance(&self, app: &AppContext, builder: &TuiUiBuilder) -> Box<dyn TuiElement> {
        let state = &self.orchestration_edit_state.orchestration_config_state;
        let mut column = TuiFlex::column();

        column.add_child(
            TuiText::new(localization::text_with_args(
                "tui.orchestration.agents",
                &[(
                    "count",
                    &self.request_fields.agent_run_configs.len().to_string(),
                )],
            ))
            .with_style(builder.primary_text_style())
            .truncate()
            .finish(),
        );
        column.add_child(self.render_agent_identity_line(builder));
        // Multi-level orchestration: the server may grant launched children
        // the run_agents tool, so tell the approver up front. Same gate as
        // the GUI card; copy per design review (trailing period, no glyph).
        if FeatureFlag::MultiLevelOrchestration.is_enabled() {
            column.add_child(
                TuiText::new(localization::text(
                    "tui.orchestration.child.may_start_children",
                ))
                .with_style(builder.muted_text_style())
                .truncate()
                .finish(),
            );
        }
        column.add_child(TuiText::new(" ").finish());
        column.add_child(self.render_metadata_line(app, builder));

        if let Some(error) = &self.accept_error {
            column.add_child(
                TuiText::new(error.clone())
                    .with_style(builder.error_text_style())
                    .finish(),
            );
        } else if let Some(message) = empty_env_recommendation_message(&state.execution_mode, app) {
            column.add_child(
                TuiText::new(message)
                    .with_style(builder.attention_glyph_style())
                    .truncate()
                    .finish(),
            );
        }

        column.finish()
    }

    /// Renders the title shared by acceptance and configuration.
    fn render_title(&self, builder: &TuiUiBuilder) -> Box<dyn TuiElement> {
        TuiText::from_spans([
            ("■ ".to_string(), builder.attention_glyph_style()),
            (
                localization::text("tui.orchestration.title"),
                builder.primary_text_style(),
            ),
        ])
        .finish()
    }

    /// Renders the active selector page.
    fn render_configuring(&self) -> Box<dyn TuiElement> {
        TuiChildView::new(&self.selector).finish()
    }

    /// Renders the key hints below the tinted card.
    fn render_footer(&self, builder: &TuiUiBuilder) -> Box<dyn TuiElement> {
        let spans = match self.mode {
            CardMode::Acceptance => vec![
                (
                    localization::text("tui.orchestration.key.accept"),
                    builder.primary_text_style(),
                ),
                (
                    localization::text("tui.orchestration.footer.accept"),
                    builder.muted_text_style(),
                ),
                (
                    localization::text("tui.orchestration.key.edit"),
                    builder.primary_text_style(),
                ),
                (
                    localization::text("tui.orchestration.footer.edit"),
                    builder.muted_text_style(),
                ),
                (
                    localization::text("tui.orchestration.key.reject"),
                    builder.primary_text_style(),
                ),
                (
                    localization::text("tui.orchestration.footer.reject"),
                    builder.muted_text_style(),
                ),
            ],
            CardMode::Configuring { .. } => vec![
                (
                    localization::text("tui.orchestration.key.accept"),
                    builder.primary_text_style(),
                ),
                (
                    localization::text("tui.orchestration.footer.accept"),
                    builder.muted_text_style(),
                ),
                (
                    localization::text("tui.orchestration.key.navigate"),
                    builder.primary_text_style(),
                ),
                (
                    localization::text("tui.orchestration.footer.navigate"),
                    builder.muted_text_style(),
                ),
                (
                    localization::text("tui.orchestration.key.go_back"),
                    builder.primary_text_style(),
                ),
                (
                    localization::text("tui.orchestration.footer.go_back"),
                    builder.muted_text_style(),
                ),
            ],
        };
        TuiText::from_spans(spans).finish()
    }
}

/// Renders the orchestration block in interactive or fallback form.
pub(super) fn render(block: &TuiOrchestrationBlock, app: &AppContext) -> Box<dyn TuiElement> {
    let status = block.controller.action_status(&block.action_id, app);
    let interactive = !block.is_restored
        && block.spawning.is_none()
        && matches!(status, Some(AIActionStatus::Blocked));
    if !interactive {
        return render_fallback_tool_call_section(&block.action, status.as_ref(), false, None, app);
    }

    let builder = TuiUiBuilder::from_app(app);
    let header = TuiContainer::new(block.render_title(&builder))
        .with_background(builder.orchestration_header_background())
        .with_padding_x(1)
        .finish();
    let body = match block.mode {
        CardMode::Acceptance => block.render_acceptance(app, &builder),
        CardMode::Configuring { .. } => block.render_configuring(),
    };
    let body = TuiContainer::new(body)
        .with_background(builder.orchestration_surface_background())
        .with_padding_x(3)
        .with_padding_y(1)
        .finish();
    TuiFlex::column()
        .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
        .child(header)
        .child(body)
        .child(
            TuiContainer::new(block.render_footer(&builder))
                .with_padding_top(1)
                .finish(),
        )
        .finish()
}
