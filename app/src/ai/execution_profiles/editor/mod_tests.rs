use std::collections::HashMap;

use settings::Setting as _;
use warp_core::features::FeatureFlag;
use warp_localization::LocaleId;
use warpui::{App, SingletonEntity as _};

use super::ui_helpers::context_window_snap_values;
use super::{
    ExecutionProfileEditorView, ExecutionProfileEditorViewAction, updated_profile_name,
    upgrade_footer_link_char_range,
};
use crate::LaunchMode;
use crate::ai::execution_profiles::profiles::AIExecutionProfilesModel;
use crate::ai::execution_profiles::{
    AIExecutionProfile, AIExecutionProfileAppExt as _, has_configurable_context_window,
    should_show_long_context_pricing_warning,
};
use crate::ai::llms::{
    AvailableLLMs, LLMContextWindow, LLMInfo, LLMPreferences, LLMProvider, LLMUsageMetadata,
    ModelsByFeature,
};
use crate::ai::mcp::TemplatableMCPServerManager;
use crate::auth::AuthStateProvider;
use crate::auth::auth_manager::AuthManager;
use crate::cloud_object::model::persistence::CloudModel;
use crate::network::NetworkStatus;
use crate::server::cloud_objects::update_manager::UpdateManager;
use crate::server::server_api::ServerApiProvider;
use crate::server::sync_queue::SyncQueue;
use crate::settings::{AppLanguage, LanguageSettings};
use crate::test_util::settings::{
    initialize_localization_for_tests, initialize_settings_for_tests,
};
use crate::workspaces::team_tester::TeamTesterStatus;
use crate::workspaces::user_workspaces::UserWorkspaces;

#[test]
fn execution_profile_display_name_localizes_only_generated_labels() {
    App::test((), |mut app| async move {
        initialize_localization_for_tests(&mut app);

        let default_profile = AIExecutionProfile {
            name: "Untitled".to_owned(),
            is_default_profile: true,
            ..Default::default()
        };
        let untitled_profile = AIExecutionProfile {
            name: "  ".to_owned(),
            ..Default::default()
        };
        let user_named_default = AIExecutionProfile {
            name: "Default".to_owned(),
            ..Default::default()
        };

        app.read(|ctx| {
            assert_eq!(default_profile.localized_display_name(ctx), "Default");
            assert_eq!(untitled_profile.localized_display_name(ctx), "Untitled");
            assert_eq!(user_named_default.localized_display_name(ctx), "Default");
        });

        app.update(|ctx| {
            LanguageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings
                    .app_language
                    .load_value(AppLanguage::SimplifiedChinese, true, ctx)
                    .expect("language setting should update");
            });
        });

        app.read(|ctx| {
            assert_eq!(default_profile.localized_display_name(ctx), "默认");
            assert_eq!(untitled_profile.localized_display_name(ctx), "未命名");
            assert_eq!(user_named_default.localized_display_name(ctx), "Default");
        });
    });
}

#[test]
fn execution_profile_permission_options_follow_simplified_chinese() {
    App::test((), |mut app| async move {
        initialize_localization_for_tests(&mut app);
        app.update(|ctx| {
            LanguageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings
                    .app_language
                    .load_value(AppLanguage::SimplifiedChinese, true, ctx)
                    .expect("language setting should update");
            });
        });

        app.read(|ctx| {
            let items = ExecutionProfileEditorView::action_permission_dropdown_items(
                |permission| ExecutionProfileEditorViewAction::SetApplyCodeDiffs { permission },
                ctx,
            );
            let labels = items
                .iter()
                .map(|item| item.display_text.as_str())
                .collect::<Vec<_>>();
            assert_eq!(labels, ["由 Agent 决定", "始终允许", "始终询问"]);
        });
    });
}

#[test]
fn upgrade_footer_link_range_uses_character_indices_for_simplified_chinese() {
    let label = crate::localization::text_for_locale(
        LocaleId::ZhCn,
        "settings.execution_profile.editor.upgrade_footer",
    );
    let upgrade_link = crate::localization::text_for_locale(
        LocaleId::ZhCn,
        "settings.execution_profile.editor.upgrade_footer.link",
    );
    let range = upgrade_footer_link_char_range(&label, &upgrade_link).unwrap();
    let selected = label
        .chars()
        .skip(range.start)
        .take(range.end - range.start)
        .collect::<String>();

    assert!(label.len() > label.chars().count());
    assert_eq!(selected, upgrade_link);
    assert_eq!(range.end - range.start, upgrade_link.chars().count());
}

#[test]
fn upgrade_footer_link_range_is_absent_without_a_link_in_the_label() {
    assert_eq!(
        upgrade_footer_link_char_range("免费套餐无法使用 Frontier 模型。", "升级"),
        None
    );
    assert_eq!(
        upgrade_footer_link_char_range("Frontier models are unavailable on free plans.", ""),
        None
    );
}

#[test]
fn generated_or_unchanged_profile_names_are_not_persisted() {
    let default_profile = AIExecutionProfile {
        name: "Untitled".to_owned(),
        is_default_profile: true,
        ..Default::default()
    };
    assert_eq!(updated_profile_name(&default_profile, "默认"), None);

    let custom_profile = AIExecutionProfile {
        name: "My profile".to_owned(),
        ..Default::default()
    };
    assert_eq!(updated_profile_name(&custom_profile, "  "), None);
    assert_eq!(updated_profile_name(&custom_profile, " My profile "), None);
    assert_eq!(
        updated_profile_name(&custom_profile, " Renamed "),
        Some("Renamed".to_owned())
    );
}

fn configurable_model(provider: LLMProvider) -> LLMInfo {
    LLMInfo {
        display_name: "test model".to_string(),
        base_model_name: "test model".to_string(),
        id: "test-model".into(),
        reasoning_level: None,
        usage_metadata: LLMUsageMetadata {
            request_multiplier: 1,
            credit_multiplier: None,
        },
        description: None,
        disable_reason: None,
        vision_supported: false,
        spec: None,
        provider,
        host_configs: HashMap::new(),
        discount_percentage: None,
        context_window: LLMContextWindow {
            is_configurable: true,
            min: 200_000,
            max: 1_000_000,
            default_max: 272_000,
        },
    }
}

fn assert_context_window_limit_for_request(
    model: &LLMInfo,
    selected_limit: Option<u32>,
    gpt_configurable_context_window_enabled: bool,
    expected: Option<u32>,
) {
    let model = model.clone();
    App::test((), move |mut app| async move {
        let _flag = FeatureFlag::GPTConfigurableContextWindow
            .override_enabled(gpt_configurable_context_window_enabled);

        initialize_settings_for_tests(&mut app);
        app.add_singleton_model(|_| ServerApiProvider::new_for_test());
        app.add_singleton_model(|_| AuthStateProvider::new_for_test());
        app.add_singleton_model(AuthManager::new_for_test);
        app.add_singleton_model(|_| NetworkStatus::new());
        app.add_singleton_model(UserWorkspaces::default_mock);
        app.add_singleton_model(CloudModel::mock);
        app.add_singleton_model(TeamTesterStatus::mock);
        app.add_singleton_model(SyncQueue::mock);
        app.add_singleton_model(UpdateManager::mock);
        app.add_singleton_model(|_| TemplatableMCPServerManager::default());
        app.add_singleton_model(|ctx| {
            AIExecutionProfilesModel::new(&LaunchMode::new_for_unit_test(), ctx)
        });
        let llm_preferences = app.add_singleton_model(LLMPreferences::new);

        let profile_model_id = model.id.clone();
        let available_model_id = profile_model_id.clone();
        llm_preferences.update(&mut app, move |preferences, ctx| {
            preferences.update_feature_model_choices(
                Ok(ModelsByFeature {
                    agent_mode: AvailableLLMs::new(available_model_id, [model], None)
                        .expect("test model should create available LLMs"),
                    ..Default::default()
                }),
                ctx,
            );
        });

        let profile = AIExecutionProfile {
            base_model: Some(profile_model_id),
            context_window_limit: selected_limit,
            ..Default::default()
        };
        app.read(|ctx| {
            assert_eq!(profile.context_window_limit_for_request(ctx), expected);
        });
    });
}
/// Helper: round-trip f32 → u32 for readable assertions and absorb the
/// negligible f64→f32 drift the snap helper picks up on large ranges.
fn rounded(values: &[f32]) -> Vec<u32> {
    values.iter().map(|v| v.round() as u32).collect()
}

#[test]
fn snap_values_for_min_eq_max_returns_single_point() {
    assert_eq!(
        rounded(&context_window_snap_values(50_000, 50_000)),
        vec![50_000]
    );
}

#[test]
fn snap_values_for_min_gt_max_collapses_to_min() {
    // Defensive: invalid bounds shouldn't panic, just degrade gracefully.
    assert_eq!(rounded(&context_window_snap_values(100, 50)), vec![100]);
}

#[test]
fn snap_values_always_include_endpoints() {
    let values = rounded(&context_window_snap_values(1_000, 200_000));
    assert_eq!(values.first(), Some(&1_000));
    assert_eq!(values.last(), Some(&200_000));
}

#[test]
fn snap_values_for_classic_200k_range_match_legacy_layout() {
    // Mirrors the old hardcoded list, except `1_000` replaces the missing
    // round multiple at the start.
    let values = rounded(&context_window_snap_values(1_000, 200_000));
    assert_eq!(
        values,
        vec![
            1_000, 25_000, 50_000, 75_000, 100_000, 125_000, 150_000, 175_000, 200_000
        ]
    );
}

#[test]
fn snap_values_for_claude_1m_range_pick_100k_steps() {
    let values = rounded(&context_window_snap_values(200_000, 1_000_000));
    assert_eq!(
        values,
        vec![
            200_000, 300_000, 400_000, 500_000, 600_000, 700_000, 800_000, 900_000, 1_000_000
        ]
    );
}

#[test]
fn snap_values_for_min_zero_skips_duplicate_zero() {
    let values = rounded(&context_window_snap_values(0, 100));
    // First entry is min (0), then nice multiples up to and including max.
    assert_eq!(values.first(), Some(&0));
    assert_eq!(values.last(), Some(&100));
    assert!(values.iter().filter(|&&v| v == 0).count() == 1);
}

#[test]
fn snap_values_for_offset_min_align_to_nice_grid() {
    // min=26_000 doesn't sit on a 25k boundary; first nice value is 50_000.
    let values = rounded(&context_window_snap_values(26_000, 200_000));
    assert_eq!(values.first(), Some(&26_000));
    assert_eq!(values.last(), Some(&200_000));
    // Ensure the second point lands on a nice multiple, not on min+step.
    assert_eq!(values.get(1), Some(&50_000));
}

#[test]
fn snap_values_keep_count_reasonable_for_huge_range() {
    // 1B span should still produce a small (~9) snap-point list, not
    // millions of entries.
    let values = context_window_snap_values(0, 1_000_000_000);
    assert!(
        values.len() <= 12,
        "expected at most 12 snap points, got {}",
        values.len()
    );
    assert!(
        values.len() >= 5,
        "expected at least 5 snap points, got {}",
        values.len()
    );
}

#[test]
fn openai_long_context_warning_starts_above_threshold() {
    let model = configurable_model(LLMProvider::OpenAI);

    assert!(!should_show_long_context_pricing_warning(
        &model,
        Some(200_000),
        true
    ));
    assert!(!should_show_long_context_pricing_warning(
        &model,
        Some(272_000),
        true
    ));
    assert!(should_show_long_context_pricing_warning(
        &model,
        Some(272_001),
        true
    ));
}

#[test]
fn openai_long_context_warning_clamps_stale_override_to_lowered_model_max() {
    let mut model = configurable_model(LLMProvider::OpenAI);
    model.context_window.max = 272_000;

    assert!(!should_show_long_context_pricing_warning(
        &model,
        Some(1_000_000),
        true
    ));
}

#[test]
fn openai_request_limit_is_clamped_when_configurable_context_is_available() {
    let model = configurable_model(LLMProvider::OpenAI);
    assert_context_window_limit_for_request(&model, Some(1_500_000), true, Some(1_000_000));
}

#[test]
fn openai_request_limit_remains_unset_without_a_selected_override() {
    let model = configurable_model(LLMProvider::OpenAI);

    assert_context_window_limit_for_request(&model, None, true, None);
}

#[test]
fn custom_endpoint_fixed_context_does_not_expose_control_or_warning() {
    let mut model = configurable_model(LLMProvider::Unknown);
    model.context_window.is_configurable = false;
    model.context_window.max = 200_000;
    assert!(!has_configurable_context_window(&model, false));
    assert_context_window_limit_for_request(&model, Some(1_000_000), false, None);
    assert!(!should_show_long_context_pricing_warning(
        &model,
        Some(1_000_000),
        false
    ));
}

#[test]
fn openai_configurable_context_uses_server_metadata_without_model_or_host_allowlist() {
    let mut model = configurable_model(LLMProvider::OpenAI);
    model.base_model_name = "new-server-configurable-model".to_string();
    assert!(has_configurable_context_window(&model, true));
    assert_context_window_limit_for_request(&model, Some(1_000_000), true, Some(1_000_000));
    assert!(should_show_long_context_pricing_warning(
        &model,
        Some(1_000_000),
        true
    ));
}

#[test]
fn openai_fixed_context_metadata_does_not_expose_control_or_warning() {
    let mut model = configurable_model(LLMProvider::OpenAI);
    model.context_window = LLMContextWindow {
        is_configurable: false,
        min: 272_000,
        max: 272_000,
        default_max: 272_000,
    };
    assert!(!has_configurable_context_window(&model, true));
    assert_context_window_limit_for_request(&model, Some(1_000_000), true, None);
    assert!(!should_show_long_context_pricing_warning(
        &model,
        Some(1_000_000),
        true
    ));
}

#[test]
fn openai_configurable_context_does_not_require_direct_host_metadata() {
    let model = configurable_model(LLMProvider::OpenAI);

    assert!(has_configurable_context_window(&model, true));
    assert_context_window_limit_for_request(&model, Some(1_000_000), true, Some(1_000_000));
    assert!(should_show_long_context_pricing_warning(
        &model,
        Some(1_000_000),
        true
    ));
}

#[test]
fn openai_expanded_context_is_hidden_while_feature_flag_is_off() {
    let model = configurable_model(LLMProvider::OpenAI);

    assert!(!has_configurable_context_window(&model, false));
    assert_context_window_limit_for_request(&model, Some(1_000_000), false, None);
    assert!(!should_show_long_context_pricing_warning(
        &model,
        Some(1_000_000),
        false
    ));
}

#[test]
fn non_openai_configurable_context_ignores_gpt_flag_and_does_not_show_openai_warning() {
    let model = configurable_model(LLMProvider::Anthropic);

    assert!(has_configurable_context_window(&model, false));
    assert_context_window_limit_for_request(&model, Some(1_000_000), false, Some(1_000_000));
    assert!(!should_show_long_context_pricing_warning(
        &model,
        Some(1_000_000),
        false
    ));
}
