use comfy_table::Cell;
use serde::Serialize;
use warp_cli::GlobalOptions;
use warp_cli::agent::AgentProfileCommand;
use warp_localization::LocaleId;
use warpui::{AppContext, ModelContext, SingletonEntity};

use crate::ai::agent_sdk::output::{self, TableFormat};
use crate::ai::execution_profiles::execution_profile_display_name_key;
use crate::ai::execution_profiles::profiles::AIExecutionProfilesModel;
use crate::cloud_object::model::generic_string_model::StringModel;
use crate::localization;
use crate::server::cloud_objects::update_manager::UpdateManager;
use crate::server::ids::SyncId;

const UNSYNCED_PROFILE_ID: &str = super::common::CANONICAL_UNSYNCED;

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
        let locale = localization::current_locale(ctx);

        ctx.spawn(initial_sync, move |_, _, ctx| {
            let profiles_model = AIExecutionProfilesModel::as_ref(ctx);

            let profile_ids = profiles_model.get_all_profile_ids();

            let profiles: Vec<_> = profile_ids
                .iter()
                .flat_map(|id| profiles_model.get_profile_by_id(id, ctx))
                .map(|profile| {
                    let profile_data = profile.data();
                    let name = profile_data.display_name();
                    let name_key = execution_profile_display_name_key(profile_data);
                    let id = match profile.sync_id() {
                        Some(SyncId::ServerId(server_id)) => server_id.to_string(),
                        _ => UNSYNCED_PROFILE_ID.to_owned(),
                    };
                    ProfileInfo { id, name, name_key }
                })
                .collect();

            output::print_list_for_locale(profiles, global_options.output_format, locale);

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
    name_key: Option<&'static str>,
}

impl TableFormat for ProfileInfo {
    fn header() -> Vec<Cell> {
        vec![Cell::new("ID"), Cell::new("Name")]
    }

    fn header_for_locale(locale: LocaleId) -> Vec<Cell> {
        vec![
            Cell::new(localization::text_for_locale(
                locale,
                "agent_sdk.profiles.table.id",
            )),
            Cell::new(localization::text_for_locale(
                locale,
                "agent_sdk.profiles.table.name",
            )),
        ]
    }

    fn row(&self) -> Vec<Cell> {
        vec![Cell::new(&self.id), Cell::new(&self.name)]
    }

    fn row_for_locale(&self, locale: LocaleId) -> Vec<Cell> {
        let id = if self.id == UNSYNCED_PROFILE_ID {
            localization::text_for_locale(locale, "agent_sdk.common.value.unsynced")
        } else {
            self.id.clone()
        };
        let name = self
            .name_key
            .map(|key| localization::text_for_locale(locale, key))
            .unwrap_or_else(|| self.name.clone());
        vec![Cell::new(id), Cell::new(name)]
    }
}

#[cfg(test)]
#[path = "profiles_tests.rs"]
mod tests;
