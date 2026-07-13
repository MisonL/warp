use settings::Setting;
use warp_localization::LocaleId;
use warpui::{App, SingletonEntity};

use super::build_from_block_prompt;
use crate::appearance;
use crate::test_util::settings::initialize_settings_for_tests;

fn initialize_app(app: &mut App) {
    initialize_settings_for_tests(app);
    appearance::register(app);
}

#[test]
fn test_from_block_prompt_localizes_to_simplified_chinese() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        app.update(|ctx| {
            crate::settings::LanguageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings
                    .app_language
                    .set_value(crate::settings::AppLanguage::SimplifiedChinese, ctx)
                    .expect("language setting should update");
            });
        });

        app.read(|ctx| {
            let prompt = build_from_block_prompt("ls", "输出内容", true, ctx);
            assert!(prompt.contains("我运行了命令"));
            assert!(prompt.contains("我接下来应该做什么"));
            assert!(!prompt.contains("I ran the command"));
        });
    });
}

#[test]
fn test_from_block_prompt_respects_character_limit_for_cjk_content() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        app.update(|ctx| {
            crate::settings::LanguageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings
                    .app_language
                    .set_value(crate::settings::AppLanguage::English, ctx)
                    .expect("language setting should update");
            });

            assert_eq!(crate::localization::current_locale(ctx), LocaleId::EnUs);

            let input = "命令".repeat(200);
            let output = "输出".repeat(800);
            let prompt = build_from_block_prompt(&input, &output, false, ctx);

            assert!(prompt.chars().count() <= crate::ai_assistant::PROMPT_CHARACTER_LIMIT);
            assert!(prompt.contains("How do I fix this?"));
        });
    });
}
