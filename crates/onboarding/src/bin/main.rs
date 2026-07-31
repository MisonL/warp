#![allow(dead_code)]

use std::borrow::Cow;

use ai::LLMId;
use anyhow::Result;
use onboarding::slides::OnboardingModelInfo;
use onboarding::{
    AgentOnboardingEvent, AgentOnboardingView, MockTelemetryContextProvider, SelectedSettings,
};
use pathfinder_color::ColorU;
use rust_embed::RustEmbed;
use warp_core::ui::appearance::Appearance;
use warp_core::ui::icons::Icon;
use warp_core::ui::theme::{
    AnsiColor, AnsiColors, Details, Fill, Image, TerminalColors, WarpTheme,
};
use warpui_core::assets::asset_cache::AssetSource;
use warpui_core::elements::{
    Container, CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize, ParentElement,
};
use warpui_core::fonts::{Cache, FamilyId, Weight};
use warpui_core::presenter::ChildView;
use warpui_core::ui_components::components::{UiComponent as _, UiComponentStyles};
use warpui_core::{
    AddWindowOptions, AppContext, AssetProvider, Element, Entity, SingletonEntity as _,
    TypedActionView, View, ViewContext, ViewHandle, platform,
};

#[derive(Clone, Copy, RustEmbed)]
#[folder = "../../app/assets"]
pub struct Assets;

pub static ASSETS: Assets = Assets;

impl AssetProvider for Assets {
    fn get(&self, path: &str) -> Result<Cow<'_, [u8]>> {
        <Assets as RustEmbed>::get(path)
            .map(|f| f.data)
            .ok_or_else(|| anyhow::anyhow!(format!("no asset exists at path {path}")))
    }
}

fn main() -> Result<()> {
    // Initialize logging for the onboarding binary.
    warp_logging::init(warp_logging::LogConfig {
        is_cli: false,
        log_destination: None,
        ..Default::default()
    })?;

    let app_builder = warpui::platform::AppBuilder::new(
        platform::AppCallbacks::default(),
        Box::new(ASSETS),
        None,
    );
    let _ = app_builder.run(move |ctx| {
        // Register Appearance singleton so views can access Appearance::handle(ctx).
        ctx.add_singleton_model(|ctx| build_appearance(phenomenon(), ctx));

        // Register telemetry context provider for logging telemetry events.
        ctx.add_singleton_model(MockTelemetryContextProvider::new_context_provider);

        ctx.add_window(AddWindowOptions::default(), |ctx| {
            OnboardingMainView::new(ctx)
        });

        onboarding::init(ctx);
    });

    Ok(())
}

#[derive(Clone, Debug)]
enum OnboardingMainState {
    Onboarding(ViewHandle<AgentOnboardingView>),
    Finished(ViewHandle<FinishedOnboardingView>),
}

struct OnboardingMainView {
    state: OnboardingMainState,
}

impl OnboardingMainView {
    fn new(ctx: &mut ViewContext<Self>) -> Self {
        let themes = [phenomenon(), dark_theme(), light_theme(), adeberry()];
        let default_model_id = LLMId::from("auto");
        let models = vec![
            OnboardingModelInfo {
                id: LLMId::from("auto"),
                title: "Auto".to_string(),
                icon: Icon::Oz,
                is_default: true,
            },
            OnboardingModelInfo {
                id: LLMId::from("claude-sonnet"),
                title: "Claude Sonnet".to_string(),
                icon: Icon::ClaudeLogo,
                is_default: false,
            },
            OnboardingModelInfo {
                id: LLMId::from("gpt-4o"),
                title: "GPT-4o".to_string(),
                icon: Icon::OpenAILogo,
                is_default: false,
            },
        ];
        let onboarding_view = ctx.add_typed_action_view(move |ctx| {
            // agent_modality_enabled is false for demo purposes
            AgentOnboardingView::new(
                themes.clone(),
                true,
                models.clone(),
                default_model_id.clone(),
                false,
                false,
                onboarding::OnboardingAuthState::LoggedOut,
                default_agent_onboarding_copy(),
                ctx,
            )
        });
        onboarding_view.update(ctx, |view, ctx| {
            view.start_onboarding(ctx);
        });
        ctx.subscribe_to_view(&onboarding_view, |me, _view, event, ctx| {
            me.handle_onboarding_event(event, ctx);
        });

        Self {
            state: OnboardingMainState::Onboarding(onboarding_view),
        }
    }

    fn handle_onboarding_event(
        &mut self,
        event: &AgentOnboardingEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            AgentOnboardingEvent::ThemeSelected { theme_name } => {
                let theme = match theme_name.as_str() {
                    "Phenomenon" => phenomenon(),
                    "Dark" => dark_theme(),
                    "Light" => light_theme(),
                    "Adeberry" => adeberry(),
                    _ => return,
                };

                Appearance::handle(ctx).update(ctx, |appearance, ctx| {
                    appearance.set_theme(theme, ctx);
                });
            }
            AgentOnboardingEvent::OnboardingCompleted(selected_settings) => {
                let finished_view = ctx.add_typed_action_view(|_| {
                    FinishedOnboardingView::new(Some(selected_settings.clone()))
                });
                self.state = OnboardingMainState::Finished(finished_view);
                ctx.notify();
            }
            AgentOnboardingEvent::OnboardingSkipped => {
                let finished_view =
                    ctx.add_typed_action_view(|_| FinishedOnboardingView::new(None));
                self.state = OnboardingMainState::Finished(finished_view);
                ctx.notify();
            }
            AgentOnboardingEvent::SyncWithOsToggled { .. }
            | AgentOnboardingEvent::UpgradeRequested
            | AgentOnboardingEvent::UpgradeCopyUrlRequested
            | AgentOnboardingEvent::UpgradePasteTokenFromClipboardRequested
            | AgentOnboardingEvent::LoginFromWelcomeRequested
            | AgentOnboardingEvent::PrivacySettingsFromTerminalThemeSlideRequested
            | AgentOnboardingEvent::AppBecameActive => {
                // No-op in the standalone demo binary
            }
        }
    }
}

fn default_agent_onboarding_copy() -> onboarding::OnboardingCopy {
    let entries = onboarding::AGENT_ONBOARDING_COPY_KEYS
        .iter()
        .chain(onboarding::AI_FEATURE_COPY_KEYS.iter())
        .map(|key| (*key, default_agent_onboarding_text(key).to_owned()));
    onboarding::OnboardingCopy::new(entries)
}

fn default_agent_onboarding_text(key: &str) -> &str {
    match key {
        "onboarding.agent.auth.browser_middle" => " and open the page manually. ",
        "onboarding.agent.auth.browser_prefix" => "If your browser hasn't launched, ",
        "onboarding.agent.auth.browser_suffix" => " to paste your token from the browser.",
        "onboarding.agent.auth.click_here" => "Click here",
        "onboarding.agent.auth.copy_url" => "copy the URL",
        "onboarding.agent.autonomy" => "Autonomy",
        "onboarding.agent.autonomy.full.description" => {
            "Runs commands, writes code, and reads files without asking."
        }
        "onboarding.agent.autonomy.full.title" => "Full",
        "onboarding.agent.autonomy.none.description" => "Takes no actions without your approval.",
        "onboarding.agent.autonomy.none.title" => "None",
        "onboarding.agent.autonomy.partial.description" => {
            "Can plan, read files, and execute low-risk commands. Asks before making any changes or executing sensitive commands."
        }
        "onboarding.agent.autonomy.partial.title" => "Partial",
        "onboarding.agent.default_model" => "Default model",
        "onboarding.agent.disable" => "Disable Warp Agent",
        "onboarding.agent.plan_activated" => "Plan successfully activated!",
        "onboarding.agent.recommended" => "Recommended",
        "onboarding.agent.subtitle" => "Select your Warp Agent's defaults.",
        "onboarding.agent.team_workspace.description" => {
            "Autonomy settings are configured as part of your team workspace."
        }
        "onboarding.agent.team_workspace.title" => "Set by Team Workspace",
        "onboarding.agent.title" => "Customize your Warp Agent",
        "onboarding.ai_access.add_custom_endpoint" => "+ Add custom endpoint",
        "onboarding.ai_access.add_key" => "+ Add key",
        "onboarding.ai_access.best_value" => "Best value",
        "onboarding.ai_access.byok.description" => {
            "Use your own API key or OpenAI-compatible endpoint with Warp for free."
        }
        "onboarding.ai_access.byok.title" => "Use my own key or endpoint",
        "onboarding.ai_access.choose_plan" => "Choose plan",
        "onboarding.ai_access.set_up_later.description" => {
            "Explore Warp's built-in AI features before committing to a plan, or bring your own inference."
        }
        "onboarding.ai_access.set_up_later.title" => "Set up later",
        "onboarding.ai_access.subscription.description" => {
            "Starting at $18 / mo, available with monthly or annual plans. Includes base credits, frontier models, cloud agents, collaboration, and more."
        }
        "onboarding.ai_access.subscription.title" => "Subscription",
        "onboarding.ai_access.subtitle" => {
            "Save with a recurring plan, or explore Warp's AI before committing."
        }
        "onboarding.ai_access.title" => "Get AI access",
        "onboarding.ai_access.tooltip.requires_inference" => {
            "Warp Agent requires a subscription or inference supplied by you"
        }
        "onboarding.ai_setup.subtitle" => {
            "Choose if you'd like to use Warp Agent or third party agents."
        }
        "onboarding.ai_setup.third_party.description" => {
            "Use agents like Claude Code, Codex, and Gemini."
        }
        "onboarding.ai_setup.third_party.title" => "Use third party agents",
        "onboarding.ai_setup.title" => "Choose your AI setup",
        "onboarding.ai_setup.warp_agent.badge" => "Access more models",
        "onboarding.ai_setup.warp_agent.description" => {
            "State of the art agent harness deeply integrated into the terminal."
        }
        "onboarding.ai_setup.warp_agent.feature.agentic_coding" => {
            "Best harness for terminal tasks and agentic coding"
        }
        "onboarding.ai_setup.warp_agent.feature.frontier_models" => {
            "Frontier models from OpenAI, Anthropic, and Google"
        }
        "onboarding.ai_setup.warp_agent.feature.model_routing" => {
            "Model routing across frontier and open-weight models"
        }
        "onboarding.ai_setup.warp_agent.feature.multi_agent" => "Multi-agent orchestration",
        "onboarding.ai_setup.warp_agent.title" => "Use Warp Agent",
        "onboarding.common.back" => "Back",
        "onboarding.common.disabled" => "Disabled",
        "onboarding.common.enabled" => "Enabled",
        "onboarding.common.get_started" => "Get started",
        "onboarding.common.get_warping" => "Get Warping",
        "onboarding.common.next" => "Next",
        "onboarding.common.skip" => "Skip",
        "onboarding.customize.code_review" => "Code review",
        "onboarding.customize.conversation_history" => "Conversation history",
        "onboarding.customize.file_explorer" => "File explorer",
        "onboarding.customize.global_file_search" => "Global file search",
        "onboarding.customize.horizontal" => "Horizontal",
        "onboarding.customize.subtitle" => "Tailor your features and UI to your working style.",
        "onboarding.customize.tab_styling" => "Tab styling",
        "onboarding.customize.title" => "Customize your Warp",
        "onboarding.customize.tools_panel" => "Tools panel",
        "onboarding.customize.vertical" => "Vertical",
        "onboarding.customize.warp_drive" => "Warp Drive",
        "onboarding.features.ai.agents_over_ssh" => "Agents over SSH",
        "onboarding.features.ai.cloud_agents" => "Hand off agent work to cloud agents",
        "onboarding.features.ai.code_review" => {
            "Review code diffs and send comments directly to agents"
        }
        "onboarding.features.ai.codebase_context" => "Codebase context",
        "onboarding.features.ai.frontier_models" => {
            "Use frontier and open-weight models with Warp Agent"
        }
        "onboarding.features.ai.long_running_commands" => {
            "Agentic control of long-running commands and TUIs"
        }
        "onboarding.features.ai.next_command_predictions" => "Next command predictions",
        "onboarding.features.ai.oz_cloud_agents" => "Oz cloud agents platform",
        "onboarding.features.ai.prompt_suggestions" => "Prompt suggestions",
        "onboarding.features.ai.remote_control" => {
            "Remote control for Claude Code, Codex, and other agents"
        }
        "onboarding.features.ai.terminal_error_fixes" => {
            "Automatically diagnose and fix terminal errors"
        }
        "onboarding.features.ai.warp_agents" => "Warp agents",
        "onboarding.intention.agent.description" => {
            "Get AI features to accelerate terminal and agent-driven workflows:"
        }
        "onboarding.intention.agent.title" => "Build faster with agents",
        "onboarding.intention.subtitle" => "How do you want to work?",
        "onboarding.intention.terminal.badge" => "No AI features",
        "onboarding.intention.terminal.description" => {
            "A modern terminal optimized for speed, context, and control without AI."
        }
        "onboarding.intention.terminal.title" => "Just use the terminal",
        "onboarding.intention.title" => "Welcome to Warp",
        "onboarding.intro.account_prompt" => "Already have an account? ",
        "onboarding.intro.log_in" => "Log in",
        "onboarding.intro.subtitle" => "A modern terminal with state of the art agents built in.",
        "onboarding.intro.title" => "Welcome to Warp",
        "onboarding.no_ai.body" => {
            "Without AI, you'll still get Warp's terminal experience, but you'll miss our agentic features like automatic fixes for terminal errors."
        }
        "onboarding.no_ai.cancel" => "Give me AI features",
        "onboarding.no_ai.confirm" => "I don't want AI",
        "onboarding.no_ai.title" => "Are you sure you don't want AI?",
        "onboarding.project.open_local_folder" => "Open local folder",
        "onboarding.project.subtitle" => "Set up a project to optimize it for coding in Warp.",
        "onboarding.project.title" => "Open a project",
        "onboarding.theme.privacy_link" => "Privacy Settings",
        "onboarding.theme.privacy_prefix" => {
            "If you'd like to opt out of analytics, you can adjust your "
        }
        "onboarding.theme.subtitle" => "Click or use arrow keys to select, Enter to confirm.",
        "onboarding.theme.sync_with_os" => "Sync light/dark theme with OS",
        "onboarding.theme.title" => "Choose a theme",
        "onboarding.theme.tos_link" => "Terms of Service",
        "onboarding.theme.tos_prefix" => "By continuing, you agree to Warp's ",
        "onboarding.third_party.cli_agent_toolbar" => "CLI agent toolbar",
        "onboarding.third_party.notifications" => "Notifications",
        "onboarding.third_party.subtitle" => {
            "Select defaults for using agents like Claude Code, Codex, and Gemini."
        }
        "onboarding.third_party.title" => "Customize third party agents",
        _ => key,
    }
}

struct FinishedOnboardingView {
    selected_settings: Option<SelectedSettings>,
}

impl FinishedOnboardingView {
    fn new(selected_settings: Option<SelectedSettings>) -> Self {
        Self { selected_settings }
    }
}

impl Entity for FinishedOnboardingView {
    type Event = ();
}

impl View for FinishedOnboardingView {
    fn ui_name() -> &'static str {
        "FinishedOnboardingView"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);

        let header_text = if self.selected_settings.is_some() {
            "Finished Onboarding"
        } else {
            "Skipped Onboarding"
        };

        let header = appearance
            .ui_builder()
            .paragraph(header_text)
            .with_style(UiComponentStyles {
                font_size: Some(28.),
                font_weight: Some(Weight::Medium),
                ..Default::default()
            })
            .build()
            .finish();

        let details_text = match &self.selected_settings {
            Some(selected_settings) => format!("SelectedSettings: {selected_settings:?}"),
            None => "SelectedSettings: (none)".to_string(),
        };

        let details = appearance
            .ui_builder()
            .paragraph(details_text)
            .with_style(UiComponentStyles {
                font_size: Some(14.),
                font_weight: Some(Weight::Normal),
                ..Default::default()
            })
            .build()
            .finish();

        let theme = appearance.theme();

        Container::new(
            Flex::column()
                .with_main_axis_size(MainAxisSize::Max)
                .with_main_axis_alignment(MainAxisAlignment::Center)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(header)
                .with_child(Container::new(details).with_margin_top(12.).finish())
                .finish(),
        )
        .with_background(theme.background())
        .with_uniform_padding(64.)
        .finish()
    }
}

impl TypedActionView for FinishedOnboardingView {
    type Action = ();
}

impl Entity for OnboardingMainView {
    type Event = ();
}

impl View for OnboardingMainView {
    fn ui_name() -> &'static str {
        "OnboardingMainView"
    }

    fn render(&self, _app: &AppContext) -> Box<dyn Element> {
        match &self.state {
            OnboardingMainState::Onboarding(view) => ChildView::new(view).finish(),
            OnboardingMainState::Finished(view) => ChildView::new(view).finish(),
        }
    }

    fn on_focus(&mut self, focus_ctx: &warpui_core::FocusContext, ctx: &mut ViewContext<Self>) {
        if let OnboardingMainState::Onboarding(view) = &self.state
            && focus_ctx.is_self_focused()
        {
            ctx.focus(view);
        }
    }
}

impl TypedActionView for OnboardingMainView {
    type Action = ();
}

// ---- Theme definitions copied from app::themes::default_themes (subset) ----

const DARK_MODE_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x616161FF),
    AnsiColor::from_u32(0xFF8272FF),
    AnsiColor::from_u32(0xB4FA72FF),
    AnsiColor::from_u32(0xFEFDC2FF),
    AnsiColor::from_u32(0xA5D5FEFF),
    AnsiColor::from_u32(0xFF8FFDFF),
    AnsiColor::from_u32(0xD0D1FEFF),
    AnsiColor::from_u32(0xF1F1F1FF),
);

const DARK_MODE_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x8E8E8EFF),
    AnsiColor::from_u32(0xFFC4BDFF),
    AnsiColor::from_u32(0xD6FCB9FF),
    AnsiColor::from_u32(0xFEFDD5FF),
    AnsiColor::from_u32(0xC1E3FEFF),
    AnsiColor::from_u32(0xFFB1FEFF),
    AnsiColor::from_u32(0xE5E6FEFF),
    AnsiColor::from_u32(0xFEFFFFFF),
);

const LIGHT_MODE_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x212121FF),
    AnsiColor::from_u32(0xC30771FF),
    AnsiColor::from_u32(0x10A778FF),
    AnsiColor::from_u32(0xA89C14FF),
    AnsiColor::from_u32(0x008EC4FF),
    AnsiColor::from_u32(0x523C79FF),
    AnsiColor::from_u32(0x20A5BAFF),
    AnsiColor::from_u32(0xE0E0E0FF),
);

const LIGHT_MODE_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x212121FF),
    AnsiColor::from_u32(0xFB007AFF),
    AnsiColor::from_u32(0x5FD7AFFF),
    AnsiColor::from_u32(0xF3E430FF),
    AnsiColor::from_u32(0x20BBFCFF),
    AnsiColor::from_u32(0x6855DEFF),
    AnsiColor::from_u32(0x4FB8CCFF),
    AnsiColor::from_u32(0xF1F1F1FF),
);

const PHENOMENON_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x121212FF),
    AnsiColor::from_u32(0xD22D1EFF),
    AnsiColor::from_u32(0x1CA05AFF),
    AnsiColor::from_u32(0xE5A01AFF),
    AnsiColor::from_u32(0x3780E9FF),
    AnsiColor::from_u32(0xBF409DFF),
    AnsiColor::from_u32(0x799C92FF),
    AnsiColor::from_u32(0xFAF9F6FF),
);

const PHENOMENON_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x292929FF),
    AnsiColor::from_u32(0xAE756FFF),
    AnsiColor::from_u32(0x789B88FF),
    AnsiColor::from_u32(0xBD9F65FF),
    AnsiColor::from_u32(0x6F839FFF),
    AnsiColor::from_u32(0xA57899FF),
    AnsiColor::from_u32(0xBFC5C3FF),
    AnsiColor::from_u32(0xFFFFFFFF),
);

const ADEBERRY_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x121212FF),
    AnsiColor::from_u32(0xC76156FF),
    AnsiColor::from_u32(0x57C78AFF),
    AnsiColor::from_u32(0xC8A35AFF),
    AnsiColor::from_u32(0x5785C7FF),
    AnsiColor::from_u32(0xC756A9FF),
    AnsiColor::from_u32(0x57C7C3FF),
    AnsiColor::from_u32(0xEEEDEBFF),
);

const ADEBERRY_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x292929FF),
    AnsiColor::from_u32(0xE3493BFF),
    AnsiColor::from_u32(0x1CA05AFF),
    AnsiColor::from_u32(0xE3AA3BFF),
    AnsiColor::from_u32(0x3BE38AFF),
    AnsiColor::from_u32(0xC8A35AFF),
    AnsiColor::from_u32(0x3BE3DDFF),
    AnsiColor::from_u32(0xFFFFFFFF),
);

fn dark_mode_colors() -> TerminalColors {
    TerminalColors::new(DARK_MODE_NORMAL_COLORS, DARK_MODE_BRIGHT_COLORS)
}

fn light_mode_colors() -> TerminalColors {
    TerminalColors::new(LIGHT_MODE_NORMAL_COLORS, LIGHT_MODE_BRIGHT_COLORS)
}

fn phenomenon_colors() -> TerminalColors {
    TerminalColors::new(PHENOMENON_NORMAL_COLORS, PHENOMENON_BRIGHT_COLORS)
}

fn adeberry_colors() -> TerminalColors {
    TerminalColors::new(ADEBERRY_NORMAL_COLORS, ADEBERRY_BRIGHT_COLORS)
}

fn dark_theme() -> WarpTheme {
    WarpTheme::new(
        Fill::Solid(ColorU::from_u32(0x000000FF)),
        ColorU::from_u32(0xffffffff),
        Fill::Solid(ColorU::from_u32(0x19AAD8FF)),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        None,
        Some("Dark".to_string()),
    )
}

fn light_theme() -> WarpTheme {
    WarpTheme::new(
        Fill::Solid(ColorU::white()),
        ColorU::new(17, 17, 17, 0xFF),
        Fill::Solid(ColorU::from_u32(0x00c2ffff)),
        None,
        Some(Details::Lighter),
        light_mode_colors(),
        None,
        Some("Light".to_string()),
    )
}

fn phenomenon() -> WarpTheme {
    WarpTheme::new(
        Fill::Solid(ColorU::from_u32(0x121212FF)),
        ColorU::from_u32(0xFAF9F6FF),
        Fill::Solid(ColorU::from_u32(0x2E5D9EFF)),
        None,
        Some(Details::Darker),
        phenomenon_colors(),
        Some(Image {
            source: AssetSource::Bundled {
                // Match app's asset layout: this image lives under app/assets/async.
                path: "async/jpg/phenomenon_bg.jpg",
            },
            opacity: 100,
        }),
        Some("Phenomenon".to_string()),
    )
}

fn adeberry() -> WarpTheme {
    WarpTheme::new(
        Fill::Solid(ColorU::from_u32(0x1D2022FF)),
        ColorU::from_u32(0xE4EEF5FF),
        Fill::Solid(ColorU::from_u32(0x6C96B4FF)),
        None,
        Some(Details::Darker),
        adeberry_colors(),
        None,
        Some("Adeberry".to_string()),
    )
}

fn build_appearance(theme: WarpTheme, ctx: &mut AppContext) -> Appearance {
    let ui_font_family =
        load_default_ui_font_family(ctx).expect("unable to load default ui font family");

    Appearance::new(
        theme,
        ui_font_family,
        13.0,
        Weight::Normal,
        ui_font_family,
        1.2,
        ui_font_family,
        ui_font_family,
    )
}

fn load_default_ui_font_family(ctx: &mut AppContext) -> anyhow::Result<FamilyId> {
    Cache::handle(ctx).update(ctx, |font_cache, _| {
        // On Windows, default to use Segoe UI as the UI font.
        #[cfg(windows)]
        if let Ok(font_family_id) = font_cache.load_system_font("Segoe UI") {
            return Ok(font_family_id);
        }

        font_cache.load_family_from_bytes(
            "Roboto",
            vec![
                ASSETS
                    .get("bundled/fonts/roboto/Roboto-Italic.ttf")?
                    .to_vec(),
                ASSETS.get("bundled/fonts/roboto/Roboto-Bold.ttf")?.to_vec(),
                ASSETS
                    .get("bundled/fonts/roboto/Roboto-Regular.ttf")?
                    .to_vec(),
                ASSETS
                    .get("bundled/fonts/roboto/Roboto-Medium.ttf")?
                    .to_vec(),
                ASSETS
                    .get("bundled/fonts/roboto/RobotoFlex-Semibold.ttf")?
                    .to_vec(),
                ASSETS
                    .get("bundled/fonts/roboto/Roboto-BoldItalic.ttf")?
                    .to_vec(),
            ],
        )
    })
}
