use std::sync::LazyLock;

use anyhow::Context as _;
use parking_lot::RwLock;
use warp_localization::{
    native_locale_candidates, replace_placeholders, AppLanguage, Catalog, CatalogBundle, LocaleId,
};
use warpui_core::AppContext;

static CATALOGS: LazyLock<CatalogBundle> = LazyLock::new(|| {
    let catalogs = [LocaleId::EnUs, LocaleId::ZhCn]
        .into_iter()
        .map(load_catalog)
        .collect::<anyhow::Result<Vec<_>>>()
        .expect("bundled TUI localization catalogs must be valid");

    CatalogBundle::new(LocaleId::EnUs, catalogs)
        .expect("default TUI localization catalog must be bundled")
});

static CURRENT_LOCALE: LazyLock<RwLock<LocaleId>> =
    LazyLock::new(|| RwLock::new(environment_locale()));

pub(crate) fn text(key: &str) -> String {
    text_for_locale(current_locale(), key)
}

pub(crate) fn text_with_args(key: &str, args: &[(&str, &str)]) -> String {
    text_with_args_for_locale(current_locale(), key, args)
}

pub(crate) fn text_for_locale(locale: LocaleId, key: &str) -> String {
    CATALOGS.text(locale, key).into_owned()
}

pub(crate) fn text_with_args_for_locale(
    locale: LocaleId,
    key: &str,
    args: &[(&str, &str)],
) -> String {
    replace_placeholders(&text_for_locale(locale, key), args)
        .expect("localized TUI text template arguments must match the catalog")
}

pub(crate) fn current_locale() -> LocaleId {
    *CURRENT_LOCALE.read()
}

pub(crate) fn sync_from_app(app: &AppContext) -> bool {
    replace_current_locale(&CURRENT_LOCALE, warp::tui_export::current_locale(app))
}

fn environment_locale() -> LocaleId {
    AppLanguage::System.effective_locale_from_candidates(
        environment_locale_candidates()
            .into_iter()
            .chain(native_locale_candidates()),
    )
}

fn replace_current_locale(cache: &RwLock<LocaleId>, locale: LocaleId) -> bool {
    let mut current = cache.write();
    if *current == locale {
        false
    } else {
        *current = locale;
        true
    }
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
    let source = if locale == LocaleId::EnUs {
        include_str!("../../../app/assets/bundled/locales/en-US.json")
    } else {
        include_str!("../../../app/assets/bundled/locales/zh-CN.json")
    };

    Catalog::from_json(locale, source).with_context(|| format!("invalid {path}"))
}

#[cfg(test)]
#[path = "localization_tests.rs"]
mod tests;
