use std::borrow::Cow;
use std::fmt;

use warpui::{Action, AppContext};

use crate::WorkspaceAction;
use crate::server::telemetry::AddTabWithShellSource;
use crate::terminal::available_shells::AvailableShell;
use crate::terminal::view::TerminalAction;

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

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::Down => "Down",
                Direction::Right => "Right",
                Direction::Up => "Up",
                Direction::Left => "Left",
            }
        )
    }
}

impl Direction {
    fn new_session_translation_key(&self) -> &'static str {
        match self {
            Direction::Down => "search.command_palette.new_session.split_down",
            Direction::Right => "search.command_palette.new_session.split_right",
            Direction::Up => "search.command_palette.new_session.split_up",
            Direction::Left => "search.command_palette.new_session.split_left",
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

    pub fn localized_description(&self, app: &AppContext) -> String {
        let (key, shell) = match &self.config {
            NewSessionConfig::NewTab(shell) => (
                "search.command_palette.new_session.new_tab",
                shell.short_name_for_app(app),
            ),
            NewSessionConfig::NewWindow(shell) => (
                "search.command_palette.new_session.new_window",
                shell.short_name_for_app(app),
            ),
            NewSessionConfig::Split(direction, shell) => (
                direction.new_session_translation_key(),
                shell.short_name_for_app(app),
            ),
        };

        crate::localization::text_for_app_with_args(app, key, &[("shell", shell.as_ref())])
    }
}

impl NewSessionOption {
    pub(super) fn new(id: NewSessionOptionId, config: NewSessionConfig) -> Self {
        let description = match &config {
            NewSessionConfig::NewTab(shell) => format!("Create New Tab: {}", shell.short_name()),
            NewSessionConfig::NewWindow(shell) => {
                format!("Create New Window: {}", shell.short_name())
            }
            NewSessionConfig::Split(direction, shell) => {
                format!("Split Pane {direction}: {}", shell.short_name())
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

    pub fn localized_details<'a>(&'a self, app: &'a AppContext) -> Cow<'a, str> {
        self.config.shell().details_for_app(app)
    }
}
