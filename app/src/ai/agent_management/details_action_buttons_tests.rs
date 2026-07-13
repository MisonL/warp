use settings::Setting;
use warp_core::ui::appearance::Appearance;
use warpui::elements::Empty;
use warpui::platform::WindowStyle;
use warpui::{App, AppContext, Element, Entity, SingletonEntity, TypedActionView, View, WindowId};

use super::{AgentDetailsAction, ConversationActionButtonsRow};
use crate::settings::{AppLanguage, LanguageSettings};
use crate::test_util::settings::initialize_settings_for_tests;

#[derive(Default)]
struct TestRootView;

impl Entity for TestRootView {
    type Event = ();
}

impl View for TestRootView {
    fn ui_name() -> &'static str {
        "ConversationActionButtonsTestRoot"
    }

    fn render(&self, _: &AppContext) -> Box<dyn Element> {
        Empty::new().finish()
    }
}

impl TypedActionView for TestRootView {
    type Action = ();
}

fn create_test_window(app: &mut App) -> WindowId {
    let (window_id, _) = app.add_window(WindowStyle::NotStealFocus, |_| TestRootView);
    window_id
}

#[test]
fn action_button_tooltips_follow_language_changes() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        app.add_singleton_model(|_| Appearance::mock());
        let window_id = create_test_window(&mut app);

        app.update(|ctx| {
            LanguageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings
                    .app_language
                    .set_value(AppLanguage::English, ctx)
                    .expect("language setting should update");
            });
        });
        let buttons = app
            .update(|ctx| ctx.add_typed_action_view(window_id, ConversationActionButtonsRow::new));

        app.read(|ctx| {
            for (action, expected) in [
                (AgentDetailsAction::Open, "Open conversation"),
                (AgentDetailsAction::CancelTask, "Cancel task"),
                (AgentDetailsAction::ForkConversation, "Fork conversation"),
                (AgentDetailsAction::ViewDetails, "View details"),
                (AgentDetailsAction::CopyLink, "Copy link to run"),
            ] {
                assert_eq!(
                    buttons.as_ref(ctx).tooltip_for_test(action, ctx).as_deref(),
                    Some(expected)
                );
            }
        });

        app.update(|ctx| {
            LanguageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings
                    .app_language
                    .set_value(AppLanguage::SimplifiedChinese, ctx)
                    .expect("language setting should update");
            });
        });

        app.read(|ctx| {
            for (action, expected) in [
                (AgentDetailsAction::Open, "打开对话"),
                (AgentDetailsAction::CancelTask, "取消任务"),
                (AgentDetailsAction::ForkConversation, "分叉对话"),
                (AgentDetailsAction::ViewDetails, "查看详情"),
                (AgentDetailsAction::CopyLink, "复制运行链接"),
            ] {
                assert_eq!(
                    buttons.as_ref(ctx).tooltip_for_test(action, ctx).as_deref(),
                    Some(expected)
                );
            }
        });
    });
}
