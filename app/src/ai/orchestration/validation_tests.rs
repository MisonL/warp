use ai::agent::action::RunAgentsExecutionMode;
use settings::Setting as _;
use warp_localization::LocaleId;
use warpui::{App, AppContext, Entity, SingletonEntity as _};

use super::{accept_disabled_reason_with_auth, empty_env_recommendation_message};
use crate::ai::cloud_environments::{AmbientAgentEnvironment, CloudAmbientAgentEnvironmentModel};
use crate::ai::orchestration::config_state::{AuthSecretSelection, OrchestrationConfigState};
use crate::cloud_object::model::persistence::CloudModel;
use crate::cloud_object::{CloudObjectMetadata, CloudObjectPermissions};
use crate::server::ids::{ClientId, SyncId};
use crate::settings::{AppLanguage, LanguageSettings};
use crate::test_util::settings::initialize_settings_for_tests;

/// Minimal entity used to borrow an `AppContext` inside `App::test`.
struct CtxProbe;

impl Entity for CtxProbe {
    type Event = ();
}

/// Runs `f` with a plain `AppContext` (no singletons registered).
fn with_app_ctx(f: impl FnOnce(&AppContext) + 'static) {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        let probe = app.add_model(|_| CtxProbe);
        probe.update(&mut app, |_, ctx| f(ctx));
    });
}

fn state(
    harness: &str,
    mode: RunAgentsExecutionMode,
    auth: AuthSecretSelection,
) -> OrchestrationConfigState {
    let mut state =
        OrchestrationConfigState::from_run_agents_fields(Some("auto"), Some(harness), &mode);
    state.auth_secret_selection = auth;
    state
}

fn cloud() -> RunAgentsExecutionMode {
    RunAgentsExecutionMode::Remote {
        environment_id: "env-1".to_string(),
        worker_host: "warp".to_string(),
        computer_use_enabled: false,
        runner_id: String::new(),
    }
}

fn empty_warp_cloud() -> RunAgentsExecutionMode {
    RunAgentsExecutionMode::Remote {
        environment_id: String::new(),
        worker_host: "warp".to_string(),
        computer_use_enabled: false,
        runner_id: String::new(),
    }
}

fn cloud_environment(
    sync_id: SyncId,
) -> crate::ai::cloud_environments::CloudAmbientAgentEnvironment {
    let environment = AmbientAgentEnvironment::new(
        "Test environment".to_string(),
        None,
        Vec::new(),
        "ubuntu:latest".to_string(),
        Vec::new(),
    );
    crate::ai::cloud_environments::CloudAmbientAgentEnvironment::new(
        sync_id,
        CloudAmbientAgentEnvironmentModel::new(environment),
        CloudObjectMetadata::mock(),
        CloudObjectPermissions::mock_personal(),
    )
}

#[test]
fn empty_environment_recommendation_localizes_for_empty_catalog() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        app.add_singleton_model(CloudModel::mock);

        app.update(|ctx| {
            let mode = empty_warp_cloud();
            assert_eq!(
                empty_env_recommendation_message(&mode, ctx),
                Some("We recommend creating an environment for cloud agents.".to_string())
            );
            LanguageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings
                    .app_language
                    .set_value(AppLanguage::SimplifiedChinese, ctx)
                    .expect("language setting should update");
            });
            assert_eq!(
                empty_env_recommendation_message(&mode, ctx),
                Some("建议为云端 Agent 创建一个环境。".to_string())
            );
            assert_eq!(crate::localization::current_locale(ctx), LocaleId::ZhCn);
        });
    });
}

#[test]
fn empty_environment_recommendation_localizes_for_existing_catalog() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        let cloud_model = app.add_singleton_model(CloudModel::mock);
        let sync_id = SyncId::ClientId(ClientId::new());
        app.update(|ctx| {
            cloud_model.update(ctx, |model, ctx| {
                model.create_object(sync_id, cloud_environment(sync_id), ctx);
            });
        });

        app.update(|ctx| {
            let mode = empty_warp_cloud();
            assert_eq!(
                empty_env_recommendation_message(&mode, ctx),
                Some("We recommend selecting an environment for cloud agents.".to_string())
            );
            LanguageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings
                    .app_language
                    .set_value(AppLanguage::SimplifiedChinese, ctx)
                    .expect("language setting should update");
            });
            assert_eq!(
                empty_env_recommendation_message(&mode, ctx),
                Some("建议为云端 Agent 选择一个环境。".to_string())
            );
        });
    });
}

#[test]
fn accept_allowed_for_oz_local_and_cloud() {
    with_app_ctx(|ctx| {
        for mode in [RunAgentsExecutionMode::Local, cloud()] {
            let state = state("oz", mode, AuthSecretSelection::Unset);
            assert_eq!(accept_disabled_reason_with_auth(&state, ctx), None);
        }
    });
}

#[test]
fn accept_blocked_for_product_disabled_local_codex() {
    with_app_ctx(|ctx| {
        let state = state(
            "codex",
            RunAgentsExecutionMode::Local,
            AuthSecretSelection::Unset,
        );
        assert_eq!(
            accept_disabled_reason_with_auth(&state, ctx),
            Some("Local Codex child agents are temporarily disabled.".to_string())
        );
    });
}

#[test]
fn accept_blocked_for_opencode_cloud() {
    with_app_ctx(|ctx| {
        let state = state("opencode", cloud(), AuthSecretSelection::Unset);
        let reason = accept_disabled_reason_with_auth(&state, ctx)
            .expect("OpenCode + Cloud should block Accept");
        assert!(reason.contains("OpenCode"));
    });
}

#[test]
fn accept_blocked_for_cloud_harness_with_unset_auth_secret() {
    with_app_ctx(|ctx| {
        for harness in ["claude", "codex"] {
            for auth in [AuthSecretSelection::Unset, AuthSecretSelection::CreatingNew] {
                let state = state(harness, cloud(), auth);
                assert_eq!(
                    accept_disabled_reason_with_auth(&state, ctx),
                    Some("Select an API key for this harness to continue.".to_string()),
                    "Cloud + {harness} without an API key choice should block Accept"
                );
            }
        }
    });
}

#[test]
fn accept_allowed_for_cloud_harness_with_named_or_inherited_auth() {
    with_app_ctx(|ctx| {
        for harness in ["claude", "codex"] {
            for auth in [
                AuthSecretSelection::Named("my-key".to_string()),
                AuthSecretSelection::Inherit,
            ] {
                let state = state(harness, cloud(), auth);
                assert_eq!(accept_disabled_reason_with_auth(&state, ctx), None);
            }
        }
    });
}
