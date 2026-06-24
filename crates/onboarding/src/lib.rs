// Onboarding library crate

mod agent_onboarding_view;
pub mod callout;
mod model;
pub mod slides;
pub mod telemetry;

/// The user's intention selected during onboarding slides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnboardingIntention {
    Terminal,
    AgentDrivenDevelopment,
}

impl std::fmt::Display for OnboardingIntention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnboardingIntention::AgentDrivenDevelopment => write!(f, "agent_driven"),
            OnboardingIntention::Terminal => write!(f, "terminal"),
        }
    }
}

pub use callout::{OnboardingCalloutView, OnboardingKeybindings};

pub const AGENT_ONBOARDING_COPY_KEYS: &[&str] = &[
    "onboarding.agent.auth.browser_middle",
    "onboarding.agent.auth.browser_prefix",
    "onboarding.agent.auth.browser_suffix",
    "onboarding.agent.auth.click_here",
    "onboarding.agent.auth.copy_url",
    "onboarding.agent.autonomy",
    "onboarding.agent.autonomy.full.description",
    "onboarding.agent.autonomy.full.title",
    "onboarding.agent.autonomy.none.description",
    "onboarding.agent.autonomy.none.title",
    "onboarding.agent.autonomy.partial.description",
    "onboarding.agent.autonomy.partial.title",
    "onboarding.agent.default_model",
    "onboarding.agent.disable",
    "onboarding.agent.plan_activated",
    "onboarding.agent.recommended",
    "onboarding.agent.subtitle",
    "onboarding.agent.team_workspace.description",
    "onboarding.agent.team_workspace.title",
    "onboarding.agent.title",
    "onboarding.ai_access.add_custom_endpoint",
    "onboarding.ai_access.add_key",
    "onboarding.ai_access.best_value",
    "onboarding.ai_access.byok.description",
    "onboarding.ai_access.byok.title",
    "onboarding.ai_access.choose_plan",
    "onboarding.ai_access.subscription.description",
    "onboarding.ai_access.subscription.title",
    "onboarding.ai_access.subtitle",
    "onboarding.ai_access.title",
    "onboarding.ai_access.tooltip.requires_inference",
    "onboarding.ai_setup.subtitle",
    "onboarding.ai_setup.third_party.description",
    "onboarding.ai_setup.third_party.title",
    "onboarding.ai_setup.title",
    "onboarding.ai_setup.warp_agent.badge",
    "onboarding.ai_setup.warp_agent.description",
    "onboarding.ai_setup.warp_agent.feature.agentic_coding",
    "onboarding.ai_setup.warp_agent.feature.frontier_models",
    "onboarding.ai_setup.warp_agent.feature.model_routing",
    "onboarding.ai_setup.warp_agent.feature.multi_agent",
    "onboarding.ai_setup.warp_agent.title",
    "onboarding.common.back",
    "onboarding.common.get_started",
    "onboarding.common.get_warping",
    "onboarding.common.next",
    "onboarding.common.skip",
    "onboarding.common.enabled",
    "onboarding.common.disabled",
    "onboarding.customize.code_review",
    "onboarding.customize.conversation_history",
    "onboarding.customize.file_explorer",
    "onboarding.customize.global_file_search",
    "onboarding.customize.horizontal",
    "onboarding.customize.subtitle",
    "onboarding.customize.tab_styling",
    "onboarding.customize.title",
    "onboarding.customize.tools_panel",
    "onboarding.customize.vertical",
    "onboarding.customize.warp_drive",
    "onboarding.intention.agent.description",
    "onboarding.intention.agent.title",
    "onboarding.intention.subtitle",
    "onboarding.intention.terminal.badge",
    "onboarding.intention.terminal.description",
    "onboarding.intention.terminal.title",
    "onboarding.intention.title",
    "onboarding.intro.account_prompt",
    "onboarding.intro.log_in",
    "onboarding.intro.subtitle",
    "onboarding.intro.title",
    "onboarding.no_ai.body",
    "onboarding.no_ai.cancel",
    "onboarding.no_ai.confirm",
    "onboarding.no_ai.title",
    "onboarding.project.open_local_folder",
    "onboarding.project.subtitle",
    "onboarding.project.title",
    "onboarding.theme.privacy_link",
    "onboarding.theme.privacy_prefix",
    "onboarding.theme.subtitle",
    "onboarding.theme.sync_with_os",
    "onboarding.theme.title",
    "onboarding.theme.tos_link",
    "onboarding.theme.tos_prefix",
    "onboarding.third_party.cli_agent_toolbar",
    "onboarding.third_party.notifications",
    "onboarding.third_party.subtitle",
    "onboarding.third_party.title",
];

#[derive(Clone, Debug)]
pub struct OnboardingCopy {
    entries: std::collections::BTreeMap<&'static str, String>,
}

impl OnboardingCopy {
    pub fn new(entries: impl IntoIterator<Item = (&'static str, String)>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    pub fn text(&self, key: &'static str) -> &str {
        self.entries.get(key).map(String::as_str).unwrap_or(key)
    }

    pub fn text_owned(&self, key: &'static str) -> String {
        self.text(key).to_owned()
    }
}

/// User-facing names of the AI features enabled when the agent intention is selected.
/// Shared by the intention slide's agent card checklist and the login slide's
/// skip-login confirmation dialog so the two always stay in sync.
pub const AI_FEATURES: &[&str] = &[
    "Warp agents",
    "Oz Cloud Agents Platform",
    "Prompt suggestions",
    "Next command predictions",
    "Full Terminal Use",
    "Codebase Context",
    "Remote Control with Claude Code, Codex, and other agents",
];

pub const AI_FEATURE_COPY_KEYS: &[&str] = &[
    "onboarding.features.ai.warp_agents",
    "onboarding.features.ai.oz_cloud_agents",
    "onboarding.features.ai.prompt_suggestions",
    "onboarding.features.ai.next_command_predictions",
    "onboarding.features.ai.agents_over_ssh",
    "onboarding.features.ai.codebase_context",
    "onboarding.features.ai.remote_control",
];

/// User-facing names of the Warp Drive features enabled when the terminal
/// intention is selected with Warp Drive turned on. Shared by the login slide's
/// skip-login confirmation dialog so the list stays in sync with any future
/// surfaces that need it.
pub const WARP_DRIVE_FEATURES: &[&str] = &["Warp Drive", "Session Sharing"];

pub const WARP_DRIVE_FEATURE_COPY_KEYS: &[&str] = &[
    "onboarding.features.drive.warp_drive",
    "onboarding.features.drive.session_sharing",
];

cfg_if::cfg_if! {
    if #[cfg(feature = "bin")] {
        mod telemetry_provider;
        pub use telemetry_provider::MockTelemetryContextProvider;
    }
}

pub mod components;
mod visuals;

/// The default mode for new sessions, chosen during onboarding.
/// Mapped to `DefaultSessionMode` at the application boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionDefault {
    #[default]
    Agent,
    Terminal,
}

impl std::fmt::Display for SessionDefault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionDefault::Agent => write!(f, "agent"),
            SessionDefault::Terminal => write!(f, "terminal"),
        }
    }
}

pub use agent_onboarding_view::{AgentOnboardingAction, AgentOnboardingEvent, AgentOnboardingView};
pub use model::{OnboardingAuthState, SelectedSettings, UICustomizationSettings};
pub use slides::ProjectOnboardingSettings;
pub use telemetry::OnboardingEvent;

pub fn init(app: &mut warpui_core::AppContext) {
    agent_onboarding_view::init(app);
    callout::init(app);
}
