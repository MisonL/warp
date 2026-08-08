pub mod bindings;
pub mod commands;

use bitflags::bitflags;
pub use commands::SlashCommandId;
use settings::SettingsMode;
use warpui::AppContext;

use crate::localization;

bitflags! {
    /// Specifies the requirements for a slash command to be available.
    ///
    /// Each flag represents a requirement that the session context must satisfy. The command is
    /// available when the session supports *all* of the command's requirement flags.
    ///
    /// A few common cases:
    /// * If neither [`Self::AGENT_VIEW`] nor [`Self::TERMINAL_VIEW`] is set, the command is available in all modes.
    ///   A command should *not* set both flags to be available in both modes - this results in requirements that cannot be satisfied.
    /// * Most `/fork`-like slash commands require [`Self::NO_LRC_CONTROL`] and [`Self::ACTIVE_CONVERSATION`]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Availability: u16 {
        /// No requirements — always available.
        const ALWAYS = 0;
        /// Requires the agent view.
        const AGENT_VIEW = 1 << 0;
        /// Requires the terminal view.
        const TERMINAL_VIEW = 1 << 1;
        /// Requires a local session (not available in remote/cloud sessions).
        const LOCAL = 1 << 2;
        /// Requires a git repository.
        const REPOSITORY = 1 << 3;
        /// Requires that the agent is not currently in control of a long-running command.
        const NO_LRC_CONTROL = 1 << 4;
        /// Requires an active AI conversation.
        const ACTIVE_CONVERSATION = 1 << 5;
        /// Requires codebase context to be enabled.
        const CODEBASE_CONTEXT = 1 << 6;
        /// Requires AI to be globally enabled.
        const AI_ENABLED = 1 << 7;
        /// Requires a non-cloud-agent context.
        const NOT_CLOUD_AGENT = 1 << 8;
        /// Requires a cloud-agent context.
        const CLOUD_AGENT = 1 << 9;
        /// Set on the session context iff the slash command data source was constructed via
        /// `SlashCommandDataSource::for_cloud_mode_v2` *and* `FeatureFlag::CloudModeInputV2`
        /// is enabled. Commands that require this bit are hidden everywhere except the V2
        /// cloud-mode composing input.
        const CLOUD_MODE_V2_COMPOSER = 1 << 10;
    }
}
/// Stable identity for a static slash command.
///
/// Front-ends dispatch on this value instead of matching command-name strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlashCommandKind {
    Agent,
    CloudAgent,
    AddMcp,
    ApiKeys,
    ConnectGrok,
    Upgrade,
    ManageBilling,
    AutoApprove,
    Statusline,
    ResetStatusline,
    Mcp,
    ViewLogs,
    Voice,
    NaturalLanguageDetection,
    Theme,
    Exit,
    Logout,
    CreateEnvironment,
    CreateDockerSandbox,
    CreateNewProject,
    EditSkill,
    InvokeSkill,
    AddPrompt,
    AddRule,
    Edit,
    RenameTab,
    RenameConversation,
    SetTabColor,
    Fork,
    MoveToCloud,
    OpenCodeReview,
    Index,
    Init,
    OpenProjectRules,
    OpenMcpServers,
    OpenSettingsFile,
    Changelog,
    Feedback,
    OpenRepo,
    OpenRules,
    New,
    Clear,
    Model,
    Host,
    Harness,
    Environment,
    Profile,
    Plan,
    Orchestrate,
    Compact,
    CompactAnd,
    Queue,
    ForkAndCompact,
    ForkFrom,
    ContinueLocally,
    Usage,
    RemoteControl,
    Cost,
    Conversations,
    Prompts,
    Rewind,
    ExportToClipboard,
    ExportToFile,
    VimMode,
    Status,
    CopyDebuggingId,
}

/// The application surfaces on which a static slash command is implemented.
///
/// This field is required on every [`StaticCommand`] so new commands must explicitly declare
/// whether they are GUI-only, TUI-only, or shared by both front-ends. GUI-capable variants also
/// require the icon path used to render the command in GUI menus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashCommandSurfaces {
    GuiOnly { icon_path: &'static str },
    TuiOnly,
    GuiAndTui { icon_path: &'static str },
}

impl SlashCommandSurfaces {
    pub fn supports_gui(self) -> bool {
        matches!(self, Self::GuiOnly { .. } | Self::GuiAndTui { .. })
    }

    pub fn supports_tui(self) -> bool {
        matches!(self, Self::TuiOnly | Self::GuiAndTui { .. })
    }

    pub fn gui_icon_path(self) -> Option<&'static str> {
        match self {
            Self::GuiOnly { icon_path } | Self::GuiAndTui { icon_path } => Some(icon_path),
            Self::TuiOnly => None,
        }
    }
    pub fn includes(self, settings_mode: SettingsMode) -> bool {
        match settings_mode {
            SettingsMode::Gui => self.supports_gui(),
            SettingsMode::Tui => self.supports_tui(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Argument {
    pub hint_text: Option<&'static str>,
    pub hint_text_key: Option<&'static str>,
    pub is_optional: bool,
    /// If `true`, selecting the slash command from the menu (or via keybinding) will execute the
    /// slash command with no arguments.
    ///
    /// If `false`, selecting the slash command from the menu (or via keybinding) inserts the
    /// slash command into the input.
    ///
    /// Set this based on whether or not you want you think a user should always have the option to
    /// supply an argument.
    pub should_execute_on_selection: bool,
}

impl Argument {
    pub(super) fn optional() -> Self {
        Self {
            is_optional: true,
            ..Default::default()
        }
    }

    pub(super) fn required() -> Self {
        Self {
            is_optional: false,
            ..Default::default()
        }
    }

    pub(super) fn with_hint_text(mut self, text: &'static str) -> Self {
        self.hint_text = Some(text);
        self
    }

    pub(super) fn with_hint_text_key(mut self, key: &'static str) -> Self {
        self.hint_text_key = Some(key);
        self
    }

    pub(super) fn with_execute_on_selection(mut self) -> Self {
        self.should_execute_on_selection = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticCommand {
    pub kind: SlashCommandKind,
    pub name: &'static str,
    pub description: &'static str,
    pub supported_surfaces: SlashCommandSurfaces,
    /// Specifies the requirements for this command to be available. See [`Availability`].
    pub availability: Availability,
    /// Whether this command requires AI mode when executed.
    /// If true, AI mode will be activated when the command is accepted.
    pub auto_enter_ai_mode: bool,
    pub argument: Option<Argument>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashCommandArgumentHint {
    pub input_prefix: String,
    pub text: &'static str,
    pub localization_key: Option<&'static str>,
}

impl StaticCommand {
    pub fn localized_description(&self, app: &AppContext) -> String {
        let key = match self.kind {
            SlashCommandKind::Agent => "terminal.slash.command.agent.description",
            SlashCommandKind::CloudAgent => "terminal.slash.command.cloud_agent.description",
            SlashCommandKind::AddMcp => "terminal.slash.command.add_mcp.description",
            SlashCommandKind::ApiKeys => "terminal.slash.command.api_keys.description",
            SlashCommandKind::ConnectGrok => "terminal.slash.command.connect_grok.description",
            SlashCommandKind::Upgrade => "terminal.slash.command.upgrade.description",
            SlashCommandKind::ManageBilling => "terminal.slash.command.manage_billing.description",
            SlashCommandKind::AutoApprove => "terminal.slash.command.auto_approve.description",
            SlashCommandKind::Statusline => "terminal.slash.command.statusline.description",
            SlashCommandKind::ResetStatusline => {
                "terminal.slash.command.reset_statusline.description"
            }
            SlashCommandKind::Mcp => "terminal.slash.command.mcp.description",
            SlashCommandKind::ViewLogs => "terminal.slash.command.view_logs.description",
            SlashCommandKind::Voice => "terminal.slash.command.voice.description",
            SlashCommandKind::NaturalLanguageDetection => {
                "terminal.slash.command.natural_language_detection.description"
            }
            SlashCommandKind::Theme => "terminal.slash.command.theme.description",
            SlashCommandKind::Exit => "terminal.slash.command.exit.description",
            SlashCommandKind::Logout => "terminal.slash.command.logout.description",
            SlashCommandKind::CreateEnvironment => {
                "terminal.slash.command.create_environment.description"
            }
            SlashCommandKind::CreateDockerSandbox => {
                "terminal.slash.command.docker_sandbox.description"
            }
            SlashCommandKind::CreateNewProject => {
                "terminal.slash.command.create_new_project.description"
            }
            SlashCommandKind::EditSkill => "terminal.slash.command.open_skill.description",
            SlashCommandKind::InvokeSkill => "terminal.slash.command.skills.description",
            SlashCommandKind::AddPrompt => "terminal.slash.command.add_prompt.description",
            SlashCommandKind::AddRule => "terminal.slash.command.add_rule.description",
            SlashCommandKind::Edit => "terminal.slash.command.open_file.description",
            SlashCommandKind::RenameTab => "terminal.slash.command.rename_tab.description",
            SlashCommandKind::RenameConversation => {
                "terminal.slash.command.rename_conversation.description"
            }
            SlashCommandKind::SetTabColor => "terminal.slash.command.set_tab_color.description",
            SlashCommandKind::Fork => "terminal.slash.command.fork.description",
            SlashCommandKind::MoveToCloud => "terminal.slash.command.handoff.description",
            SlashCommandKind::OpenCodeReview => {
                "terminal.slash.command.open_code_review.description"
            }
            SlashCommandKind::Index => "terminal.slash.command.index.description",
            SlashCommandKind::Init => "terminal.slash.command.init.description",
            SlashCommandKind::OpenProjectRules => {
                "terminal.slash.command.open_project_rules.description"
            }
            SlashCommandKind::OpenMcpServers => {
                "terminal.slash.command.open_mcp_servers.description"
            }
            SlashCommandKind::OpenSettingsFile => {
                "terminal.slash.command.open_settings_file.description"
            }
            SlashCommandKind::Changelog => "terminal.slash.command.changelog.description",
            SlashCommandKind::Feedback => "terminal.slash.command.feedback.description",
            SlashCommandKind::OpenRepo => "terminal.slash.command.open_repo.description",
            SlashCommandKind::OpenRules => "terminal.slash.command.open_rules.description",
            SlashCommandKind::New => "terminal.slash.command.new.description",
            SlashCommandKind::Clear => "terminal.slash.command.clear.description",
            SlashCommandKind::Model => "terminal.slash.command.model.description",
            SlashCommandKind::Host => "terminal.slash.command.host.description",
            SlashCommandKind::Harness => "terminal.slash.command.harness.description",
            SlashCommandKind::Environment => "terminal.slash.command.environment.description",
            SlashCommandKind::Profile => "terminal.slash.command.profile.description",
            SlashCommandKind::Plan => "terminal.slash.command.plan.description",
            SlashCommandKind::Orchestrate => "terminal.slash.command.orchestrate.description",
            SlashCommandKind::Compact => "terminal.slash.command.compact.description",
            SlashCommandKind::CompactAnd => "terminal.slash.command.compact_and.description",
            SlashCommandKind::Queue => "terminal.slash.command.queue.description",
            SlashCommandKind::ForkAndCompact => {
                "terminal.slash.command.fork_and_compact.description"
            }
            SlashCommandKind::ForkFrom => "terminal.slash.command.fork_from.description",
            SlashCommandKind::ContinueLocally => {
                "terminal.slash.command.continue_locally.description"
            }
            SlashCommandKind::Usage => "terminal.slash.command.usage.description",
            SlashCommandKind::RemoteControl => "terminal.slash.command.remote_control.description",
            SlashCommandKind::Cost => "terminal.slash.command.cost.description",
            SlashCommandKind::Conversations => "terminal.slash.command.conversations.description",
            SlashCommandKind::Prompts => "terminal.slash.command.prompts.description",
            SlashCommandKind::Rewind => "terminal.slash.command.rewind.description",
            SlashCommandKind::ExportToClipboard => {
                "terminal.slash.command.export_to_clipboard.description"
            }
            SlashCommandKind::ExportToFile => "terminal.slash.command.export_to_file.description",
            SlashCommandKind::VimMode => "terminal.slash.command.vim_mode.description",
            SlashCommandKind::Status => "terminal.slash.command.status.description",
            SlashCommandKind::CopyDebuggingId => {
                "terminal.slash.command.copy_debugging_id.description"
            }
        };
        localization::text_for_app(app, key)
    }

    pub fn supports_gui(&self) -> bool {
        self.supported_surfaces.supports_gui()
    }

    pub fn supports_tui(&self) -> bool {
        self.supported_surfaces.supports_tui()
    }
    pub fn supports_surface(&self, settings_mode: SettingsMode) -> bool {
        self.supported_surfaces.includes(settings_mode)
    }

    pub fn matches_filter(&self, filter_text: &str) -> bool {
        if filter_text.is_empty() {
            return true;
        }

        let filter_lower = filter_text.to_lowercase();
        self.name
            .to_lowercase()
            .get(1..)
            .unwrap_or("")
            .starts_with(&filter_lower)
    }

    pub fn is_active(&self, session_context: Availability) -> bool {
        session_context.contains(self.availability)
    }

    pub fn argument_hint(&self) -> Option<SlashCommandArgumentHint> {
        let argument = self.argument.as_ref()?;
        let text = argument.hint_text?;
        Some(SlashCommandArgumentHint {
            input_prefix: format!("{} ", self.name),
            text,
            localization_key: argument.hint_text_key,
        })
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
