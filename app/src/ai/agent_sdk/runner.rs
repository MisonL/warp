use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use comfy_table::Cell;
use serde::Serialize;
use warp_cli::GlobalOptions;
use warp_cli::agent::OutputFormat;
use warp_cli::runner::{
    CreateRunnerArgs, DeleteRunnerArgs, ListRunnersArgs, RunnerArchArg, RunnerCommand,
    RunnerMacosVersionArg, RunnerOsArg, RunnerSortByArg, UpdateRunnerArgs, validate_os_config,
};
use warp_graphql::mutations::upsert_runner::{
    LinuxConfigInput, MacOsConfigInput, RunnerInput, RunnerInstanceShapeInput, UpsertRunnerInput,
};
use warp_graphql::object::SpaceType;
use warp_graphql::object_permissions::Owner as GqlOwner;
use warp_graphql::queries::get_runners::{
    Runner, RunnerArch, RunnerConfig, RunnerMacOsVersion, RunnerOs, RunnerSortBy,
};
use warp_localization::LocaleId;
use warpui::platform::TerminationMode;
use warpui::{AppContext, ModelContext, SingletonEntity};

use super::output::{self, TableFormat};
use crate::ai::runner_display::{arch_display, macos_version_display, os_display};
use crate::localization;
use crate::server::server_api::ServerApiProvider;
use crate::util::time_format::format_approx_duration_from_now_utc;

/// Handle runner-related CLI commands.
pub fn run(
    ctx: &mut AppContext,
    global_options: GlobalOptions,
    command: RunnerCommand,
) -> Result<()> {
    let runner = ctx.add_singleton_model(|_ctx| RunnerCommandRunner);
    match command {
        RunnerCommand::List(args) => {
            runner.update(ctx, |runner, ctx| {
                runner.list(global_options.output_format, args, ctx)
            });
            Ok(())
        }
        RunnerCommand::Create(args) => {
            runner.update(ctx, |runner, ctx| {
                runner.create(global_options.output_format, args, ctx)
            });
            Ok(())
        }
        RunnerCommand::Update(args) => {
            runner.update(ctx, |runner, ctx| {
                runner.update_runner(global_options.output_format, args, ctx)
            });
            Ok(())
        }
        RunnerCommand::Delete(args) => {
            runner.update(ctx, |runner, ctx| runner.delete(args, ctx));
            Ok(())
        }
    }
}

/// Singleton model for running async work as part of runner CLI commands.
struct RunnerCommandRunner;

impl RunnerCommandRunner {
    fn list(
        &self,
        output_format: OutputFormat,
        args: ListRunnersArgs,
        ctx: &mut ModelContext<Self>,
    ) {
        let factory = ServerApiProvider::as_ref(ctx).get_factory_client();
        let sort_by = args.sort_by.map(sort_by_to_gql);
        let locale = localization::current_locale(ctx);

        ctx.spawn(
            async move {
                let runners = factory.get_runners(sort_by).await?;

                let infos: Vec<RunnerInfo> = runners.into_iter().map(RunnerInfo::from).collect();
                if args.json_output.force_json_output() {
                    output::print_raw_json_for_locale(
                        serde_json::to_value(&infos)?,
                        &args.json_output,
                        locale,
                    )?;
                } else {
                    output::write_list_for_locale(infos, output_format, std::io::stdout(), locale)?;
                }
                Ok(())
            },
            |_, result: Result<()>, ctx| finish_command(result, ctx),
        );
    }

    fn create(
        &self,
        output_format: OutputFormat,
        args: CreateRunnerArgs,
        ctx: &mut ModelContext<Self>,
    ) {
        let locale = localization::current_locale(ctx);
        // Mirror the server rule client-side so users get a fast, clear error.
        if let Err(err) = validate_os_config_for_locale(
            args.os,
            args.docker_image.as_deref(),
            args.macos_version,
            locale,
        ) {
            super::report_fatal_error(err, ctx);
            return;
        }

        // Refresh team metadata so owner resolution can default to the team.
        let refresh = super::common::refresh_workspace_metadata(ctx, locale);
        ctx.spawn(refresh, move |_, result, ctx| {
            if let Err(err) = result {
                super::report_fatal_error(err, ctx);
                return;
            }

            let owner =
                match super::common::resolve_owner(args.scope.team, args.scope.personal, ctx) {
                    Ok(owner) => owner,
                    Err(e) => {
                        super::report_fatal_error(e, ctx);
                        return;
                    }
                };

            let factory = ServerApiProvider::as_ref(ctx).get_factory_client();
            let input = build_create_input(args, owner.into());

            ctx.spawn(
                async move {
                    let upserted = factory.upsert_runner(input).await?;
                    print_upsert_result(
                        &upserted.runner,
                        upserted.is_update,
                        output_format,
                        locale,
                    )?;
                    Ok(())
                },
                |_, result: Result<()>, ctx| finish_command(result, ctx),
            );
        });
    }

    fn update_runner(
        &self,
        output_format: OutputFormat,
        args: UpdateRunnerArgs,
        ctx: &mut ModelContext<Self>,
    ) {
        let factory = ServerApiProvider::as_ref(ctx).get_factory_client();
        let locale = localization::current_locale(ctx);

        ctx.spawn(
            async move {
                // Fetch existing runners so we can resolve the target and preserve
                // any fields that aren't being changed (the server upsert takes a
                // full runner config).
                let runners = factory.get_runners(None).await?;

                let existing =
                    resolve_runner(&runners, args.id.as_deref(), args.name.as_deref(), locale)?;
                let uid = existing.uid.inner().to_string();

                let runner = build_update_input(&args, &existing.config, locale)?;

                let input = UpsertRunnerInput {
                    uid: Some(cynic::Id::new(uid)),
                    owner: None,
                    runner,
                };
                let upserted = factory.upsert_runner(input).await?;
                print_upsert_result(&upserted.runner, upserted.is_update, output_format, locale)?;
                Ok(())
            },
            |_, result: Result<()>, ctx| finish_command(result, ctx),
        );
    }

    fn delete(&self, args: DeleteRunnerArgs, ctx: &mut ModelContext<Self>) {
        use std::io::IsTerminal as _;

        let locale = localization::current_locale(ctx);

        if !args.force {
            match confirm_delete(&args.id, std::io::stdin().is_terminal(), locale) {
                Ok(true) => {}
                Ok(false) => {
                    // Interactive decline: not an error, exit cleanly.
                    ctx.terminate_app(TerminationMode::ForceTerminate, None);
                    return;
                }
                Err(e) => {
                    // A non-interactive refusal is a failure, so automation
                    // doesn't mistake a skipped delete for a successful one.
                    super::report_fatal_error(e, ctx);
                    return;
                }
            }
        }

        let factory = ServerApiProvider::as_ref(ctx).get_factory_client();
        let uid = args.id;

        ctx.spawn(
            async move {
                let deleted_uid = factory.delete_runner(uid).await?;
                println!(
                    "{}",
                    localization::text_for_locale_with_args(
                        locale,
                        "agent_sdk.runner.output.deleted",
                        &[("uid", &deleted_uid)],
                    )
                );
                Ok(())
            },
            |_, result: Result<()>, ctx| finish_command(result, ctx),
        );
    }
}

impl warpui::Entity for RunnerCommandRunner {
    type Event = ();
}
impl SingletonEntity for RunnerCommandRunner {}

/// Prompt the user to confirm deletion of a runner.
///
/// Returns `Ok(true)`/`Ok(false)` for an interactive confirm/decline. In
/// non-interactive mode (no TTY) without `--force`, returns `Err` so the caller
/// fails loudly (non-zero exit) instead of silently skipping the delete.
fn confirm_delete(uid: &str, is_terminal: bool, locale: LocaleId) -> Result<bool> {
    if !is_terminal {
        return Err(anyhow!(localization::text_for_locale_with_args(
            locale,
            "agent_sdk.runner.error.delete_non_interactive_requires_force",
            &[("uid", uid)],
        )));
    }

    Ok(
        inquire::Confirm::new(&localization::text_for_locale_with_args(
            locale,
            "agent_sdk.runner.confirm.delete",
            &[("uid", uid)],
        ))
        .with_default(false)
        .prompt()
        .unwrap_or_default(),
    )
}

/// Resolve a runner by UID or (unambiguous) name from a fetched list.
fn resolve_runner<'a>(
    runners: &'a [Runner],
    id: Option<&str>,
    name: Option<&str>,
    locale: LocaleId,
) -> Result<&'a Runner> {
    if let Some(id) = id {
        return runners
            .iter()
            .find(|runner| runner.uid.inner() == id)
            .ok_or_else(|| {
                anyhow!(localization::text_for_locale_with_args(
                    locale,
                    "agent_sdk.runner.error.not_found",
                    &[("identifier", id)],
                ))
            });
    }

    let name = name.ok_or_else(|| {
        anyhow!(localization::text_for_locale(
            locale,
            "agent_sdk.runner.error.identifier_required",
        ))
    })?;
    let matches: Vec<&Runner> = runners
        .iter()
        .filter(|runner| runner.config.name == name)
        .collect();
    match matches.as_slice() {
        [] => Err(anyhow!(localization::text_for_locale_with_args(
            locale,
            "agent_sdk.runner.error.not_found",
            &[("identifier", name)],
        ))),
        [runner] => Ok(runner),
        _ => Err(anyhow!(localization::text_for_locale_with_args(
            locale,
            "agent_sdk.runner.error.multiple_matches_specify_uid",
            &[("name", name)],
        ))),
    }
}

/// Build the [`RunnerInput`] for a create operation.
fn build_create_input(args: CreateRunnerArgs, owner: GqlOwner) -> UpsertRunnerInput {
    let os = os_to_gql(args.os);
    let (linux, mac) = match args.os {
        RunnerOsArg::Linux => (
            args.docker_image
                .map(|docker_image| LinuxConfigInput { docker_image }),
            None,
        ),
        RunnerOsArg::Macos => (
            None,
            Some(MacOsConfigInput {
                version: args.macos_version.map(macos_version_to_gql),
            }),
        ),
    };

    let instance_shape = match (args.vcpus, args.memory_gb) {
        (Some(vcpus), Some(memory_gb)) => Some(RunnerInstanceShapeInput { vcpus, memory_gb }),
        _ => None,
    };

    let setup_commands = if args.setup_command.is_empty() {
        None
    } else {
        Some(args.setup_command)
    };

    UpsertRunnerInput {
        uid: None,
        owner: Some(owner),
        runner: RunnerInput {
            name: args.name,
            description: args.description,
            setup_commands,
            instance_shape,
            os: Some(os),
            arch: Some(resolve_arch(args.arch, args.os)),
            mac,
            linux,
        },
    }
}

/// Build the [`RunnerInput`] for an update operation, preserving existing
/// config fields that aren't being changed.
fn build_update_input(
    args: &UpdateRunnerArgs,
    existing: &RunnerConfig,
    locale: LocaleId,
) -> Result<RunnerInput> {
    let effective_os = args.os.map(os_to_gql).unwrap_or(existing.os);

    // Validate against the effective OS (the new one if provided, else existing).
    let effective_os_arg = match effective_os {
        RunnerOs::Linux => RunnerOsArg::Linux,
        RunnerOs::Macos => RunnerOsArg::Macos,
    };
    validate_os_config_for_locale(
        effective_os_arg,
        args.docker_image.as_deref(),
        args.macos_version,
        locale,
    )?;

    let (linux, mac) = match effective_os {
        RunnerOs::Linux => {
            let docker_image = args
                .docker_image
                .clone()
                .or_else(|| existing.linux.as_ref().map(|l| l.docker_image.clone()));
            (
                docker_image.map(|docker_image| LinuxConfigInput { docker_image }),
                None,
            )
        }
        RunnerOs::Macos => {
            let version = args
                .macos_version
                .map(macos_version_to_gql)
                .or_else(|| existing.mac.as_ref().and_then(|m| m.version));
            (None, Some(MacOsConfigInput { version }))
        }
    };

    // vCPUs and memory can be updated independently; each unspecified dimension
    // is preserved from the existing shape.
    let existing_shape = existing
        .instance_shape
        .as_ref()
        .map(|shape| (shape.vcpus, shape.memory_gb));
    let instance_shape = merge_instance_shape(args.vcpus, args.memory_gb, existing_shape, locale)?
        .map(|(vcpus, memory_gb)| RunnerInstanceShapeInput { vcpus, memory_gb });

    let setup_commands = if args.setup_command.is_empty() {
        existing.setup_commands.clone()
    } else {
        Some(args.setup_command.clone())
    };

    let description = args
        .description
        .clone()
        .or_else(|| existing.description.clone());

    Ok(RunnerInput {
        name: resolve_updated_name(args.id.is_some(), args.name.as_deref(), &existing.name),
        description,
        setup_commands,
        instance_shape,
        os: Some(effective_os),
        arch: Some(match args.arch {
            Some(arch) => resolve_arch(arch, effective_os_arg),
            None => existing.arch,
        }),
        mac,
        linux,
    })
}

/// Determine the runner's name for an update. A UID identifies the runner
/// directly, so a `--name` given alongside it is a rename. Without a UID,
/// `--name` is the lookup selector, so the name is left unchanged.
fn resolve_updated_name(has_uid: bool, new_name: Option<&str>, existing_name: &str) -> String {
    match (has_uid, new_name) {
        (true, Some(name)) => name.to_string(),
        _ => existing_name.to_string(),
    }
}

/// Merge instance-shape overrides with the existing shape (`(vcpus, memory_gb)`),
/// allowing vCPUs and memory to be updated independently. Returns `None` when
/// there is no shape to set, or an error when only one dimension is provided for
/// a runner that has no existing shape to supply the other.
fn merge_instance_shape(
    new_vcpus: Option<i32>,
    new_memory_gb: Option<i32>,
    existing: Option<(i32, i32)>,
    locale: LocaleId,
) -> Result<Option<(i32, i32)>> {
    if new_vcpus.is_none() && new_memory_gb.is_none() {
        return Ok(existing);
    }
    let vcpus = new_vcpus.or(existing.map(|(v, _)| v)).ok_or_else(|| {
        anyhow!(localization::text_for_locale(
            locale,
            "agent_sdk.runner.error.vcpus_required",
        ))
    })?;
    let memory_gb = new_memory_gb.or(existing.map(|(_, m)| m)).ok_or_else(|| {
        anyhow!(localization::text_for_locale(
            locale,
            "agent_sdk.runner.error.memory_required",
        ))
    })?;
    Ok(Some((vcpus, memory_gb)))
}

fn validate_os_config_for_locale(
    os: RunnerOsArg,
    docker_image: Option<&str>,
    macos_version: Option<RunnerMacosVersionArg>,
    locale: LocaleId,
) -> Result<()> {
    validate_os_config(os, docker_image, macos_version).map_err(|_| {
        let key = match (os, docker_image.is_some(), macos_version.is_some()) {
            (RunnerOsArg::Linux, _, true) => "agent_sdk.runner.error.macos_version_requires_macos",
            (RunnerOsArg::Macos, true, _) => "agent_sdk.runner.error.docker_image_requires_linux",
            (RunnerOsArg::Linux, _, false) | (RunnerOsArg::Macos, false, _) => {
                "agent_sdk.runner.error.invalid_os_configuration"
            }
        };
        anyhow!(localization::text_for_locale(locale, key))
    })
}

fn os_to_gql(os: RunnerOsArg) -> RunnerOs {
    match os {
        RunnerOsArg::Linux => RunnerOs::Linux,
        RunnerOsArg::Macos => RunnerOs::Macos,
    }
}

/// Resolve a [`RunnerArchArg`] into a concrete [`RunnerArch`], mapping `auto`
/// to the default architecture for the given OS (x86-64 on Linux, aarch64 on
/// macOS).
fn resolve_arch(arch: RunnerArchArg, os: RunnerOsArg) -> RunnerArch {
    match arch {
        RunnerArchArg::X8664 => RunnerArch::X8664,
        RunnerArchArg::Aarch64 => RunnerArch::Aarch64,
        RunnerArchArg::Auto => match os {
            RunnerOsArg::Linux => RunnerArch::X8664,
            RunnerOsArg::Macos => RunnerArch::Aarch64,
        },
    }
}

fn macos_version_to_gql(version: RunnerMacosVersionArg) -> RunnerMacOsVersion {
    match version {
        RunnerMacosVersionArg::Macos14 => RunnerMacOsVersion::Macos14,
        RunnerMacosVersionArg::Macos15 => RunnerMacOsVersion::Macos15,
        RunnerMacosVersionArg::Macos26 => RunnerMacOsVersion::Macos26,
        RunnerMacosVersionArg::Macos27 => RunnerMacOsVersion::Macos27,
    }
}

fn sort_by_to_gql(sort_by: RunnerSortByArg) -> RunnerSortBy {
    match sort_by {
        RunnerSortByArg::Name => RunnerSortBy::Name,
        RunnerSortByArg::LastUpdated => RunnerSortBy::LastUpdated,
    }
}

fn space_display(space_type: SpaceType) -> &'static str {
    match space_type {
        SpaceType::Team => "Team",
        SpaceType::User => "Personal",
    }
}

fn print_upsert_result(
    runner: &Runner,
    is_update: bool,
    output_format: OutputFormat,
    locale: LocaleId,
) -> Result<()> {
    let info = RunnerInfo::from_ref(runner);
    match output_format {
        OutputFormat::Json => output::write_json_for_locale(&info, std::io::stdout(), locale)?,
        OutputFormat::Ndjson => {
            output::write_json_line_for_locale(&info, std::io::stdout(), locale)?
        }
        OutputFormat::Pretty | OutputFormat::Text => {
            let key = if is_update {
                "agent_sdk.runner.output.updated"
            } else {
                "agent_sdk.runner.output.created"
            };
            println!(
                "{}",
                localization::text_for_locale_with_args(locale, key, &[("uid", &info.uid)],)
            );
        }
    }
    Ok(())
}

fn finish_command(result: Result<()>, ctx: &mut ModelContext<RunnerCommandRunner>) {
    match result {
        Ok(()) => ctx.terminate_app(TerminationMode::ForceTerminate, None),
        Err(err) => super::report_fatal_error(err, ctx),
    }
}

/// Runner information shown in the `list` command and in JSON output.
#[derive(Serialize)]
struct RunnerInfo {
    uid: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    os: String,
    arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    vcpus: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_gb: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docker_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    macos_version: Option<String>,
    scope: String,
    setup_commands: Vec<String>,
    #[serde(skip_serializing)]
    last_updated_display: String,
    #[serde(rename = "last_updated")]
    last_updated_utc: DateTime<Utc>,
}

impl RunnerInfo {
    fn from_ref(runner: &Runner) -> Self {
        let config = &runner.config;
        let (vcpus, memory_gb) = config
            .instance_shape
            .as_ref()
            .map(|shape| (shape.vcpus, shape.memory_gb))
            .unzip();
        let last_updated_utc = runner.last_updated.utc();

        RunnerInfo {
            uid: runner.uid.inner().to_string(),
            name: config.name.clone(),
            description: config.description.clone(),
            os: os_display(config.os).to_string(),
            arch: arch_display(config.arch).to_string(),
            vcpus,
            memory_gb,
            docker_image: config.linux.as_ref().map(|l| l.docker_image.clone()),
            macos_version: config
                .mac
                .as_ref()
                .and_then(|m| m.version)
                .map(|v| macos_version_display(v).to_string()),
            scope: space_display(runner.scope.type_).to_string(),
            setup_commands: config.setup_commands.clone().unwrap_or_default(),
            last_updated_display: format_approx_duration_from_now_utc(last_updated_utc),
            last_updated_utc,
        }
    }

    /// The OS-specific config value shown in the combined "OS Settings" column,
    /// labeled with which kind of setting it is (empty when neither applies).
    fn os_specific_display_for_locale(&self, locale: LocaleId) -> String {
        if let Some(image) = &self.docker_image {
            localization::text_for_locale_with_args(
                locale,
                "agent_sdk.runner.value.docker_image",
                &[("image", image)],
            )
        } else if let Some(version) = &self.macos_version {
            localization::text_for_locale_with_args(
                locale,
                "agent_sdk.runner.value.macos_version",
                &[("version", version)],
            )
        } else {
            String::new()
        }
    }

    /// The instance-shape value shown in the table.
    fn shape_display_for_locale(&self, locale: LocaleId) -> String {
        match (self.vcpus, self.memory_gb) {
            (Some(vcpus), Some(memory_gb)) => localization::text_for_locale_with_args(
                locale,
                "agent_sdk.runner.value.shape",
                &[
                    ("vcpus", &vcpus.to_string()),
                    ("memory_gb", &memory_gb.to_string()),
                ],
            ),
            _ => localization::text_for_locale(locale, "agent_sdk.runner.value.default"),
        }
    }
}

impl From<Runner> for RunnerInfo {
    fn from(runner: Runner) -> Self {
        RunnerInfo::from_ref(&runner)
    }
}

impl TableFormat for RunnerInfo {
    fn header() -> Vec<Cell> {
        Self::header_for_locale(LocaleId::EnUs)
    }

    fn header_for_locale(locale: LocaleId) -> Vec<Cell> {
        runner_info_header_for_locale(locale)
    }

    fn row(&self) -> Vec<Cell> {
        self.row_for_locale(LocaleId::EnUs)
    }

    fn row_for_locale(&self, locale: LocaleId) -> Vec<Cell> {
        vec![
            Cell::new(&self.uid),
            Cell::new(&self.name),
            Cell::new(self.description.as_deref().unwrap_or("")),
            Cell::new(self.shape_display_for_locale(locale)),
            Cell::new(localized_runner_os(locale, &self.os)),
            Cell::new(localized_runner_arch(locale, &self.arch)),
            Cell::new(self.os_specific_display_for_locale(locale)),
            Cell::new(localized_scope(locale, &self.scope)),
            Cell::new(&self.last_updated_display),
        ]
    }
}

fn runner_info_header_for_locale(locale: LocaleId) -> Vec<Cell> {
    vec![
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.runner.table.uid",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.runner.table.name",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.runner.table.description",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.runner.table.shape",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.runner.table.os",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.runner.table.arch",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.runner.table.os_settings",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.runner.table.scope",
        )),
        Cell::new(localization::text_for_locale(
            locale,
            "agent_sdk.runner.table.last_updated",
        )),
    ]
}

fn localized_scope(locale: LocaleId, scope: &str) -> String {
    match scope {
        "Personal" => localization::text_for_locale(locale, "agent_sdk.secret.scope.personal"),
        "Team" => localization::text_for_locale(locale, "agent_sdk.secret.scope.team"),
        _ => scope.to_owned(),
    }
}

fn localized_runner_os(locale: LocaleId, os: &str) -> String {
    let key = match os {
        "Linux" => "agent_sdk.runner.value.os.linux",
        "macOS" => "agent_sdk.runner.value.os.macos",
        _ => return os.to_owned(),
    };
    localization::text_for_locale(locale, key)
}

fn localized_runner_arch(locale: LocaleId, arch: &str) -> String {
    let key = match arch {
        "x86-64" => "agent_sdk.runner.value.arch.x86_64",
        "aarch64" => "agent_sdk.runner.value.arch.aarch64",
        _ => return arch.to_owned(),
    };
    localization::text_for_locale(locale, key)
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
