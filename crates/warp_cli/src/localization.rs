use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::Context as _;
use clap::Command;
use warp_localization::{
    AppLanguage, Catalog, CatalogBundle, LocaleId, TranslationSource, native_locale_candidates,
    replace_placeholders,
};

static CATALOGS: LazyLock<CatalogBundle> = LazyLock::new(|| {
    let catalogs = [LocaleId::EnUs, LocaleId::ZhCn]
        .into_iter()
        .map(load_catalog)
        .collect::<anyhow::Result<Vec<_>>>()
        .expect("bundled CLI localization catalogs must be valid");

    CatalogBundle::new(LocaleId::EnUs, catalogs)
        .expect("default CLI localization catalog must be bundled")
});

static ENGLISH_HELP_TO_KEYS: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let mut keys_by_text = HashMap::<String, Vec<String>>::new();
    for (key, value) in CATALOGS
        .catalog(LocaleId::EnUs)
        .expect("default CLI localization catalog must be bundled")
        .entries()
        .filter(|(key, _)| key.starts_with("cli.help."))
    {
        keys_by_text
            .entry(value.to_owned())
            .or_default()
            .push(key.to_owned());
    }
    keys_by_text
});

pub(crate) fn text(key: &str) -> String {
    CATALOGS.text(current_locale(), key).into_owned()
}

pub(crate) fn text_with_args(key: &str, args: &[(&str, &str)]) -> String {
    replace_placeholders(&text(key), args)
        .expect("localized text template arguments must match the catalog")
}

pub(crate) fn localized_clap_command(command: Command) -> Command {
    localize_command(command, current_locale(), "")
}

fn localize_command(mut command: Command, locale: LocaleId, parent_key: &str) -> Command {
    let command_key = command_key(parent_key, command.get_name());

    command = localize_command_text(command, locale, &command_key);
    command = localize_args(command, locale, &command_key);
    command.mut_subcommands(|subcommand| localize_command(subcommand, locale, &command_key))
}

fn localize_command_text(mut command: Command, locale: LocaleId, command_key: &str) -> Command {
    if let Some(template) = optional_text(locale, "cli.help.template") {
        command = command.help_template(template);
    }
    if let Some(heading) = optional_text(locale, "cli.help.heading.subcommands") {
        command = command.subcommand_help_heading(heading);
    }
    if let Some(heading) = optional_text(locale, "cli.help.heading.options") {
        command = command.next_help_heading(heading);
    }
    let about = command.get_about().map(ToString::to_string);
    if let Some(about) = optional_help_text(
        locale,
        &format!("cli.help.command.{command_key}.about"),
        about.as_deref(),
    ) {
        command = command.about(about);
    }
    let long_about = command.get_long_about().map(ToString::to_string);
    if let Some(long_about) = optional_help_text(
        locale,
        &format!("cli.help.command.{command_key}.long_about"),
        long_about.as_deref(),
    ) {
        command = command.long_about(long_about);
    }
    command
}

fn localize_args(command: Command, locale: LocaleId, command_key: &str) -> Command {
    command.mut_args(|mut arg| {
        match optional_text(locale, "cli.help.heading.options") {
            Some(heading) if !arg.is_positional() => {
                arg = arg.help_heading(heading);
            }
            _ => {}
        }

        let arg_key = arg.get_id().as_str().to_owned();
        let help_key = format!("cli.help.command.{command_key}.arg.{arg_key}.help");
        let help = arg.get_help().map(ToString::to_string);
        if let Some(help) = optional_help_text(locale, &help_key, help.as_deref()) {
            arg = arg.help(help);
        }

        let long_help_key = format!("cli.help.command.{command_key}.arg.{arg_key}.long_help");
        let long_help = arg.get_long_help().map(ToString::to_string);
        if let Some(long_help) = optional_help_text(locale, &long_help_key, long_help.as_deref()) {
            arg = arg.long_help(long_help);
        }

        arg
    })
}

fn command_key(parent_key: &str, name: &str) -> String {
    if parent_key.is_empty() {
        name.to_owned()
    } else {
        format!("{parent_key}.{name}")
    }
}

fn current_locale() -> LocaleId {
    AppLanguage::System.effective_locale_from_candidates(
        environment_locale_candidates()
            .into_iter()
            .chain(native_locale_candidates()),
    )
}

fn environment_locale_candidates() -> Vec<String> {
    environment_locale_candidates_from(|key| std::env::var(key).ok())
}

fn environment_locale_candidates_from(mut get: impl FnMut(&str) -> Option<String>) -> Vec<String> {
    ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .flat_map(|key| {
            get(key).into_iter().flat_map(move |value| {
                let values = if key == "LANGUAGE" {
                    value.split(':').map(str::to_owned).collect::<Vec<_>>()
                } else {
                    vec![value]
                };

                values
                    .into_iter()
                    .map(|candidate| candidate.trim().to_owned())
            })
        })
        .filter(|candidate| !candidate.is_empty())
        .collect()
}

fn load_catalog(locale: LocaleId) -> anyhow::Result<Catalog> {
    let path = format!("../../../app/assets/bundled/locales/{}.json", locale.code());
    let source = include_str!(concat!(
        "../../../app/assets/bundled/locales/",
        "en-US.json"
    ));
    let source = if locale == LocaleId::EnUs {
        source
    } else {
        include_str!(concat!(
            "../../../app/assets/bundled/locales/",
            "zh-CN.json"
        ))
    };

    Catalog::from_json(locale, source).with_context(|| format!("invalid {path}"))
}

fn optional_text(locale: LocaleId, key: &str) -> Option<String> {
    let lookup = CATALOGS.lookup(locale, key);
    if lookup.source == TranslationSource::Key {
        None
    } else {
        Some(lookup.text.into_owned())
    }
}

fn optional_help_text(locale: LocaleId, exact_key: &str, english: Option<&str>) -> Option<String> {
    optional_text(locale, exact_key)
        .or_else(|| {
            english
                .and_then(|text| ENGLISH_HELP_TO_KEYS.get(text))
                .and_then(|keys| keys.iter().find_map(|key| optional_text(locale, key)))
        })
        .filter(|text| !text.trim().is_empty())
}

#[cfg(test)]
#[path = "localization_tests.rs"]
mod tests;
