use std::ffi::OsString;

use serial_test::serial;

use super::*;

fn collect_missing_help_keys(
    command: &clap::Command,
    parent_key: &str,
    missing: &mut Vec<(String, String)>,
) {
    if command.is_hide_set() {
        return;
    }

    let command_key = command_key(parent_key, command.get_name());
    if let Some(about) = command.get_about() {
        collect_missing_help_key(
            &format!("cli.help.command.{command_key}.about"),
            &about.to_string(),
            missing,
        );
    }
    if let Some(long_about) = command.get_long_about() {
        collect_missing_help_key(
            &format!("cli.help.command.{command_key}.long_about"),
            &long_about.to_string(),
            missing,
        );
    }

    for arg in command.get_arguments().filter(|arg| !arg.is_hide_set()) {
        let arg_key = arg.get_id().as_str();
        if let Some(help) = arg.get_help() {
            collect_missing_help_key(
                &format!("cli.help.command.{command_key}.arg.{arg_key}.help"),
                &help.to_string(),
                missing,
            );
        }
        if let Some(long_help) = arg.get_long_help() {
            collect_missing_help_key(
                &format!("cli.help.command.{command_key}.arg.{arg_key}.long_help"),
                &long_help.to_string(),
                missing,
            );
        }
    }

    for subcommand in command.get_subcommands() {
        collect_missing_help_keys(subcommand, &command_key, missing);
    }
}

fn collect_missing_help_key(key: &str, english: &str, missing: &mut Vec<(String, String)>) {
    if optional_help_text(LocaleId::ZhCn, key, Some(english)).is_none() {
        missing.push((key.to_owned(), english.to_owned()));
    }
}

fn set_env_var(name: &str, value: &str) -> Option<OsString> {
    let previous = std::env::var_os(name);
    // Safety: tests that mutate process environment are marked `serial` so we
    // do not race with other environment readers or writers in this crate.
    unsafe { std::env::set_var(name, value) };
    previous
}

fn restore_env_var(name: &str, previous: Option<OsString>) {
    match previous {
        // Safety: tests that mutate process environment are marked `serial` so
        // we do not race with other environment readers or writers in this crate.
        Some(value) => unsafe { std::env::set_var(name, value) },
        // Safety: tests that mutate process environment are marked `serial` so
        // we do not race with other environment readers or writers in this crate.
        None => unsafe { std::env::remove_var(name) },
    }
}

fn rendered_help() -> String {
    render_help_for(crate::Args::clap_command())
}

fn rendered_subcommand_help(path: &[&str]) -> String {
    let mut command = crate::Args::clap_command();
    for name in path {
        command = command
            .find_subcommand(name)
            .unwrap_or_else(|| panic!("subcommand {name} should exist"))
            .clone();
    }
    render_help_for(command)
}

fn rendered_control_help() -> String {
    let mut command = crate::local_control::ControlArgs::clap_command().disable_colored_help(true);
    let mut output = Vec::new();
    command
        .write_help(&mut output)
        .expect("control help should render");
    String::from_utf8(output).expect("control help should be valid UTF-8")
}

fn render_help_for(command: clap::Command) -> String {
    let mut command = command.disable_colored_help(true);
    let mut output = Vec::new();
    command.write_help(&mut output).expect("help should render");
    String::from_utf8(output).expect("help should be valid UTF-8")
}

#[test]
fn environment_locale_candidates_prioritize_language_override() {
    let candidates = environment_locale_candidates_from(|key| match key {
        "LANGUAGE" => Some("zh_CN:fr_FR::en_US".to_owned()),
        "LC_ALL" => Some("de_DE.UTF-8".to_owned()),
        "LC_MESSAGES" => Some("it_IT.UTF-8".to_owned()),
        "LANG" => Some("en_GB.UTF-8".to_owned()),
        _ => None,
    });

    assert_eq!(
        candidates,
        vec![
            "zh_CN",
            "fr_FR",
            "en_US",
            "de_DE.UTF-8",
            "it_IT.UTF-8",
            "en_GB.UTF-8",
        ]
    );
}

#[test]
fn visible_cli_help_has_simplified_chinese_catalog_entries() {
    let mut missing = Vec::new();
    collect_missing_help_keys(
        &<crate::Args as clap::CommandFactory>::command(),
        "",
        &mut missing,
    );
    collect_missing_help_keys(
        &<crate::local_control::ControlArgs as clap::CommandFactory>::command(),
        "",
        &mut missing,
    );
    missing.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
    missing.dedup_by(|left, right| left.1 == right.1);

    assert!(
        missing.is_empty(),
        "visible CLI help is missing Simplified Chinese catalog entries:\n{}",
        missing
            .into_iter()
            .map(|(key, english)| format!("{key} = {english:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
#[serial]
fn clap_help_uses_simplified_chinese_catalog() {
    warp_core::features::mark_initialized();
    let previous_language = set_env_var("LANGUAGE", "zh_CN");

    let help = rendered_help();

    restore_env_var("LANGUAGE", previous_language);

    assert!(help.contains("面向云端 Agent 的编排平台"));
    assert!(help.contains("用法:"));
    assert!(help.contains("命令:"));
    assert!(help.contains("选项:"));
    assert!(help.contains("启用调试日志"));
    assert!(help.contains("示例："));
    assert!(help.contains("管理 Agent。"));
    assert!(!help.contains("The orchestration platform for cloud agents"));
    assert!(!help.contains("Manage agents."));
}

#[test]
#[serial]
fn clap_agent_help_uses_simplified_chinese_catalog() {
    warp_core::features::mark_initialized();
    let previous_language = set_env_var("LANGUAGE", "zh_CN");

    let agent_help = rendered_subcommand_help(&["agent"]);
    let run_help = rendered_subcommand_help(&["agent", "run"]);

    restore_env_var("LANGUAGE", previous_language);

    assert!(agent_help.contains("运行新的 Oz Agent。"));
    assert!(run_help.contains("用法:"));
    assert!(!agent_help.contains("Run a new Oz agent."));
}

#[test]
#[serial]
fn warpctrl_help_uses_simplified_chinese_catalog() {
    let previous_language = set_env_var("LANGUAGE", "zh_CN");

    let help = rendered_control_help();

    restore_env_var("LANGUAGE", previous_language);

    assert!(help.contains("控制正在运行的本地 Warp 应用实例"));
    assert!(help.contains("示例："));
    assert!(help.contains("命令:"));
    assert!(help.contains("选项:"));
    assert!(help.contains("了解更多："));
    assert!(!help.contains("Control a running local Warp app instance"));
    assert!(!help.contains("Examples:"));
    assert!(!help.contains("Learn more:"));
}

#[test]
fn disabled_subcommand_error_uses_catalog_template() {
    let error = replace_placeholders(
        &CATALOGS.text(LocaleId::ZhCn, "cli.error.unrecognized_subcommand"),
        &[("subcommand", "environment")],
    )
    .expect("disabled subcommand template should accept subcommand");

    assert_eq!(error, "错误：无法识别子命令 'environment'");
}
