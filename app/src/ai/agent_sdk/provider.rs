//! Provider command for linking third-party services.
use comfy_table::Cell;
use serde::Serialize;
use warp_cli::provider::{ProviderCommand, ProviderType};
use warp_cli::GlobalOptions;
use warp_core::channel::ChannelState;
use warp_localization::LocaleId;
use warpui::platform::TerminationMode;
use warpui::{AppContext, ModelContext, SingletonEntity};

use crate::ai::agent_sdk::output::{self, TableFormat};
use crate::localization;
use crate::workspaces::user_workspaces::UserWorkspaces;

const PROVIDER_STATUS_NOT_CONNECTED: &str = "Not Connected";

/// Handle provider-related CLI commands.
pub fn run(
    ctx: &mut AppContext,
    global_options: GlobalOptions,
    command: ProviderCommand,
) -> anyhow::Result<()> {
    let runner = ctx.add_singleton_model(|_ctx| ProviderCommandRunner);
    match command {
        ProviderCommand::Setup(args) => runner.update(ctx, |runner, ctx| {
            runner.setup(args.provider_type, args.team, args.personal, ctx)
        }),
        ProviderCommand::List => runner.update(ctx, |runner, ctx| runner.list(global_options, ctx)),
    }
}

/// Singleton model for running provider CLI commands.
struct ProviderCommandRunner;

impl ProviderCommandRunner {
    // This shouldn't need to be done, it's usually done as part of create
    fn setup(
        &self,
        provider_type: ProviderType,
        team: bool,
        personal: bool,
        ctx: &mut ModelContext<Self>,
    ) -> anyhow::Result<()> {
        // Construct the OAuth connect URL
        let server_url = ChannelState::server_root_url();

        let mut use_team_auth = team;
        if !team && !personal {
            if provider_type.allowed_in_team_context()
                && provider_type.allowed_in_personal_context()
            {
                return Err(anyhow::anyhow!(localization::text_for_app_with_args(
                    ctx,
                    "agent_sdk.provider.error.scope_required",
                    &[("provider", &provider_type.slug())]
                )));
            }
            use_team_auth = provider_type.allowed_in_team_context();
        } else if personal {
            use_team_auth = false;
        }

        // TODO(bens): initiate the OAuth flow and use the login-less auth URL
        let slug = provider_type.slug();
        let url = if use_team_auth {
            let team_uid = match UserWorkspaces::as_ref(ctx).current_team_uid() {
                Some(uid) => uid,
                None => {
                    return Err(anyhow::anyhow!(localization::text_for_app(
                        ctx,
                        "agent_sdk.common.error.user_not_on_team"
                    )));
                }
            };
            format!("{server_url}/oauth/connect/{slug}?principalType=team&principalId={team_uid}")
        } else {
            format!("{server_url}/oauth/connect/{slug}")
        };

        println!(
            "{}",
            localization::text_for_app_with_args(
                ctx,
                "agent_sdk.provider.output.authenticate_url",
                &[("provider", &slug), ("url", &url)]
            )
        );

        // Open the URL in the default browser
        ctx.open_url(&url);

        // TODO(bens): poll/subscribe until connection is created

        ctx.terminate_app(TerminationMode::ForceTerminate, None);

        Ok(())
    }

    fn list(
        &self,
        global_options: GlobalOptions,
        ctx: &mut ModelContext<Self>,
    ) -> anyhow::Result<()> {
        let locale = localization::current_locale(ctx);
        let providers = vec![ProviderType::Linear, ProviderType::Slack];

        let provider_infos: Vec<_> = providers
            .into_iter()
            .map(|provider| {
                let name = provider.name();
                let slug = provider.slug();
                let mut allowed_for = Vec::new();

                if provider.allowed_in_personal_context() {
                    allowed_for.push("personal");
                }
                if provider.allowed_in_team_context() {
                    allowed_for.push("team");
                }

                let allowed_str = allowed_for.join(", ");
                let status = PROVIDER_STATUS_NOT_CONNECTED.to_owned(); // TODO(bens): get this from gql

                ProviderInfo {
                    name,
                    slug,
                    allowed_for: allowed_str,
                    status,
                }
            })
            .collect();

        output::write_list_for_locale(
            provider_infos,
            global_options.output_format,
            std::io::stdout(),
            locale,
        )?;

        ctx.terminate_app(TerminationMode::ForceTerminate, None);

        Ok(())
    }
}

impl warpui::Entity for ProviderCommandRunner {
    type Event = ();
}
impl SingletonEntity for ProviderCommandRunner {}

/// Provider information that's shown in the `list` command.
#[derive(Serialize)]
struct ProviderInfo {
    name: String,
    slug: String,
    allowed_for: String,
    status: String,
}

impl TableFormat for ProviderInfo {
    fn header() -> Vec<Cell> {
        provider_info_header_for_locale(LocaleId::EnUs)
    }

    fn header_for_locale(locale: LocaleId) -> Vec<Cell> {
        provider_info_header_for_locale(locale)
    }

    fn row(&self) -> Vec<Cell> {
        vec![
            Cell::new(&self.name),
            Cell::new(&self.slug),
            Cell::new(&self.allowed_for),
            Cell::new(&self.status),
        ]
    }

    fn row_for_locale(&self, locale: LocaleId) -> Vec<Cell> {
        let allowed_for = self
            .allowed_for
            .split(", ")
            .map(|scope| match scope {
                "personal" => {
                    localization::text_for_locale(locale, "agent_sdk.secret.scope.personal")
                }
                "team" => localization::text_for_locale(locale, "agent_sdk.secret.scope.team"),
                scope => scope.to_owned(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let status = if self.status == PROVIDER_STATUS_NOT_CONNECTED {
            localization::text_for_locale(locale, "agent_sdk.provider.status.not_connected")
        } else {
            self.status.clone()
        };
        vec![
            Cell::new(&self.name),
            Cell::new(&self.slug),
            Cell::new(allowed_for),
            Cell::new(status),
        ]
    }
}

fn provider_info_header_for_locale(locale: LocaleId) -> Vec<Cell> {
    vec![
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.provider.table.name",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.provider.table.slug",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.provider.table.allowed_for",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.provider.table.status",
        )),
    ]
}

#[cfg(test)]
#[path = "provider_tests.rs"]
mod tests;
