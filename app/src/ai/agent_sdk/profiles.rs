use comfy_table::Cell;
use serde::Serialize;
use warp_cli::agent::AgentProfileCommand;
use warp_cli::GlobalOptions;
use warp_localization::LocaleId;
use warpui::{AppContext, ModelContext, SingletonEntity};

use crate::ai::agent_sdk::output::{self, TableFormat};
use crate::ai::execution_profiles::profiles::AIExecutionProfilesModel;
use crate::ai::execution_profiles::AIExecutionProfile;
use crate::cloud_object::model::generic_string_model::StringModel;
use crate::localization;
use crate::server::cloud_objects::update_manager::UpdateManager;
use crate::server::ids::SyncId;

fn text(app: &AppContext, key: &str) -> String {
    localization::text_for_app(app, key)
}

fn text_for_locale(locale: LocaleId, key: &str) -> String {
    localization::text_for_locale(locale, key)
}

/// Handle Agent Profile-related CLI commands.
pub fn run(
    ctx: &mut AppContext,
    global_options: GlobalOptions,
    command: AgentProfileCommand,
) -> anyhow::Result<()> {
    let runner = ctx.add_singleton_model(|_ctx| ProfilesCommandRunner);
    match command {
        AgentProfileCommand::List => {
            runner.update(ctx, |runner, ctx| runner.list(global_options, ctx));
            Ok(())
        }
    }
}

/// Singleton model that runs async work for profile CLI commands.
struct ProfilesCommandRunner;

impl ProfilesCommandRunner {
    fn list(&self, global_options: GlobalOptions, ctx: &mut ModelContext<Self>) {
        // Ensure initial cloud sync completes so profiles from the server are available.
        let initial_sync = UpdateManager::as_ref(ctx).initial_load_complete();

        ctx.spawn(initial_sync, move |_, _, ctx| {
            let profiles_model = AIExecutionProfilesModel::as_ref(ctx);

            let profile_ids = profiles_model.get_all_profile_ids();

            let profiles: Vec<_> = profile_ids
                .iter()
                .flat_map(|id| profiles_model.get_profile_by_id(*id, ctx))
                .map(|profile| {
                    let profile_data = profile.data();
                    let name = profile_data.display_name();
                    let name_fallback = ProfileNameFallback::from_profile(profile_data);
                    let id = match profile.sync_id() {
                        Some(SyncId::ServerId(server_id)) => server_id.to_string(),
                        _ => super::common::UNSYNCED_ID.to_string(),
                    };
                    ProfileInfo {
                        id,
                        name,
                        name_fallback,
                    }
                })
                .collect();

            output::print_list_for_app(profiles, global_options.output_format, ctx);

            ctx.terminate_app(warpui::platform::TerminationMode::ForceTerminate, None);
        });
    }
}

impl warpui::Entity for ProfilesCommandRunner {
    type Event = ();
}
impl SingletonEntity for ProfilesCommandRunner {}

/// Profile information that's shown in the `list` command.
#[derive(Serialize)]
struct ProfileInfo {
    id: String,
    name: String,
    #[serde(skip)]
    name_fallback: Option<ProfileNameFallback>,
}

#[derive(Clone, Copy)]
enum ProfileNameFallback {
    Default,
    Untitled,
}

impl ProfileNameFallback {
    fn from_profile(profile: &AIExecutionProfile) -> Option<Self> {
        if profile.is_default_profile {
            Some(Self::Default)
        } else if profile.name.trim().is_empty() {
            Some(Self::Untitled)
        } else {
            None
        }
    }

    fn text_for_app(self, app: &AppContext) -> String {
        text(app, self.localization_key())
    }

    fn text_for_locale(self, locale: LocaleId) -> String {
        text_for_locale(locale, self.localization_key())
    }

    fn localization_key(self) -> &'static str {
        match self {
            Self::Default => "settings.execution_profile.editor.default_profile_name",
            Self::Untitled => "settings.execution_profile.untitled_profile_name",
        }
    }
}

impl TableFormat for ProfileInfo {
    fn header() -> Vec<Cell> {
        vec![
            Cell::new(text_for_locale(
                LocaleId::EnUs,
                "agent_sdk.profiles.table.id",
            )),
            Cell::new(text_for_locale(
                LocaleId::EnUs,
                "agent_sdk.profiles.table.name",
            )),
        ]
    }

    fn header_for_app(app: &AppContext) -> Vec<Cell> {
        vec![
            Cell::new(text(app, "agent_sdk.profiles.table.id")),
            Cell::new(text(app, "agent_sdk.profiles.table.name")),
        ]
    }

    fn row(&self) -> Vec<Cell> {
        vec![Cell::new(&self.id), Cell::new(&self.name)]
    }

    fn row_for_app(&self, app: &AppContext) -> Vec<Cell> {
        let name = self
            .name_fallback
            .map(|fallback| fallback.text_for_app(app))
            .unwrap_or_else(|| self.name.clone());
        vec![
            Cell::new(super::common::format_sync_id_for_app(&self.id, app)),
            Cell::new(name),
        ]
    }

    fn row_for_locale(&self, locale: LocaleId) -> Vec<Cell> {
        let name = self
            .name_fallback
            .map(|fallback| fallback.text_for_locale(locale))
            .unwrap_or_else(|| self.name.clone());
        vec![Cell::new(&self.id), Cell::new(name)]
    }
}

#[cfg(test)]
#[path = "profiles_tests.rs"]
mod tests;
