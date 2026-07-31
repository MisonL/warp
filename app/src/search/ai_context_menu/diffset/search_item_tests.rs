use settings::Setting as _;
use warpui::{App, SingletonEntity as _};

use super::DiffSetSearchItem;
use crate::code_review::diff_state::DiffMode;
use crate::search::ai_context_menu::diffset::data_source::DiffSetDataSource;
use crate::search::data_source::Query;
use crate::search::item::SearchItem;
use crate::search::mixer::SyncDataSource;
use crate::settings::{AppLanguage, LanguageSettings};
use crate::test_util::settings::initialize_localization_for_tests;

#[test]
fn diffset_has_higher_priority_tier() {
    let match_result =
        fuzzy_match::match_indices_case_insensitive("uncommitted changes", "uncommitted")
            .expect("query should match");

    let item = DiffSetSearchItem {
        diff_mode: DiffMode::Head,
        match_result,
    };

    assert_eq!(item.priority_tier(), 1);
}

#[test]
fn diffset_search_and_accessibility_follow_app_language() {
    App::test((), |mut app| async move {
        initialize_localization_for_tests(&mut app);
        app.update(|ctx| {
            LanguageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings
                    .app_language
                    .load_value(AppLanguage::SimplifiedChinese, true, ctx)
                    .expect("language setting should update")
            });
        });

        let data_source = DiffSetDataSource;
        app.read(|ctx| {
            assert_eq!(
                data_source
                    .run_query(&Query::from("未提交"), ctx)
                    .expect("Chinese query should run")
                    .len(),
                1
            );
            assert_eq!(
                data_source
                    .run_query(&Query::from("uncommitted"), ctx)
                    .expect("English alias query should run")
                    .len(),
                1
            );

            let item = DiffSetSearchItem {
                diff_mode: DiffMode::Head,
                match_result: fuzzy_match::match_indices_case_insensitive("未提交的更改", "未提交")
                    .expect("query should match"),
            };
            assert_eq!(
                item.accessibility_label_for_app(ctx),
                "未提交的更改 - 工作目录中的所有未提交更改"
            );
        });
    });
}
