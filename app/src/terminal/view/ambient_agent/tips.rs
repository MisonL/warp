//! Tips for cloud mode loading screen.

use warpui::keymap::Keystroke;
use warpui::AppContext;

use crate::ai::agent_tips::AITip;
use crate::localization;

/// A cloud mode tip with text and optional link.
#[derive(Clone, Debug)]
pub struct CloudModeTip {
    text: String,
    link: Option<String>,
}

impl CloudModeTip {
    pub fn new(text: impl Into<String>, link: Option<impl Into<String>>) -> Self {
        Self {
            text: text.into(),
            link: link.map(|l| l.into()),
        }
    }
}

impl AITip for CloudModeTip {
    fn keystroke(&self, _app: &AppContext) -> Option<Keystroke> {
        None
    }

    fn link(&self) -> Option<String> {
        self.link.clone()
    }

    fn description(&self) -> &str {
        &self.text
    }

    // Uses the default implementation which adds "Tip: " prefix and parses backticks as inline code
}

/// Returns a collection of tips for the cloud mode loading screen.
pub fn get_cloud_mode_tips(app: &AppContext) -> Vec<CloudModeTip> {
    const TIP_LINKS: [&str; 40] = [
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations/slack",
        "https://docs.warp.dev/reference/api-and-sdk",
        "https://docs.warp.dev/agent-platform/cloud-agents/secrets",
        "https://oz.warp.dev",
        "https://docs.warp.dev/agent-platform/cloud-agents/viewing-cloud-agent-runs",
        "https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents",
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations/linear",
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations/github-actions",
        "https://github.com/warpdotdev/oz-agent-action",
        "https://docs.warp.dev/reference/api-and-sdk",
        "https://docs.warp.dev/agent-platform/cloud-agents/environments",
        "https://docs.warp.dev/agent-platform/cloud-agents/viewing-cloud-agent-runs",
        "https://docs.warp.dev/agent-platform/cloud-agents/platform",
        "https://docs.warp.dev/agent-platform/cloud-agents/viewing-cloud-agent-runs",
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations",
        "https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents",
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations/linear",
        "https://docs.warp.dev/agent-platform/cloud-agents/platform",
        "https://docs.warp.dev/agent-platform/capabilities/mcp",
        "https://docs.warp.dev/agent-platform/cloud-agents/platform",
        "https://oz.warp.dev",
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations/github-actions",
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations/github-actions",
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations/github-actions",
        "https://docs.warp.dev/agent-platform/cloud-agents/environments",
        "https://docs.warp.dev/reference/api-and-sdk",
        "https://docs.warp.dev/agent-platform/cloud-agents/triggers",
        "https://docs.warp.dev/agent-platform/cloud-agents/secrets",
        "https://docs.warp.dev/agent-platform/cloud-agents/secrets",
        "https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents",
        "https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents",
        "https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents",
        "https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents",
        "https://docs.warp.dev/agent-platform/capabilities/mcp",
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations/slack",
        "https://docs.warp.dev/agent-platform/cloud-agents/integrations/slack",
        "https://docs.warp.dev/reference/api-and-sdk",
        "https://docs.warp.dev/reference/api-and-sdk",
        "https://docs.warp.dev/reference/api-and-sdk",
        "https://docs.warp.dev/reference/api-and-sdk",
    ];

    TIP_LINKS
        .iter()
        .enumerate()
        .map(|(index, link)| {
            let key = format!("terminal.ambient_agent.tip.{:02}", index + 1);
            CloudModeTip::new(
                localization::text_for_app(app, &key),
                Some((*link).to_owned()),
            )
        })
        .collect()
}
