use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::Context as _;
use parking_lot::RwLock;
use warp_localization::{
    native_locale_candidates, replace_placeholders, AppLanguage, Catalog, CatalogBundle, LocaleId,
    TranslationSource,
};
use warpui::{AppContext, AssetProvider as _, Entity, ModelContext, SingletonEntity as _};

use crate::settings::{LanguageSettings, LanguageSettingsChangedEvent};
use crate::ASSETS;

pub(crate) enum LocalizationEvent {
    LocaleChanged,
}

static CATALOGS: LazyLock<CatalogBundle> = LazyLock::new(|| {
    let catalogs = [LocaleId::EnUs, LocaleId::ZhCn]
        .into_iter()
        .map(load_catalog)
        .collect::<anyhow::Result<Vec<_>>>()
        .expect("bundled localization catalogs must be valid");

    CatalogBundle::new(LocaleId::EnUs, catalogs)
        .expect("default localization catalog must be bundled")
});

static ENGLISH_TEXT_TO_KEY: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    CATALOGS
        .catalog(LocaleId::EnUs)
        .expect("default localization catalog must be bundled")
        .entries()
        .map(|(key, value)| (value.to_owned(), key))
        .collect()
});

static SYSTEM_LOCALE_CANDIDATES: LazyLock<RwLock<Vec<String>>> =
    LazyLock::new(|| RwLock::new(system_locale_candidates()));

pub(crate) fn current_locale(app: &AppContext) -> LocaleId {
    match *LanguageSettings::as_ref(app).app_language {
        AppLanguage::System => {
            let candidates = SYSTEM_LOCALE_CANDIDATES.read();
            AppLanguage::System
                .effective_locale_from_candidates(candidates.iter().map(String::as_str))
        }
        AppLanguage::English => LocaleId::EnUs,
        AppLanguage::SimplifiedChinese => LocaleId::ZhCn,
    }
}

pub(crate) fn register_localization_updater(ctx: &mut AppContext) {
    ctx.add_singleton_model(LocalizationUpdater::new);
}

pub(crate) fn refresh_system_locale_candidates_if_needed(app: &AppContext) -> bool {
    if *LanguageSettings::as_ref(app).app_language == AppLanguage::System {
        refresh_system_locale_candidates()
    } else {
        false
    }
}

pub(crate) fn text_for_app(app: &AppContext, key: &str) -> String {
    text(current_locale(app), key)
}

pub(crate) fn text_for_app_with_args(app: &AppContext, key: &str, args: &[(&str, &str)]) -> String {
    replace_placeholders(&text_for_app(app, key), args)
        .expect("localized text template arguments must match the catalog")
}

pub(crate) fn file_picker_error_for_app(app: &AppContext, error: impl std::fmt::Display) -> String {
    let error = error.to_string();
    text_for_app_with_args(app, "file_picker.error", &[("error", &error)])
}

pub(crate) fn labeled_error_for_app(app: &AppContext, label: &str, message: &str) -> String {
    text_for_app_with_args(
        app,
        "error.labeled",
        &[("label", label), ("message", message)],
    )
}

pub(crate) fn text_for_app_or(app: &AppContext, key: &str, fallback: &str) -> String {
    let lookup = CATALOGS.lookup(current_locale(app), key);
    if lookup.source == TranslationSource::Key {
        fallback.to_owned()
    } else {
        lookup.text.into_owned()
    }
}

pub(crate) fn binding_description_key_for_english_text(text: &str) -> Option<&'static str> {
    ENGLISH_TEXT_TO_KEY.get(text).copied()
}

pub(crate) fn text_for_locale(locale: LocaleId, key: &str) -> String {
    text(locale, key)
}

pub(crate) fn text_for_locale_with_args(
    locale: LocaleId,
    key: &str,
    args: &[(&str, &str)],
) -> String {
    replace_placeholders(&text_for_locale(locale, key), args)
        .expect("localized text template arguments must match the catalog")
}

fn text(locale: LocaleId, key: &str) -> String {
    CATALOGS.text(locale, key).into_owned()
}

fn load_catalog(locale: LocaleId) -> anyhow::Result<Catalog> {
    let path = format!("bundled/locales/{}.json", locale.code());
    let bytes = ASSETS
        .get(&path)
        .with_context(|| format!("failed to load {path}"))?;
    let source = std::str::from_utf8(&bytes)
        .with_context(|| format!("localization catalog {path} is not UTF-8"))?;

    Catalog::from_json(locale, source).with_context(|| format!("invalid {path}"))
}

fn system_locale_candidates() -> Vec<String> {
    native_locale_candidates()
        .into_iter()
        .chain(environment_locale_candidates())
        .collect()
}

fn refresh_system_locale_candidates() -> bool {
    replace_system_locale_candidates(&SYSTEM_LOCALE_CANDIDATES, system_locale_candidates())
}

fn replace_system_locale_candidates(
    cache: &RwLock<Vec<String>>,
    new_candidates: Vec<String>,
) -> bool {
    let mut cached_candidates = cache.write();
    if *cached_candidates == new_candidates {
        false
    } else {
        *cached_candidates = new_candidates;
        true
    }
}

fn environment_locale_candidates() -> impl Iterator<Item = String> {
    environment_locale_candidates_from(|key| std::env::var(key).ok()).into_iter()
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
                    .filter(|candidate| !candidate.is_empty())
            })
        })
        .collect()
}

pub(crate) struct LocalizationUpdater;

impl LocalizationUpdater {
    fn new(ctx: &mut ModelContext<Self>) -> Self {
        ctx.subscribe_to_model(&LanguageSettings::handle(ctx), |_, _, event, ctx| {
            let LanguageSettingsChangedEvent::AppLanguageSetting { .. } = event;
            let _ = refresh_system_locale_candidates_if_needed(ctx);
            notify_locale_changed_from_model(ctx);
        });

        Self
    }
}

impl Entity for LocalizationUpdater {
    type Event = LocalizationEvent;
}

impl warpui::SingletonEntity for LocalizationUpdater {}

fn notify_locale_changed_from_model(ctx: &mut ModelContext<LocalizationUpdater>) {
    ctx.emit(LocalizationEvent::LocaleChanged);
    ctx.invalidate_all_views();
    #[cfg(target_os = "macos")]
    {
        crate::platform::refresh_localized_menus();
    }
}

#[cfg(test)]
#[path = "localization_tests.rs"]
mod tests;
