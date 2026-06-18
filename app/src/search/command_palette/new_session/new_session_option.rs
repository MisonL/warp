use std::borrow::Cow;

use warpui::{Action, AppContext};

use crate::server::telemetry::AddTabWithShellSource;
use crate::terminal::available_shells::AvailableShell;
use crate::terminal::view::TerminalAction;
use crate::{localization, WorkspaceAction};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NewSessionOptionId(pub(crate) String);
impl NewSessionOptionId {
    #[cfg_attr(not(feature = "local_tty"), allow(dead_code))]
    pub(super) fn new(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug)]
pub(super) enum Direction {
    Down,
    Right,
    Up,
    Left,
}

impl Direction {
    fn localization_key(&self) -> &'static str {
        match self {
            Self::Down => "search.command_palette.new_session.split_down",
            Self::Right => "search.command_palette.new_session.split_right",
            Self::Up => "search.command_palette.new_session.split_up",
            Self::Left => "search.command_palette.new_session.split_left",
        }
    }
}

#[derive(Debug)]
pub(super) enum NewSessionConfig {
    NewTab(AvailableShell),
    NewWindow(AvailableShell),
    Split(Direction, AvailableShell),
}

impl NewSessionConfig {
    fn shell(&self) -> &AvailableShell {
        match self {
            NewSessionConfig::NewTab(shell) => shell,
            NewSessionConfig::NewWindow(shell) => shell,
            NewSessionConfig::Split(_, shell) => shell,
        }
    }
}

#[derive(Debug)]
/// An option for creating a new terminal session
///
/// Contains configuration information like:
/// - which shell to use
/// - how to display the option in the command palette
pub struct NewSessionOption {
    id: NewSessionOptionId,
    description: String,
    config: NewSessionConfig,
}

impl NewSessionOption {
    pub fn id(&self) -> &NewSessionOptionId {
        &self.id
    }

    /// Returns the description (a.k.a. the top line in the command palette entry)
    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}

impl NewSessionOption {
    pub(super) fn new(id: NewSessionOptionId, config: NewSessionConfig, app: &AppContext) -> Self {
        let shell_name = config.shell().short_name_for_app(app);
        let description = match &config {
            NewSessionConfig::NewTab(_) => localized_description(
                app,
                "search.command_palette.new_session.new_tab",
                &shell_name,
            ),
            NewSessionConfig::NewWindow(_) => localized_description(
                app,
                "search.command_palette.new_session.new_window",
                &shell_name,
            ),
            NewSessionConfig::Split(direction, _) => {
                localized_description(app, direction.localization_key(), &shell_name)
            }
        };
        Self {
            id,
            description,
            config,
        }
    }

    /// Returns an action that should be triggered if this entry is accepted
    pub fn action(&self) -> Box<dyn Action> {
        match &self.config {
            NewSessionConfig::NewTab(shell) => Box::new(WorkspaceAction::AddTabWithShell {
                shell: shell.clone(),
                source: AddTabWithShellSource::CommandPalette,
            }),
            NewSessionConfig::NewWindow(shell) => Box::new(WorkspaceAction::AddWindowWithShell {
                shell: shell.clone(),
            }),
            NewSessionConfig::Split(Direction::Down, shell) => {
                Box::new(TerminalAction::SplitDown(Some(shell.clone())))
            }
            NewSessionConfig::Split(Direction::Up, shell) => {
                Box::new(TerminalAction::SplitUp(Some(shell.clone())))
            }
            NewSessionConfig::Split(Direction::Right, shell) => {
                Box::new(TerminalAction::SplitRight(Some(shell.clone())))
            }
            NewSessionConfig::Split(Direction::Left, shell) => {
                Box::new(TerminalAction::SplitLeft(Some(shell.clone())))
            }
        }
    }

    /// Returns the details (a.k.a. the second line in the command palette entry)
    pub fn details(&self) -> Cow<'_, str> {
        self.config.shell().details()
    }

    pub fn details_for_app(&self, app: &AppContext) -> Cow<'_, str> {
        self.config.shell().details_for_app(app)
    }
}

fn localized_description(app: &AppContext, key: &str, shell: &str) -> String {
    localization::text_for_app_with_args(app, key, &[("shell", shell)])
}
