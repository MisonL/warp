use std::collections::HashMap;
use std::sync::Arc;

pub const AI_FEATURE_KEYS: &[&str] = &[
    "onboarding.features.ai.warp_agents",
    "onboarding.features.ai.oz_cloud_agents",
    "onboarding.features.ai.next_command_predictions",
    "onboarding.features.ai.prompt_suggestions",
    "onboarding.features.ai.codebase_context",
    "onboarding.features.ai.remote_control",
    "onboarding.features.ai.agents_over_ssh",
];

pub const WARP_DRIVE_FEATURE_KEYS: &[&str] = &[
    "onboarding.features.drive.warp_drive",
    "onboarding.features.drive.session_sharing",
];

pub const SUBSCRIBE_ITEM_KEYS: &[&str] = &[
    "onboarding.free_user_no_ai.subscribe.item.credits",
    "onboarding.free_user_no_ai.subscribe.item.frontier_models",
    "onboarding.free_user_no_ai.subscribe.item.reload_credits",
    "onboarding.free_user_no_ai.subscribe.item.cloud_agents",
    "onboarding.free_user_no_ai.subscribe.item.indexing",
    "onboarding.free_user_no_ai.subscribe.item.warp_drive",
    "onboarding.free_user_no_ai.subscribe.item.support",
    "onboarding.free_user_no_ai.subscribe.item.cloud_storage",
];

#[derive(Clone, Debug)]
pub struct OnboardingCopy {
    values: Arc<HashMap<&'static str, String>>,
}

impl OnboardingCopy {
    pub fn localized(mut localize: impl FnMut(&'static str) -> String) -> Self {
        Self {
            values: Arc::new(
                DEFAULT_COPY
                    .iter()
                    .map(|(key, _)| (*key, localize(key)))
                    .collect(),
            ),
        }
    }

    pub fn text(&self, key: &'static str) -> &str {
        self.values
            .get(key)
            .map(String::as_str)
            .unwrap_or_else(|| default_text(key))
    }

    pub fn text_owned(&self, key: &'static str) -> String {
        self.text(key).to_string()
    }

    pub fn price_badge(&self, price_dollars: i32) -> String {
        self.text("onboarding.agent.price_badge")
            .replace("{price}", &price_dollars.to_string())
    }
}

impl Default for OnboardingCopy {
    fn default() -> Self {
        Self {
            values: Arc::new(
                DEFAULT_COPY
                    .iter()
                    .map(|(key, value)| (*key, (*value).to_string()))
                    .collect(),
            ),
        }
    }
}

fn default_text(key: &'static str) -> &'static str {
    DEFAULT_COPY
        .iter()
        .find_map(|(candidate, value)| (*candidate == key).then_some(*value))
        .unwrap_or_else(|| panic!("missing onboarding copy key: {key}"))
}

const DEFAULT_COPY: &[(&str, &str)] = &[
    ("onboarding.agent.auth.browser_middle", " and open the page manually. "),
    (
        "onboarding.agent.auth.browser_prefix",
        "If your browser hasn't launched, ",
    ),
    (
        "onboarding.agent.auth.browser_suffix",
        " to paste your token from the browser.",
    ),
    ("onboarding.agent.auth.click_here", "Click here"),
    ("onboarding.agent.auth.copy_url", "copy the URL"),
    ("onboarding.agent.autonomy", "Autonomy"),
    (
        "onboarding.agent.autonomy.full.description",
        "Runs commands, writes code, and reads files without asking.",
    ),
    ("onboarding.agent.autonomy.full.title", "Full"),
    (
        "onboarding.agent.autonomy.none.description",
        "Takes no actions without your approval.",
    ),
    ("onboarding.agent.autonomy.none.title", "None"),
    (
        "onboarding.agent.autonomy.partial.description",
        "Can plan, read files, and execute low-risk commands. Asks before making any changes or executing sensitive commands.",
    ),
    ("onboarding.agent.autonomy.partial.title", "Partial"),
    (
        "onboarding.agent.default_model",
        "Default model",
    ),
    ("onboarding.agent.disable", "Disable Warp Agent"),
    ("onboarding.agent.plan_activated", "Plan successfully activated. All premium models are available."),
    ("onboarding.agent.premium", "Premium"),
    ("onboarding.agent.price_badge", "Starting at ${price}/mo"),
    ("onboarding.agent.recommended", "Recommended"),
    (
        "onboarding.agent.subtitle",
        "Select your in-app agent's defaults.",
    ),
    (
        "onboarding.agent.team_workspace.description",
        "Autonomy settings are configured as part of your team workspace.",
    ),
    (
        "onboarding.agent.team_workspace.title",
        "Set by Team Workspace",
    ),
    ("onboarding.agent.title", "Customize your Warp Agent"),
    (
        "onboarding.agent.upgrade.subtitle",
        "State-of-the-art models require paid plans.",
    ),
    (
        "onboarding.agent.upgrade.title",
        "Upgrade for access to premium models.",
    ),
    ("onboarding.common.back", "Back"),
    ("onboarding.common.disabled", "Disabled"),
    ("onboarding.common.enabled", "Enabled"),
    ("onboarding.common.finish", "Finish"),
    ("onboarding.common.free", "Free"),
    ("onboarding.common.get_started", "Get started"),
    ("onboarding.common.get_warping", "Get Warping"),
    ("onboarding.common.next", "Next"),
    ("onboarding.common.skip", "Skip"),
    ("onboarding.common.submit", "Submit"),
    ("onboarding.common.subscribe", "Subscribe"),
    ("onboarding.common.upgrade", "Upgrade"),
    (
        "onboarding.callout.agent_mode.back_to_terminal",
        "Back to terminal",
    ),
    (
        "onboarding.callout.agent_mode.initialize",
        "Initialize",
    ),
    (
        "onboarding.callout.agent_mode.skip_initialization",
        "Skip initialization",
    ),
    (
        "onboarding.callout.agent_mode.title",
        "You're in agent mode",
    ),
    (
        "onboarding.callout.agent_mode.with_project_body",
        "Agent mode gives your questions and tasks their own conversation, so you can ask follow-ups without leaving your terminal workflow.\n\nSubmit the query below to have the agent initialize this project, or clear the input and start your own!",
    ),
    (
        "onboarding.callout.agent_mode.without_project_body",
        "Agent mode gives your questions and tasks their own conversation, so you can ask follow-ups without leaving your terminal workflow. Press {keybinding} to return to terminal mode at any point.",
    ),
    (
        "onboarding.callout.agent_prompt.placeholder",
        "Tell the agent what to build...",
    ),
    (
        "onboarding.callout.meet_input.body",
        "Your terminal input accepts both terminal commands and agent prompts and automatically detects which you're using. Use {keybinding} to lock the input to Agent mode (natural language) or Terminal mode (commands).",
    ),
    (
        "onboarding.callout.meet_input.title",
        "Meet the Warp input",
    ),
    (
        "onboarding.callout.talk_to_agent.body",
        "You can type in natural language to engage the agent. Submit the query below to start: What tests exist in this repo, how are they structured, and what do they cover?",
    ),
    (
        "onboarding.callout.talk_to_agent.prompt",
        "What tests exist in this repo, how are they structured, and what do they cover?",
    ),
    (
        "onboarding.callout.talk_to_agent.title",
        "Talk to the agent",
    ),
    (
        "onboarding.callout.terminal_command.placeholder",
        "Run a command...",
    ),
    (
        "onboarding.callout.terminal_mode.body",
        "Run commands here, just like a regular terminal. If you type a question or task using natural language, Warp can suggest opening it in agent mode. You can always override using {keybinding}.",
    ),
    (
        "onboarding.callout.terminal_mode.enable_nld",
        "Enable Natural Language Detection",
    ),
    (
        "onboarding.callout.terminal_mode.title",
        "You're in terminal mode",
    ),
    (
        "onboarding.callout.terminal_mode.welcome_title",
        "Welcome to terminal mode",
    ),
    ("onboarding.customize.code_review", "Code review"),
    ("onboarding.customize.conversation_history", "Conversation history"),
    ("onboarding.customize.file_explorer", "File explorer"),
    ("onboarding.customize.global_file_search", "Global file search"),
    (
        "onboarding.customize.subtitle",
        "Tailor your features and UI to your working style.",
    ),
    ("onboarding.customize.tab_styling", "Tab styling"),
    ("onboarding.customize.title", "Customize your Warp"),
    ("onboarding.customize.tools_panel", "Tools panel"),
    ("onboarding.customize.horizontal", "Horizontal"),
    ("onboarding.customize.vertical", "Vertical"),
    ("onboarding.customize.warp_drive", "Warp Drive"),
    (
        "onboarding.features.ai.agents_over_ssh",
        "Agents over SSH",
    ),
    (
        "onboarding.features.ai.codebase_context",
        "Codebase context",
    ),
    (
        "onboarding.features.ai.next_command_predictions",
        "Next command predictions",
    ),
    (
        "onboarding.features.ai.oz_cloud_agents",
        "Oz cloud agents platform",
    ),
    (
        "onboarding.features.ai.prompt_suggestions",
        "Prompt suggestions",
    ),
    (
        "onboarding.features.ai.remote_control",
        "Remote control with Claude Code, Codex, and other agents",
    ),
    ("onboarding.features.ai.warp_agents", "Warp agents"),
    ("onboarding.features.drive.session_sharing", "Session Sharing"),
    ("onboarding.features.drive.warp_drive", "Warp Drive"),
    (
        "onboarding.free_user_no_ai.agent.description",
        "Iterate, plan, and build with Oz: Warp's built-in agent. Available locally or in the cloud.",
    ),
    (
        "onboarding.free_user_no_ai.agent.title",
        "Agent driven development with Warp's built-in agent",
    ),
    (
        "onboarding.free_user_no_ai.subscribe.item.cloud_agents",
        "Extended cloud agents access",
    ),
    (
        "onboarding.free_user_no_ai.subscribe.item.cloud_storage",
        "Unlimited cloud conversation storage",
    ),
    (
        "onboarding.free_user_no_ai.subscribe.item.credits",
        "1,500 credits per month",
    ),
    (
        "onboarding.free_user_no_ai.subscribe.item.frontier_models",
        "Access to frontier OpenAI, Anthropic, and Google models",
    ),
    (
        "onboarding.free_user_no_ai.subscribe.item.indexing",
        "Highest codebase indexing limits",
    ),
    (
        "onboarding.free_user_no_ai.subscribe.item.reload_credits",
        "Access to Reload credits and volume-based discounts",
    ),
    (
        "onboarding.free_user_no_ai.subscribe.item.support",
        "Private email support",
    ),
    (
        "onboarding.free_user_no_ai.subscribe.item.warp_drive",
        "Unlimited Warp Drive objects and collaboration",
    ),
    (
        "onboarding.free_user_no_ai.subscribe.title",
        "Subscribe to access agent driven development in Warp.",
    ),
    (
        "onboarding.free_user_no_ai.terminal.description",
        "A modern terminal that supports third-party agents (Claude Code, Codex, Gemini CLI) and classic terminal workflows.",
    ),
    (
        "onboarding.free_user_no_ai.terminal.title",
        "Classic terminal with third-party agents",
    ),
    ("onboarding.free_user_no_ai.title", "Let's get started."),
    ("onboarding.intention.agent.title", "Build faster with AI agents"),
    (
        "onboarding.intention.agent.description",
        "An agent-first experience with best in class terminal support. Get terminal and agent driven development AI features like:",
    ),
    (
        "onboarding.intention.subtitle",
        "How do you want to work?",
    ),
    (
        "onboarding.intention.terminal.badge",
        "No AI features",
    ),
    (
        "onboarding.intention.terminal.description",
        "A modern terminal optimized for speed, context, and control without AI.",
    ),
    (
        "onboarding.intention.terminal.title",
        "Just use the terminal",
    ),
    ("onboarding.intention.title", "Welcome to Warp"),
    (
        "onboarding.intro.account_prompt",
        "Already have an account? ",
    ),
    ("onboarding.intro.log_in", "Log in"),
    (
        "onboarding.intro.subtitle",
        "A modern terminal with state of the art agents built in.",
    ),
    ("onboarding.intro.title", "Welcome to Warp"),
    (
        "onboarding.project.initialize.description",
        "Prepares the project environment, builds an index of your code, and generates project rules, giving the agent deeper understanding and better performance.",
    ),
    (
        "onboarding.project.initialize.title",
        "Initialize project automatically",
    ),
    ("onboarding.project.open_local_folder", "Open local folder"),
    (
        "onboarding.project.subtitle",
        "Set up a project to optimize it for coding in Warp.",
    ),
    ("onboarding.project.title", "Open a project"),
    ("onboarding.theme.default_theme_name", "Theme"),
    ("onboarding.theme.privacy_link", "Privacy Settings"),
    (
        "onboarding.theme.privacy_prefix",
        "If you'd like to opt out of analytics, you can adjust your ",
    ),
    (
        "onboarding.theme.subtitle",
        "Click or use arrow keys to select, Enter to confirm.",
    ),
    (
        "onboarding.theme.sync_with_os",
        "Sync light/dark theme with OS",
    ),
    ("onboarding.theme.title", "Choose a theme"),
    ("onboarding.theme.tos_link", "Terms of Service"),
    (
        "onboarding.theme.tos_prefix",
        "By continuing, you agree to Warp's ",
    ),
    (
        "onboarding.third_party.cli_agent_toolbar",
        "CLI agent toolbar",
    ),
    ("onboarding.third_party.notifications", "Notifications"),
    (
        "onboarding.third_party.subtitle",
        "Select defaults for using agents like Claude Code, Codex, and Gemini.",
    ),
    (
        "onboarding.third_party.title",
        "Customize third party agents",
    ),
];
