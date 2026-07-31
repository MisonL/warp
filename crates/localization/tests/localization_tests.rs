use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use serde_yaml::Value as YamlValue;
use settings_value::SettingsValue as _;
use warp_localization::{
    AppLanguage, Catalog, CatalogBundle, CatalogError, LocaleId, TemplateError, TranslationSource,
    replace_placeholders, settings_schema_translation_key,
};

const EN_US: &str = r#"{
  "hello": "Hello",
  "fallback_only": "Fallback"
}"#;

const ZH_CN: &str = r#"{
  "hello": "你好"
}"#;

const BUNDLED_EN_US: &str = include_str!("../../../app/assets/bundled/locales/en-US.json");
const BUNDLED_ZH_CN: &str = include_str!("../../../app/assets/bundled/locales/zh-CN.json");
const DEV_DESKTOP_ENTRY: &str = include_str!("../../../app/channels/dev/dev.warp.WarpDev.desktop");
const LOCAL_DESKTOP_ENTRY: &str =
    include_str!("../../../app/channels/local/dev.warp.WarpLocal.desktop");
const OSS_DESKTOP_ENTRY: &str = include_str!("../../../app/channels/oss/dev.warp.WarpOss.desktop");
const PREVIEW_DESKTOP_ENTRY: &str =
    include_str!("../../../app/channels/preview/dev.warp.WarpPreview.desktop");
const STABLE_DESKTOP_ENTRY: &str =
    include_str!("../../../app/channels/stable/dev.warp.Warp.desktop");
const LOCAL_BIN_SOURCE: &str = include_str!("../../../app/src/bin/local.rs");
const OSS_BIN_SOURCE: &str = include_str!("../../../app/src/bin/oss.rs");
const CLI_INFO_PLIST: &str = include_str!("../../../app/assets/resources/mac/CLI-Info.plist");
const WINDOWS_INSTALLER_SOURCE: &str =
    include_str!("../../../script/windows/windows-installer.iss");
const WINDOWS_INSTALLER_ZH_CN_MESSAGES: &str =
    include_str!("../../../script/windows/languages/ChineseSimplified.isl");
const WINDOWS_INSTALLER_CHECK_SOURCE: &str =
    include_str!("../../../script/windows/check_installer.ps1");
const CI_WORKFLOW_SOURCE: &str = include_str!("../../../.github/workflows/ci.yml");
const LAUNCH_CONFIG_SAVE_MODAL_SOURCE: &str =
    include_str!("../../../app/src/launch_configs/save_modal.rs");
const USER_CONFIG_NATIVE_SOURCE: &str = include_str!("../../../app/src/user_config/native.rs");
const WORKSPACE_VIEW_SOURCE: &str = include_str!("../../../app/src/workspace/view.rs");
const AI_SETTINGS_PAGE_SOURCE: &str = include_str!("../../../app/src/settings_view/ai_page.rs");
const AMBIENT_AGENT_MODEL_SELECTOR_SOURCE: &str =
    include_str!("../../../app/src/terminal/view/ambient_agent/model_selector.rs");
const AGENT_SDK_AMBIENT_SOURCE: &str = include_str!("../../../app/src/ai/agent_sdk/ambient.rs");
const AMBIENT_AGENT_TASK_SOURCE: &str = include_str!("../../../app/src/ai/ambient_agents/task.rs");
const WORKSPACE_CLI_INSTALL_SOURCE: &str =
    include_str!("../../../app/src/workspace/cli_install.rs");
const LOCAL_AGENT_TASK_SYNC_MODEL_SOURCE: &str =
    include_str!("../../../app/src/ai/blocklist/local_agent_task_sync_model.rs");
const DRIVE_SOURCE: &str = include_str!("../../../app/src/drive/mod.rs");
const WARPIFY_BANNER_SOURCE: &str =
    include_str!("../../../app/src/terminal/view/block_banner/warpify.rs");
const MCP_SOURCE: &str = include_str!("../../../app/src/ai/mcp/mod.rs");
const MCP_LIST_PAGE_SOURCE: &str =
    include_str!("../../../app/src/settings_view/mcp_servers/list_page.rs");
const MCP_SERVERS_PAGE_SOURCE: &str =
    include_str!("../../../app/src/settings_view/mcp_servers_page.rs");
const SLASH_COMMANDS_SOURCE: &str =
    include_str!("../../../app/src/terminal/input/slash_commands/mod.rs");
const TERMINAL_AGENT_VIEW_SOURCE: &str =
    include_str!("../../../app/src/terminal/view/agent_view.rs");
const AGENT_VIEW_CONTROLLER_SOURCE: &str =
    include_str!("../../../app/src/ai/blocklist/agent_view/controller.rs");
const INLINE_WEB_SEARCH_SOURCE: &str =
    include_str!("../../../app/src/ai/blocklist/inline_action/web_search.rs");
const AI_BLOCK_SOURCE: &str = include_str!("../../../app/src/ai/blocklist/block.rs");
const REQUESTED_COMMAND_SOURCE: &str =
    include_str!("../../../app/src/ai/blocklist/inline_action/requested_command.rs");
const LLMS_SOURCE: &str = include_str!("../../../app/src/ai/llms.rs");
const MODEL_DATA_SOURCE: &str =
    include_str!("../../../app/src/terminal/input/models/data_source.rs");
const PROFILE_MODEL_SELECTOR_SOURCE: &str =
    include_str!("../../../app/src/terminal/profile_model_selector.rs");
const AGENT_ASSISTED_ENVIRONMENT_SOURCE: &str =
    include_str!("../../../app/src/settings_view/agent_assisted_environment_modal.rs");
const DRIVE_EXPORT_SOURCE: &str = include_str!("../../../app/src/drive/export.rs");
const DRIVE_IMPORT_NODES_SOURCE: &str = include_str!("../../../app/src/drive/import/nodes.rs");
const DRIVE_IMPORT_MODAL_SOURCE: &str = include_str!("../../../app/src/drive/import/modal_body.rs");
const DEFAULT_WORKTREE_TAB_CONFIG: &str =
    include_str!("../../../app/resources/tab_configs/default_worktree.toml");
const DEFAULT_WORKTREE_TAB_CONFIG_ZH_CN: &str =
    include_str!("../../../app/resources/tab_configs/default_worktree.zh-CN.toml");
const NEW_TAB_CONFIG_TEMPLATE: &str =
    include_str!("../../../app/resources/tab_configs/new_tab_config_template.toml");
const NEW_TAB_CONFIG_TEMPLATE_ZH_CN: &str =
    include_str!("../../../app/resources/tab_configs/new_tab_config_template.zh-CN.toml");
const CREATE_SKILL_EVAL_REVIEW_HTML: &str =
    include_str!("../../../resources/bundled/skills/create-skill/assets/eval_review.html");
const CREATE_SKILL_EVAL_VIEWER_HTML: &str =
    include_str!("../../../resources/bundled/skills/create-skill/eval-viewer/viewer.html");
const ALLOWED_EMPTY_TRANSLATION_KEYS: &[&str] = &["auth.empty"];

const UI_LITERAL_PATTERNS: &[&str] = &[
    "with_text_label(",
    "with_centered_text_label(",
    "set_placeholder_text(",
    "ActionButton::new(",
    "MenuItemFields::new(",
    "DropdownItem::new(",
    ".span(",
    ".paragraph(",
    "Text::new(",
    "Text::new_inline(",
    "DismissibleToast::error(",
    "DismissibleToast::success(",
    "DismissibleToast::default(",
    "MessageItem::text(",
    "AccessibilityContent::new(",
    "CompactibleActionButton::new(",
    "tool_tip(",
    "with_tooltip(",
    "FormattedTextFragment::plain_text(",
    "FormattedTextFragment::hyperlink(",
    "FormattedTextFragment::hyperlink_action(",
    "button::Content::Label(",
    "FormattedTextElement::from_str(",
    ".link(",
    ".wrappable_text(",
];

const ONBOARDING_UI_LITERAL_PATTERNS: &[&str] = &[
    "button::Content::Label(",
    "FormattedTextElement::from_str(",
    ".link(",
    ".wrappable_text(",
];

const AI_SETTINGS_HIGH_RISK_UI_PATTERNS: &[&str] = &[
    "ActionButton::new(",
    "DropdownItem::new(",
    "Modal::new(",
    "ToggleSettingActionPair::new(",
    "SettingActionPairDescriptions::new(",
    "build_sub_header(",
    "render_ai_setting_toggle::<",
    "render_ai_setting_description(",
    "render_dropdown_item(",
    "render_full_pane_width_ai_button(",
    ".set_title(",
];

const ALLOWED_DIRECT_UI_LITERALS: &[&str] = &["...", "Warp", "ZDR"];

type CatalogMap = serde_json::Map<String, serde_json::Value>;

fn bundle() -> CatalogBundle {
    CatalogBundle::new(
        LocaleId::EnUs,
        [
            Catalog::from_json(LocaleId::EnUs, EN_US).unwrap(),
            Catalog::from_json(LocaleId::ZhCn, ZH_CN).unwrap(),
        ],
    )
    .unwrap()
}

#[test]
fn looks_up_requested_locale_text() {
    let bundle = bundle();
    let lookup = bundle.lookup(LocaleId::ZhCn, "hello");

    assert_eq!(lookup.text, "你好");
    assert_eq!(lookup.source, TranslationSource::RequestedLocale);
}

#[test]
fn falls_back_to_default_locale_before_key() {
    let bundle = bundle();
    let lookup = bundle.lookup(LocaleId::ZhCn, "fallback_only");

    assert_eq!(lookup.text, "Fallback");
    assert_eq!(lookup.source, TranslationSource::DefaultLocale);

    let missing = bundle.lookup(LocaleId::ZhCn, "missing.key");

    assert_eq!(missing.text, "missing.key");
    assert_eq!(missing.source, TranslationSource::Key);
}

#[test]
fn rejects_empty_catalog_keys() {
    let err = Catalog::from_json(LocaleId::EnUs, r#"{"": "bad"}"#).unwrap_err();

    assert!(matches!(err, CatalogError::EmptyKey { .. }));
}

#[test]
fn resolves_app_language_to_effective_locale() {
    assert_eq!(
        AppLanguage::System.effective_locale(Some("zh_CN.UTF-8")),
        LocaleId::ZhCn
    );
    assert_eq!(
        AppLanguage::System.effective_locale(Some("zh_TW.UTF-8")),
        LocaleId::EnUs
    );
    assert_eq!(AppLanguage::English.effective_locale(None), LocaleId::EnUs);
    assert_eq!(
        AppLanguage::SimplifiedChinese.effective_locale(None),
        LocaleId::ZhCn
    );
}

#[test]
fn resolves_common_bcp_47_and_posix_locale_values() {
    let cases = [
        ("en", Some(LocaleId::EnUs)),
        ("en-GB", Some(LocaleId::EnUs)),
        ("en_US.UTF-8", Some(LocaleId::EnUs)),
        ("zh", Some(LocaleId::ZhCn)),
        ("zh-CN", Some(LocaleId::ZhCn)),
        ("zh-SG", Some(LocaleId::ZhCn)),
        ("zh-Hans", Some(LocaleId::ZhCn)),
        ("zh-Hans-CN", Some(LocaleId::ZhCn)),
        ("zh_Hans_CN", Some(LocaleId::ZhCn)),
        ("zh-Hant-TW", None),
        ("fr-FR", None),
        ("C.UTF-8", None),
        ("POSIX", None),
        ("", None),
    ];

    for (locale, expected) in cases {
        assert_eq!(
            LocaleId::from_system_locale(locale),
            expected,
            "locale {locale}"
        );
    }
}

#[test]
fn resolves_system_language_from_first_supported_candidate() {
    assert_eq!(
        AppLanguage::System.effective_locale_from_candidates(["", "zh-Hant-TW", "zh-Hans-CN"]),
        LocaleId::ZhCn
    );
    assert_eq!(
        AppLanguage::System.effective_locale_from_candidates(["fr-FR", "en-US"]),
        LocaleId::EnUs
    );
    assert_eq!(
        AppLanguage::SimplifiedChinese.effective_locale_from_candidates(["en-US"]),
        LocaleId::ZhCn
    );
}

#[test]
fn app_language_file_value_uses_snake_case() {
    let value = AppLanguage::SimplifiedChinese.to_file_value();

    assert_eq!(value, json!("simplified_chinese"));
    assert_eq!(
        AppLanguage::from_file_value(&value),
        Some(AppLanguage::SimplifiedChinese)
    );
}

#[test]
fn bundled_language_option_labels_match_current_locale() {
    let en_us = Catalog::from_json(LocaleId::EnUs, BUNDLED_EN_US).unwrap();
    let zh_cn = Catalog::from_json(LocaleId::ZhCn, BUNDLED_ZH_CN).unwrap();

    assert_language_option_labels(
        &en_us,
        [
            (AppLanguage::System, "System"),
            (AppLanguage::English, "English"),
            (AppLanguage::SimplifiedChinese, "Simplified Chinese"),
        ],
    );
    assert_language_option_labels(
        &zh_cn,
        [
            (AppLanguage::System, "跟随系统"),
            (AppLanguage::English, "英语"),
            (AppLanguage::SimplifiedChinese, "简体中文"),
        ],
    );
}

#[test]
fn linux_desktop_entries_include_zh_cn_metadata() {
    let entries = [
        ("dev", DEV_DESKTOP_ENTRY),
        ("local", LOCAL_DESKTOP_ENTRY),
        ("oss", OSS_DESKTOP_ENTRY),
        ("preview", PREVIEW_DESKTOP_ENTRY),
        ("stable", STABLE_DESKTOP_ENTRY),
    ];

    for (name, content) in entries {
        assert!(
            content.contains("\nGenericName[zh_CN]="),
            "{name} desktop entry should localize GenericName"
        );
        assert!(
            content.contains("\nComment="),
            "{name} desktop entry should include a default Comment"
        );
        assert!(
            content.contains("\nComment[zh_CN]="),
            "{name} desktop entry should localize Comment"
        );
        assert!(
            content.contains("\nKeywords[zh_CN]="),
            "{name} desktop entry should localize Keywords"
        );
    }
}

#[test]
fn mac_plist_development_regions_use_language_codes() {
    let entries = [
        ("local binary plist", LOCAL_BIN_SOURCE),
        ("oss binary plist", OSS_BIN_SOURCE),
        ("cli helper plist", CLI_INFO_PLIST),
    ];

    for (name, content) in entries {
        assert!(
            content.contains("<key>CFBundleDevelopmentRegion</key>"),
            "{name} should declare CFBundleDevelopmentRegion"
        );
        assert!(
            content.contains("<string>en</string>"),
            "{name} should use the BCP 47 language code for English"
        );
        assert!(
            !content.contains("<string>English</string>"),
            "{name} should not spell out the development region"
        );
    }
}

#[test]
fn windows_installer_localizes_simplified_chinese_surfaces() {
    assert!(
        WINDOWS_INSTALLER_SOURCE.contains(
            "Name: \"chinesesimplified\"; MessagesFile: \"languages\\ChineseSimplified.isl\""
        ),
        "Windows installer should use the bundled Simplified Chinese messages"
    );
    assert!(
        WINDOWS_INSTALLER_ZH_CN_MESSAGES.contains("LanguageName=简体中文")
            && WINDOWS_INSTALLER_ZH_CN_MESSAGES.contains("LanguageID=$0804")
            && WINDOWS_INSTALLER_ZH_CN_MESSAGES.contains("SetupAppTitle=安装"),
        "bundled Windows installer messages should be the Simplified Chinese catalog"
    );

    for (key, english, simplified_chinese) in [
        ("AddToPath", "Add %1 to PATH", "将 %1 添加到 PATH"),
        ("OpenInNewTab", "Open %1 in new tab", "在新标签页中打开 %1"),
        (
            "OpenInNewWindow",
            "Open %1 in new window",
            "在新窗口中打开 %1",
        ),
    ] {
        assert!(
            WINDOWS_INSTALLER_SOURCE.contains(&format!("english.{key}={english}")),
            "Windows installer should define the English {key} custom message"
        );
        assert!(
            WINDOWS_INSTALLER_SOURCE
                .contains(&format!("chinesesimplified.{key}={simplified_chinese}")),
            "Windows installer should define the Simplified Chinese {key} custom message"
        );
    }

    for localized_reference in [
        "Description: \"{cm:AddToPath,{#MyAppName}}\"",
        "ValueData: \"{cm:OpenInNewTab,{#MyAppName}}\"",
        "ValueData: \"{cm:OpenInNewWindow,{#MyAppName}}\"",
    ] {
        assert!(
            WINDOWS_INSTALLER_SOURCE.contains(localized_reference),
            "Windows installer should use localized message reference {localized_reference}"
        );
    }

    for direct_english in [
        "Description: \"Add Warp to PATH\"",
        "ValueData: \"Open {#MyAppName} in new tab\"",
        "ValueData: \"Open {#MyAppName} in new window\"",
    ] {
        assert!(
            !WINDOWS_INSTALLER_SOURCE.contains(direct_english),
            "Windows installer should not keep direct English UI text {direct_english}"
        );
    }
}

#[test]
fn ci_compiles_the_windows_installer_script() {
    assert!(
        WINDOWS_INSTALLER_CHECK_SOURCE.contains("windows-installer.iss")
            && WINDOWS_INSTALLER_CHECK_SOURCE.contains("& $iscc.Source @iscc_args"),
        "Windows installer check should invoke ISCC for the production installer script"
    );
    assert!(
        CI_WORKFLOW_SOURCE.contains("./script/windows/check_installer.ps1"),
        "Windows release compilation CI should run the installer compiler check"
    );
}

#[test]
fn launch_config_save_error_classification_does_not_match_english_text() {
    assert!(
        LAUNCH_CONFIG_SAVE_MODAL_SOURCE.contains("SaveNewLaunchConfigError::FileAlreadyExists"),
        "launch config save modal should classify duplicate files by error type"
    );
    assert!(
        !LAUNCH_CONFIG_SAVE_MODAL_SOURCE.contains("File already exists"),
        "launch config save modal should not branch on English error text"
    );
    assert!(
        !USER_CONFIG_NATIVE_SOURCE.contains("\"File name is empty\""),
        "launch config save errors should not expose English UI text from user_config"
    );
    assert!(
        !USER_CONFIG_NATIVE_SOURCE.contains("\"File already exists\""),
        "launch config save errors should not expose English UI text from user_config"
    );
}

#[test]
fn oz_cli_toast_errors_are_localized_before_rendering() {
    assert!(
        WORKSPACE_VIEW_SOURCE.contains("localized_cli_install_error"),
        "workspace view should localize Oz CLI error details before rendering toast text"
    );
    assert!(
        !WORKSPACE_VIEW_SOURCE.contains("&error.to_string()"),
        "workspace view should not pass raw English error strings into Oz CLI toast text"
    );
    assert!(
        !WORKSPACE_CLI_INSTALL_SOURCE.contains("Installation cancelled by user."),
        "Oz CLI installation cancellation should be classified, not exposed as English UI text"
    );
    assert!(
        !WORKSPACE_CLI_INSTALL_SOURCE.contains("Uninstallation cancelled by user."),
        "Oz CLI uninstallation cancellation should be classified, not exposed as English UI text"
    );
    assert!(
        !WORKSPACE_CLI_INSTALL_SOURCE.contains("Oz command is not currently installed."),
        "Oz CLI not-installed state should be classified, not exposed as English UI text"
    );
}

#[test]
fn replaces_template_placeholders_once() {
    let text = replace_placeholders(
        "Failed to load {path}: {error}",
        &[("path", "/tmp/{error}.json"), ("error", "not found")],
    )
    .unwrap();

    assert_eq!(text, "Failed to load /tmp/{error}.json: not found");
}

#[test]
fn preserves_escaped_braces_in_templates() {
    let text =
        replace_placeholders("Use {{name}} for {value}", &[("value", "the result")]).unwrap();

    assert_eq!(text, "Use {name} for the result");
}

#[test]
fn rejects_missing_template_arguments() {
    let err =
        replace_placeholders("Failed to load {path}: {error}", &[("path", "/tmp")]).unwrap_err();

    assert_eq!(
        err,
        TemplateError::MissingArgument {
            name: "error".to_owned()
        }
    );
}

#[test]
fn rejects_unused_template_arguments() {
    let err = replace_placeholders(
        "Failed to load {path}",
        &[("path", "/tmp"), ("error", "bad")],
    )
    .unwrap_err();

    assert_eq!(
        err,
        TemplateError::UnusedArgument {
            name: "error".to_owned()
        }
    );
}

#[test]
fn rejects_duplicate_template_arguments() {
    let err = replace_placeholders(
        "Failed to load {path}",
        &[("path", "/tmp"), ("path", "/var")],
    )
    .unwrap_err();

    assert_eq!(
        err,
        TemplateError::DuplicateArgument {
            name: "path".to_owned()
        }
    );
}

#[test]
fn bundled_appearance_catalogs_include_theme_and_icon_copy() {
    let required_keys = [
        "agent.zero_state.cloud_title",
        "agent.zero_state.cloud_description",
        "agent.zero_state.cloud_docs_link",
        "agent.zero_state.cloud_docs_prefix",
        "agent.zero_state.local_description",
        "agent.zero_state.local_description_with_location",
        "agent.zero_state.local_title",
        "agent.zero_state.oz_updates",
        "agent.zero_state.shortcut.attach_context",
        "agent.zero_state.shortcut.auto_accept",
        "agent.zero_state.shortcut.go_back_to_terminal",
        "agent.zero_state.shortcut.input_shell_command",
        "agent.zero_state.shortcut.new_agent_conversation",
        "agent.zero_state.shortcut.new_cloud_agent_conversation",
        "agent.zero_state.shortcut.open_code_review",
        "agent.zero_state.shortcut.open_history",
        "agent.zero_state.shortcut.pause_agent",
        "agent.zero_state.shortcut.search_conversations",
        "agent.zero_state.shortcut.slash_commands",
        "agent.zero_state.shortcut.switch_model",
        "agent.zero_state.shortcut.toggle_conversation_list",
        "agent.zero_state.recent_activity",
        "agent.zero_state.view_changelog",
        "ai_document.action.restore",
        "ai_document.action.update_agent",
        "ai_document.menu.attach_to_active_session",
        "ai_document.menu.copy_link",
        "ai_document.menu.copy_plan_id",
        "ai_document.menu.save_as_markdown",
        "ai_document.menu.show_in_warp_drive",
        "ai_document.title.default",
        "ai_document.toast.link_copied",
        "ai_document.toast.plan_id_copied",
        "ai_document.tooltip.save_and_sync_to_warp_drive",
        "ai_document.tooltip.show_version_history",
        "ai_document.tooltip.update_agent",
        "ai.facts.offline",
        "ai.facts.rule_editor.action.delete",
        "ai.facts.rule_editor.action.save",
        "ai.facts.rule_editor.description_placeholder",
        "ai.facts.rule_editor.field.name",
        "ai.facts.rule_editor.field.rule",
        "ai.facts.rule_editor.name_placeholder",
        "ai.facts.rule_editor.title.add",
        "ai.facts.rule_editor.title.edit",
        "ai.facts.rules.action.add",
        "ai.facts.rules.action.initialize_project",
        "ai.facts.rules.action.open_file",
        "ai.facts.rules.description",
        "ai.facts.rules.disabled_banner.link",
        "ai.facts.rules.disabled_banner.prefix",
        "ai.facts.rules.disabled_banner.suffix",
        "ai.facts.rules.scope.global",
        "ai.facts.rules.scope.project_based",
        "ai.facts.rules.search_placeholder",
        "ai.facts.rules.title",
        "ai.facts.rules.untitled",
        "ai.facts.rules.zero_state.global",
        "ai.facts.rules.zero_state.project",
        "auth.logout.cancel",
        "auth.logout.confirm",
        "auth.logout.running_processes_plural",
        "auth.logout.running_processes_singular",
        "auth.logout.shared_sessions_plural",
        "auth.logout.shared_sessions_singular",
        "auth.logout.show_running_processes",
        "auth.logout.title",
        "auth.logout.unsaved_files_plural",
        "auth.logout.unsaved_files_singular",
        "auth.logout.unsynced_objects_plural",
        "auth.logout.unsynced_objects_singular",
        "auth.a11y.description",
        "auth.already_have_account",
        "auth.back",
        "auth.browser.copy_url",
        "auth.browser.hint_middle",
        "auth.browser.hint_page_manually",
        "auth.browser.hint_prefix",
        "auth.browser.hint_suffix",
        "auth.browser.title",
        "auth.browser.title.single_line",
        "auth.continue",
        "auth.disable_ai_features",
        "auth.disable_confirm.description.ai",
        "auth.disable_confirm.description.drive",
        "auth.disable_confirm.title.ai",
        "auth.disable_confirm.title.drive",
        "auth.disable_warp_drive",
        "auth.dismiss",
        "auth.done",
        "auth.empty",
        "auth.enable_ai_features",
        "auth.enable_warp_drive",
        "auth.learn_more",
        "auth.offline.description",
        "auth.offline.paragraph_1",
        "auth.offline.paragraph_2",
        "auth.offline.paragraph_3",
        "auth.offline.title",
        "auth.onboarding.subtitle.ai",
        "auth.onboarding.subtitle.drive",
        "auth.onboarding.title.ai",
        "auth.onboarding.title.drive",
        "auth.privacy.adjust_ai_prefix",
        "auth.privacy.adjust_analytics_prefix",
        "auth.privacy.adjust_prefix",
        "auth.privacy.cloud_storage_disabled_description",
        "auth.privacy.cloud_storage_enabled_description",
        "auth.privacy.help_improve",
        "auth.privacy.help_improve_description",
        "auth.privacy.opt_out_prefix",
        "auth.privacy.send_crash_reports",
        "auth.privacy.send_crash_reports_description",
        "auth.privacy.store_ai_conversations",
        "auth.privacy_settings",
        "auth.require_login.ai",
        "auth.require_login.drive_limit",
        "auth.require_login.share",
        "auth.sign_in",
        "auth.sign_up",
        "auth.sign_up_for_warp",
        "auth.skip_confirm.description_1",
        "auth.skip_confirm.description_2",
        "auth.skip_confirm.link",
        "auth.skip_confirm.title",
        "auth.skip_for_now",
        "auth.skip_prompt",
        "auth.terms.link",
        "auth.terms.prefix",
        "auth.token.browser_placeholder",
        "auth.token.paste_from_browser",
        "auth.token.placeholder",
        "auth.welcome",
        "auth.web_handoff.error",
        "env_vars.title.untitled",
        "agent.child_agent.name",
        "agent.child_agent.untitled",
        "code.action.accept_and_save",
        "code.action.discard_this_version",
        "code.action.overwrite",
        "code.action.reject",
        "code.binding.close_all_tabs",
        "code.binding.close_saved_tabs",
        "code.binding.save_file",
        "code.binding.save_file_as",
        "code.find.scanning",
        "drive.export.exported_named",
        "drive.export.exported_object",
        "drive.export.failed",
        "drive.export.failed_named",
        "drive.export.finished_objects",
        "drive.export.open_in_finder",
        "drive.export.open_in_folder",
        "env_vars.validation.secret_conflict.enterprise",
        "env_vars.validation.secret_conflict.user",
        "drive.sharing_onboarding.body.permissions",
        "drive.sharing_onboarding.body.primary",
        "drive.sharing_onboarding.share_kind.environment_variables",
        "drive.sharing_onboarding.share_kind.folder",
        "drive.sharing_onboarding.share_kind.item",
        "drive.sharing_onboarding.share_kind.notebook",
        "drive.sharing_onboarding.share_kind.rule",
        "drive.sharing_onboarding.share_kind.workflow",
        "drive.sharing_onboarding.share_named",
        "drive.sharing_onboarding.title",
        "drive.space.personal",
        "drive.space.shared_with_me",
        "drive.space.team",
        "editor.image_context.tooltip.attach",
        "editor.image_context.tooltip.conversation_limit",
        "editor.image_context.tooltip.query_limit",
        "editor.image_context.tooltip.unsupported_model",
        "editor.voice.toast.enabled_with_shortcut",
        "editor.voice.toast.microphone_access",
        "editor.voice.tooltip.default",
        "editor.voice.tooltip.hold_key",
        "editor.voice.tooltip.microphone_denied",
        "editor.voice.try_voice_input",
        "cloud_object.semantic.edited",
        "cloud_object.semantic.edited_by",
        "cloud_object.semantic.last_edited_by",
        "cloud_object.semantic.permadeletion.plural",
        "cloud_object.semantic.permadeletion.singular",
        "cloud_object.time.day_many",
        "cloud_object.time.day_one",
        "cloud_object.time.hour_many",
        "cloud_object.time.hour_one",
        "cloud_object.time.just_now",
        "cloud_object.time.minute",
        "cloud_object.time.month_many",
        "cloud_object.time.month_one",
        "cloud_object.time.week_many",
        "cloud_object.time.week_one",
        "cloud_object.time.year_many",
        "cloud_object.time.year_one",
        "cloud_object.toast.deleted_forever",
        "cloud_object.toast.failed_create",
        "cloud_object.toast.failed_delete",
        "cloud_object.toast.failed_empty_trash",
        "cloud_object.toast.failed_leave",
        "cloud_object.toast.failed_move",
        "cloud_object.toast.failed_restore",
        "cloud_object.toast.failed_start_editing",
        "cloud_object.toast.failed_trash",
        "cloud_object.toast.failed_update",
        "cloud_object.toast.failed_update_permissions",
        "cloud_object.toast.left",
        "cloud_object.toast.moved_to",
        "cloud_object.toast.no_objects_to_empty",
        "cloud_object.toast.object_count.plural",
        "cloud_object.toast.object_count.singular",
        "cloud_object.toast.rejection.env_vars",
        "cloud_object.toast.rejection.rule",
        "cloud_object.toast.rejection.workflow",
        "cloud_object.toast.restored",
        "cloud_object.toast.saved_to",
        "cloud_object.toast.trashed",
        "cloud_object.toast.trash_emptied",
        "cloud_object.toast.updated",
        "cloud_object.toast.updated_permissions",
        "code.comment.action.comment",
        "code.comment.action.remove",
        "code.comment.imported_from_github",
        "code.diff_viewer.suggested_fixes_title",
        "code.editor.binding.cursor_at_buffer_end",
        "code.editor.binding.cursor_at_buffer_start",
        "code.editor.binding.cut_all_left",
        "code.editor.binding.cut_all_right",
        "code.editor.binding.cut_word_left",
        "code.editor.binding.cut_word_right",
        "code.editor.binding.delete",
        "code.editor.binding.delete_all_left",
        "code.editor.binding.delete_all_right",
        "code.editor.binding.delete_word_left",
        "code.editor.binding.delete_word_right",
        "code.editor.binding.end",
        "code.editor.binding.exit_vim_insert_mode",
        "code.editor.binding.find",
        "code.editor.binding.go_to_line",
        "code.editor.binding.home",
        "code.editor.binding.move_backward_one_word",
        "code.editor.binding.move_cursor_down",
        "code.editor.binding.move_cursor_left",
        "code.editor.binding.move_cursor_right",
        "code.editor.binding.move_cursor_up",
        "code.editor.binding.move_forward_one_word",
        "code.editor.binding.move_to_line_end",
        "code.editor.binding.move_to_line_start",
        "code.editor.binding.remove_previous_character",
        "code.editor.binding.select_all",
        "code.editor.binding.select_down",
        "code.editor.binding.select_left",
        "code.editor.binding.select_left_by_word",
        "code.editor.binding.select_right",
        "code.editor.binding.select_right_by_word",
        "code.editor.binding.select_to_line_end",
        "code.editor.binding.select_to_line_start",
        "code.editor.binding.select_up",
        "code.editor.binding.toggle_comment",
        "code.find_references.loading",
        "code.find_references.showing_many",
        "code.find_references.showing_one",
        "code.find.action.replace_all",
        "code.find.action.select_all",
        "code.find.a11y.description",
        "code.find.a11y.description_with_matches",
        "code.find.a11y.find_focused_help",
        "code.find.a11y.no_results",
        "code.find.a11y.replace_focused_help",
        "code.find.a11y.replace_help",
        "code.find.a11y.replaced_last_match",
        "code.find.a11y.replaced_match",
        "code.find.a11y.result_count",
        "code.find.a11y.result_help",
        "code.find.placeholder.find",
        "code.find.placeholder.replace",
        "code.find.tooltip.case_sensitive",
        "code.find.tooltip.preserve_case",
        "code.find.tooltip.regex_toggle",
        "code.file_tree.error.disabled",
        "code.file_tree.error.remote",
        "code.file_tree.error.wsl",
        "code.file_tree.menu.attach_as_context",
        "code.file_tree.menu.cd_to_directory",
        "code.file_tree.menu.copy_path",
        "code.file_tree.menu.copy_relative_path",
        "code.file_tree.menu.delete",
        "code.file_tree.menu.new_file",
        "code.file_tree.menu.open_file",
        "code.file_tree.menu.open_in_new_pane",
        "code.file_tree.menu.open_in_new_tab",
        "code.file_tree.menu.rename",
        "code.file_tree.menu.reveal_in_explorer",
        "code.file_tree.menu.reveal_in_file_manager",
        "code.file_tree.menu.reveal_in_finder",
        "code.file_tree.tooltip.remote_file_unavailable",
        "code.footer.codebase.default",
        "code.footer.lsp.enable_server",
        "code.footer.lsp.enable_servers",
        "code.footer.lsp.install_server",
        "code.footer.lsp.install_servers",
        "code.footer.lsp.installing_server",
        "code.footer.lsp.server_error",
        "code.footer.lsp.server_stopped",
        "code.footer.lsp.server_unavailable_codebase",
        "code.footer.lsp.support_not_enabled",
        "code.footer.lsp.support_unavailable",
        "code.footer.lsp.unavailable_file_type",
        "code.footer.menu.manage_servers",
        "code.footer.menu.open_logs",
        "code.footer.menu.remove_server",
        "code.footer.menu.restart_all_servers",
        "code.footer.menu.restart_server",
        "code.footer.menu.start_all_servers",
        "code.footer.menu.start_all_stopped_servers",
        "code.footer.menu.start_server",
        "code.footer.menu.stop_all_servers",
        "code.footer.menu.stop_server",
        "code.footer.tab_config.tooltip.enable_ai",
        "code.footer.tab_config.tooltip.open_agent_input",
        "code.footer.workspace.default",
        "code.footer.workspace.unknown",
        "code.goto_line.error.empty_line",
        "code.goto_line.error.invalid_column",
        "code.goto_line.error.invalid_line",
        "code.goto_line.placeholder",
        "code.goto_line.title",
        "code.gutter.tooltip.add_comment",
        "code.gutter.tooltip.add_diff_hunk_as_context",
        "code.gutter.tooltip.revert_diff_hunk",
        "code.gutter.tooltip.save_changes_to_add_comment",
        "code.gutter.tooltip.save_changes_to_attach",
        "code.gutter.tooltip.save_changes_to_revert",
        "code.gutter.tooltip.show_saved_comment",
        "code.menu.close_saved",
        "code.menu.copy_file_path",
        "code.menu.find_references",
        "code.menu.go_to_definition",
        "code.menu.view_markdown_preview",
        "code.nav.hunk_label",
        "code.nav.next",
        "code.nav.previous",
        "code.saved_changes_notice",
        "code.tab.new_suffix",
        "code.tab.untitled",
        "code.toast.load_failed",
        "code.toast.save_failed",
        "code.toast.save_succeeded",
        "conversation_details.action.view_in_oz",
        "conversation_details.tooltip.view_in_oz",
        "auth.sso.detail",
        "auth.sso.header",
        "auth.sso.link_button",
        "agent.confirmation.exit",
        "agent.confirmation.start_new_conversation",
        "agent.confirmation.stop_and_exit",
        "agent.message_bar.autodetected_shell_command",
        "agent.message_bar.autodetected_shell_command_prefix",
        "agent.message_bar.code_review",
        "agent.message_bar.commands",
        "agent.message_bar.current_pane",
        "agent.message_bar.exit_shell_mode",
        "agent.message_bar.fork_and_continue",
        "agent.message_bar.figma.enable_mcp",
        "agent.message_bar.figma.enabling",
        "agent.message_bar.figma.get_mcp",
        "agent.message_bar.help",
        "agent.message_bar.hide_help",
        "agent.message_bar.hide_plan",
        "agent.message_bar.new_pane",
        "agent.message_bar.new_tab",
        "agent.message_bar.open_conversation",
        "agent.message_bar.override",
        "agent.message_bar.resume_conversation",
        "agent.message_bar.view_plan",
        "agent.message_bar.view_plans",
        "agent.block.action.send_feedback",
        "agent.block.toast.copied_to_clipboard",
        "agent.block.toast.feedback_thanks",
        "agent.orchestration.back_to_parent_conversation",
        "agent.orchestration.menu.delete_agent",
        "agent.orchestration.menu.focus_pane",
        "agent.orchestration.menu.kill_agent",
        "agent.orchestration.menu.open_in_new_pane",
        "agent.orchestration.menu.open_in_new_tab",
        "agent.orchestration.menu.stop_agent",
        "agent.orchestration.menu.view_in_oz",
        "agent.orchestration.parent_conversation",
        "agent.orchestration.sending_message_to",
        "agent.orchestration.started_agent",
        "agent_sdk.api_key.error.create_failed",
        "agent_sdk.api_key.error.expire_failed",
        "agent_sdk.api_key.confirm.expire",
        "agent_sdk.api_key.confirm.expire_cancelled",
        "agent_sdk.api_key.confirm.expire_help",
        "agent_sdk.api_key.error.expire_non_interactive_requires_force",
        "agent_sdk.api_key.error.multiple_matches_specify_uid",
        "agent_sdk.api_key.error.not_found",
        "agent_sdk.api_key.output.created",
        "agent_sdk.api_key.output.expired",
        "agent_sdk.api_key.output.multiple_matches",
        "agent_sdk.api_key.output.not_expired",
        "agent_sdk.api_key.output.raw_api_key",
        "agent_sdk.api_key.output.secret_shown_once",
        "agent_sdk.api_key.output.uid",
        "agent_sdk.api_key.prompt.select_key_to_expire",
        "agent.output.current_directory",
        "agent.output.conversation_search.grepping",
        "agent.output.conversation_search.grepping_with_patterns",
        "agent.output.conversation_search.listing_messages",
        "agent.output.conversation_search.reading_messages",
        "agent.output.file_glob.cancelled_patterns",
        "agent.output.file_glob.queued_patterns",
        "agent.output.file_glob.queued_prefix",
        "agent.output.file_glob.running_patterns",
        "agent.output.file_glob.running_prefix",
        "agent.output.grep.cancelled_patterns",
        "agent.output.grep.queued_patterns",
        "agent.output.grep.queued_prefix",
        "agent.output.grep.running_patterns",
        "agent.output.grep.running_prefix",
        "agent.output.in_path",
        "agent.output.in_path_cancelled",
        "agent_management.action.clear_all",
        "agent_management.action.clear_filters",
        "agent_management.action.get_started",
        "agent_management.action.new_agent",
        "agent_management.action.view_agents",
        "agent_management.cloud_setup.action.visit_oz",
        "agent_management.cloud_setup.docs.link",
        "agent_management.cloud_setup.docs.prefix",
        "agent_management.cloud_setup.docs.suffix",
        "agent_management.cloud_setup.header.subtitle",
        "agent_management.cloud_setup.header.title",
        "agent_management.cloud_setup.manual_header",
        "agent_management.cloud_setup.quick_start",
        "agent_management.cloud_setup.step.docs_link",
        "agent_management.cloud_setup.step1.description",
        "agent_management.cloud_setup.step1.docs_prefix",
        "agent_management.cloud_setup.step1.or_existing",
        "agent_management.cloud_setup.step1.title",
        "agent_management.cloud_setup.step2.docs_prefix",
        "agent_management.cloud_setup.step2.title",
        "agent_management.artifact.file.fallback",
        "agent_management.artifact.file_download.failed",
        "agent_management.artifact.file_download.filename_fallback",
        "agent_management.artifact.file_download.prepare_failed",
        "agent_management.artifact.file_download.reveal.explorer",
        "agent_management.artifact.file_download.reveal.finder",
        "agent_management.artifact.file_download.reveal.folder",
        "agent_management.artifact.file_download.success",
        "agent_management.artifact.file_download.success_with_directory",
        "agent_management.artifact.screenshot.failed_to_load",
        "agent_management.filter.artifact.file",
        "agent_management.filter.artifact.plan",
        "agent_management.filter.artifact.pull_request",
        "agent_management.filter.artifact.screenshot",
        "agent_management.filter.created_on",
        "agent_management.filter.created_on.last_24_hours",
        "agent_management.filter.created_on.last_week",
        "agent_management.filter.created_on.past_3_days",
        "agent_management.filter.harness",
        "agent_management.filter.has_artifact",
        "agent_management.filter.created_by",
        "agent_management.filter.environment",
        "agent_management.filter.option.all",
        "agent_management.filter.option.none",
        "agent_management.filter.owner.all",
        "agent_management.filter.owner.all_tooltip",
        "agent_management.filter.owner.personal",
        "agent_management.filter.owner.personal_tooltip",
        "agent_management.filter.source",
        "agent_management.filter.status",
        "agent_management.filter.status.done",
        "agent_management.filter.status.failed",
        "agent_management.filter.status.working",
        "agent_management.loading.agents",
        "agent_management.loading.tooltip",
        "agent_sdk.secret.confirm.delete",
        "agent_sdk.secret.confirm.delete_cancelled",
        "agent_sdk.secret.confirm.delete_help",
        "agent_sdk.secret.error.bedrock_access_key_non_interactive_required",
        "agent_sdk.secret.error.bedrock_access_key_update_value",
        "agent_sdk.secret.error.bedrock_api_key_update_value",
        "agent_sdk.secret.error.bedrock_non_interactive_required",
        "agent_sdk.secret.error.delete_non_interactive_requires_force",
        "agent_sdk.secret.error.not_found",
        "agent_sdk.secret.error.read_value_file_failed",
        "agent_sdk.secret.output.created",
        "agent_sdk.secret.output.deleted",
        "agent_sdk.secret.output.updated",
        "agent_sdk.secret.prompt.aws_access_key_id",
        "agent_sdk.secret.prompt.aws_region",
        "agent_sdk.secret.prompt.aws_secret_access_key",
        "agent_sdk.secret.prompt.aws_session_token_optional",
        "agent_sdk.secret.prompt.bedrock_api_key",
        "agent_sdk.secret.prompt.openai_base_url",
        "agent_sdk.secret.prompt.openai_base_url_help",
        "agent_sdk.secret.prompt.secret_value",
        "agent_sdk.secret.scope.personal",
        "agent_sdk.secret.scope.team",
        "agent_management.metadata.credits_used",
        "agent_management.metadata.harness",
        "agent_management.metadata.run_time",
        "agent_management.metadata.source",
        "agent_management.no_results",
        "agent_management.notifications.action.close",
        "agent_management.notifications.action.mark_all_read",
        "agent_management.notifications.empty",
        "agent_management.notifications.filter.all_tabs",
        "agent_management.notifications.filter.errors",
        "agent_management.notifications.filter.unread",
        "agent_management.notifications.open_conversation",
        "agent_management.notifications.title",
        "agent_management.search.placeholder",
        "agent_management.session_status.expired",
        "agent_management.session_status.expired_tooltip",
        "agent_management.session_status.unavailable",
        "agent_management.title.runs",
        "agent_management.toast.copied_branch_name",
        "agent_management.value.unknown",
        "agent.pending_user_query.action.remove_queued_prompt",
        "agent.pending_user_query.action.send_now",
        "agent.pending_user_query.badge.queued",
        "agent.requested_script.expand",
        "agent.requested_script.hide",
        "agent.requested_script.running",
        "agent.summarization_cancel.action.cancel",
        "agent.summarization_cancel.action.continue",
        "agent.summarization_cancel.description",
        "agent.summarization_cancel.title",
        "terminal.input.agent_hint.ab_testing",
        "terminal.input.agent_hint.backup_postgres",
        "terminal.input.agent_hint.bigquery_pipeline",
        "terminal.input.agent_hint.build_fastapi",
        "terminal.input.agent_hint.configure_https",
        "terminal.input.agent_hint.create_auth_tests",
        "terminal.input.agent_hint.deploy_react_vercel",
        "terminal.input.agent_hint.elk_logs",
        "terminal.input.agent_hint.fix_node_memory_leak",
        "terminal.input.agent_hint.github_actions_deploy",
        "terminal.input.agent_hint.help_debug_python_tests",
        "terminal.input.agent_hint.migrate_mysql_postgres",
        "terminal.input.agent_hint.monitor_aws",
        "terminal.input.agent_hint.oauth_express",
        "terminal.input.agent_hint.optimize_docker",
        "terminal.input.agent_hint.optimize_sql",
        "terminal.input.agent_hint.redis_caching",
        "terminal.input.agent_hint.refactor_legacy",
        "terminal.input.agent_hint.setup_microservice_docker",
        "terminal.input.agent_hint.troubleshoot_kubernetes",
        "terminal.input.binding.new_agent_conversation",
        "terminal.input.cloud_handoff.prepare_failed",
        "terminal.input.conversation_export.error.directory_not_found",
        "terminal.input.conversation_export.error.failed",
        "terminal.input.conversation_export.error.file_exists",
        "terminal.input.conversation_export.error.permission_denied",
        "terminal.input.conversation_export.no_active_conversation",
        "terminal.input.conversation_export.overwrite_warning",
        "terminal.input.conversation_export.success",
        "terminal.input.image_limit.per_conversation",
        "terminal.input.image_limit.per_query",
        "terminal.input.toast.attachment_skipped.plural",
        "terminal.input.toast.attachment_skipped.singular",
        "terminal.input.toast.attached_images_removed",
        "terminal.input.toast.cannot_start_conversation_agent_monitoring",
        "terminal.input.toast.command_already_running",
        "terminal.input.toast.could_not_navigate_to_conversation",
        "terminal.input.toast.image_limit.plural",
        "terminal.input.toast.image_limit.singular",
        "terminal.input.toast.images_removed.plural",
        "terminal.input.toast.images_removed.singular",
        "terminal.input.toast.no_agent_harnesses",
        "terminal.input.toast.preparing_handoff",
        "terminal.input.toast.read_only_viewer",
        "terminal.input.toast.skill_not_found",
        "terminal.input.toast.too_many_attachments",
        "input_suggestions.a11y.closed",
        "input_suggestions.a11y.command_suggestions",
        "input_suggestions.a11y.help",
        "input_suggestions.a11y.last_ran",
        "input_suggestions.a11y.selected",
        "input_suggestions.a11y.suggestion",
        "input_suggestions.no_suggestions",
        "input_suggestions.tooltip.ignore",
        "launch_config.save_modal.a11y.description",
        "launch_config.save_modal.a11y.title",
        "launch_config.save_modal.action.open_file",
        "launch_config.save_modal.action.save",
        "launch_config.save_modal.description",
        "launch_config.save_modal.description_with_keybinding",
        "launch_config.save_modal.documentation_link",
        "launch_config.save_modal.error.file_already_exists",
        "launch_config.save_modal.error.other",
        "launch_config.save_modal.path_prefix",
        "launch_config.save_modal.sentence_suffix",
        "launch_config.save_modal.success_prefix",
        "launch_config.save_modal.title",
        "terminal.ambient_agent.cancelled.subtitle",
        "terminal.ambient_agent.cancelled.title",
        "terminal.ambient_agent.error.title",
        "terminal.ambient_agent.github_auth.action",
        "terminal.ambient_agent.github_auth.message",
        "terminal.ambient_agent.github_auth.title",
        "terminal.ambient_agent.learn_more",
        "terminal.ambient_agent.loading.connecting",
        "terminal.ambient_agent.loading.creating",
        "terminal.ambient_agent.loading.starting",
        "terminal.ambient_agent.tier.current_machine",
        "terminal.ambient_agent.tier.upgrade",
        "terminal.ambient_agent.tier.upgrade_suffix",
        "terminal.auth_secret.delete.action.cancel",
        "terminal.auth_secret.delete.action.delete",
        "terminal.auth_secret.delete.description",
        "terminal.auth_secret.delete.title",
        "terminal.universal_developer_input.context.disabled_terminal_mode",
        "terminal.universal_developer_input.context.no_objects",
        "terminal.universal_developer_input.context.ssh_session",
        "terminal.universal_developer_input.context.subshell",
        "terminal.universal_developer_input.context.wasm",
        "terminal.universal_developer_input.input_mode.agent_monitoring",
        "terminal.universal_developer_input.input_mode.request_edit_access",
        "terminal.universal_developer_input.mode.agent",
        "terminal.universal_developer_input.mode.agent_short",
        "terminal.universal_developer_input.mode.auto",
        "terminal.universal_developer_input.mode.auto_detection",
        "terminal.universal_developer_input.mode.shell",
        "terminal.universal_developer_input.mode.terminal",
        "terminal.universal_developer_input.tooltip.attach_context",
        "terminal.universal_developer_input.tooltip.attach_file",
        "terminal.universal_developer_input.tooltip.slash_commands",
        "terminal.universal_developer_input.tooltip.voice_input",
        "terminal.cloud_mode_v2_history.no_results",
        "terminal.inline_conversation.tab.all",
        "terminal.inline_conversation.tab.current_directory",
        "terminal.inline_history.a11y.ai_prompt",
        "terminal.inline_history.a11y.command",
        "terminal.inline_history.a11y.conversation",
        "terminal.inline_history.configure",
        "terminal.inline_history.header",
        "terminal.inline_history.tab.all",
        "terminal.inline_history.tab.commands",
        "terminal.inline_history.tab.prompts",
        "terminal.shell_terminated.action.copy_error",
        "terminal.shell_terminated.action.file_issue",
        "terminal.shell_terminated.action.more_info",
        "terminal.shell_terminated.normal.title",
        "terminal.shell_terminated.premature.description",
        "terminal.shell_terminated.premature.title",
        "terminal.shell_terminated.pty_spawn.title",
        "terminal.ssh_error.action.continue_without_warpification",
        "terminal.ssh_error.action.warpify_without_tmux",
        "terminal.ssh_error.header",
        "terminal.ssh_error.report_issue.link",
        "terminal.ssh_error.report_issue.prefix",
        "terminal.ssh_error.report_issue.suffix",
        "terminal.ssh_error.title.ssh_warpify_timeout",
        "terminal.ssh_error.title.tmux_failed",
        "terminal.ssh_error.title.tmux_install_failed",
        "terminal.ssh_error.title.tmux_install_timeout",
        "terminal.ssh_error.title.tmux_not_installed",
        "terminal.ssh_error.title.unsupported_shell",
        "terminal.ssh_error.title.unsupported_tmux_version",
        "terminal.ssh_error.tmux_failed",
        "terminal.ssh_error.tmux_install_failed",
        "terminal.ssh_error.tmux_not_installed",
        "terminal.ssh_error.unsupported_shell",
        "terminal.ssh_error.unsupported_tmux_version",
        "terminal.ssh_error.warpify_timeout",
        "terminal.ssh_file_upload.clear_upload",
        "terminal.ssh_file_upload.close_session",
        "terminal.ssh_file_upload.destination",
        "terminal.ssh_file_upload.header",
        "terminal.ssh_file_upload.status.failed",
        "terminal.ssh_file_upload.status.uploaded",
        "terminal.ssh_file_upload.status.uploading",
        "terminal.ssh_file_upload.view_session",
        "terminal.ssh_file_upload.waiting_for_password",
        "terminal.queued_prompts.tooltip.delete",
        "terminal.queued_prompts.tooltip.edit",
        "terminal.queued_prompts.tooltip.initial_cloud_mode_prompt",
        "terminal.queued_prompts.tooltip.send_now",
        "terminal.queued_prompts.tooltip.send_now_cloud_setup",
        "terminal.queued_prompts.tooltip.send_now_full_terminal_use_agent",
        "terminal.rewind.a11y.current",
        "terminal.rewind.a11y.rewind_to_no_changes",
        "terminal.rewind.a11y.rewind_to_with_changes",
        "terminal.rewind.current",
        "terminal.rewind.no_code_to_restore",
        "terminal.status.loading_session",
        "terminal.status.starting_shell",
        "terminal.init_project.action.enable_language_support",
        "terminal.init_project.action.create_environment",
        "terminal.init_project.action.generate_agents_md",
        "terminal.init_project.action.index_codebase",
        "terminal.init_project.action.install_and_enable",
        "terminal.init_project.action.regenerate_agents_md",
        "terminal.init_project.action.skip_agents_md_generation",
        "terminal.init_project.action.skip_for_now",
        "terminal.init_project.action.skip_for_now_period",
        "terminal.init_project.action.view_index_status",
        "terminal.init_project.codebase_context.cancelled",
        "terminal.init_project.codebase_context.prompt",
        "terminal.init_project.codebase_context.started",
        "terminal.init_project.environment.created",
        "terminal.init_project.environment.creating",
        "terminal.init_project.environment.prompt",
        "terminal.init_project.environment.skipped",
        "terminal.init_project.lsp.enabled",
        "terminal.init_project.lsp.enabled_one",
        "terminal.init_project.lsp.install_failed",
        "terminal.init_project.lsp.install_success",
        "terminal.init_project.lsp.installing_background",
        "terminal.init_project.lsp.installation_started",
        "terminal.lsp.start_failed",
        "terminal.init_project.lsp.multiple_prompt",
        "terminal.init_project.lsp.single_prompt",
        "terminal.init_project.lsp.skipped",
        "terminal.init_project.project_rules.already_configured",
        "terminal.init_project.project_rules.configured",
        "terminal.init_project.project_rules.generating",
        "terminal.init_project.project_rules.linked_from",
        "terminal.init_project.project_rules.prompt",
        "terminal.init_project.project_rules.skipped",
        "terminal.init_project.welcome.already_setup",
        "terminal.init_project.welcome.onboarding",
        "terminal.input.conversations.a11y.label",
        "terminal.input.plans.a11y.label",
        "terminal.input.prompts.a11y.label",
        "terminal.input.repos.a11y.indexed_repository",
        "terminal.input.skills.a11y.label",
        "terminal.input.user_query.a11y.label",
        "terminal.prompt_suggestion.execute_this_plan",
        "terminal.prompt_suggestion.zero_state.code",
        "terminal.prompt_suggestion.zero_state.deploy",
        "terminal.prompt_suggestion.zero_state.explain",
        "terminal.prompt_suggestion.zero_state.fix",
        "terminal.prompt_suggestion.zero_state.install",
        "terminal.prompt_suggestion.zero_state.something_else",
        "terminal.prompt_suggestions.tooltip.out_of_credits",
        "terminal.prompt_suggestions.tooltip.restricted_payment_issue",
        "terminal.share_block_modal.action.copy",
        "terminal.share_block_modal.action.create_link",
        "terminal.share_block_modal.action.get_embed",
        "terminal.share_block_modal.action.manage_shared_blocks",
        "terminal.share_block_modal.display.command",
        "terminal.share_block_modal.display.command_and_output",
        "terminal.share_block_modal.display.output",
        "terminal.share_block_modal.embed.default_title",
        "terminal.share_block_modal.embed.error",
        "terminal.share_block_modal.option.redact_secrets",
        "terminal.share_block_modal.option.show_prompt",
        "terminal.share_block_modal.placeholder.title",
        "terminal.share_block_modal.status.creating_block",
        "terminal.share_block_modal.title",
        "terminal.share_block_modal.toast.creation_failed",
        "terminal.share_block_modal.toast.embed_copied",
        "terminal.share_block_modal.toast.link_copied",
        "terminal.shared_session.action.continue",
        "terminal.shared_session.action.continue_locally",
        "terminal.shared_session.action.open_in_warp",
        "terminal.shared_session.action.request_edit_access",
        "terminal.shared_session.menu.copy_link",
        "terminal.shared_session.menu.share_session",
        "terminal.shared_session.menu.stop_sharing",
        "terminal.shared_session.role.edit",
        "terminal.shared_session.role.view",
        "terminal.shared_session.tooltip.continue_cloud",
        "terminal.shared_session.tooltip.continue_locally",
        "terminal.shared_session.tooltip.open_in_warp",
        "terminal.shared_session.toast.continue_cloud_failed",
        "terminal.shared_session.toast.edit_permissions_revoked_sharer_idle",
        "terminal.shared_session.toast.link_copied",
        "terminal.shared_session.toast.shared_edit_permissions_revoked_inactivity",
        "terminal.shared_session.toast.sharing_ended_inactivity",
        "terminal.use_agent_footer.action.dismiss",
        "terminal.use_agent_footer.action.dont_show_again",
        "terminal.use_agent_footer.action.give_control_back",
        "terminal.use_agent_footer.action.use_agent",
        "terminal.use_agent_footer.action.warpify_ssh_session",
        "terminal.use_agent_footer.action.warpify_subshell",
        "terminal.use_agent_footer.tooltip.ask_agent_assist",
        "terminal.use_agent_footer.tooltip.ask_agent_resume",
        "terminal.use_agent_footer.tooltip.enable_warp_shell_integration",
        "terminal.zero_state.autodetect_agent_prompts",
        "terminal.zero_state.dismiss",
        "terminal.zero_state.title",
        "terminal.menu.ai_command_search",
        "terminal.menu.ask_warp_ai",
        "terminal.menu.close_pane",
        "terminal.menu.command_search",
        "terminal.menu.copy",
        "terminal.menu.copy_command",
        "terminal.menu.copy_commands",
        "terminal.menu.copy_conversation_id",
        "terminal.menu.copy_conversation_text",
        "terminal.menu.copy_debugging_id",
        "terminal.menu.copy_debugging_link",
        "terminal.menu.copy_filtered_output",
        "terminal.menu.copy_git_branch",
        "terminal.menu.copy_output",
        "terminal.menu.copy_output_as_markdown",
        "terminal.menu.copy_path",
        "terminal.menu.copy_prompt",
        "terminal.menu.copy_right_prompt",
        "terminal.menu.copy_url",
        "terminal.menu.copy_working_directory",
        "terminal.menu.cut",
        "terminal.menu.edit_agent_toolbelt",
        "terminal.menu.edit_cli_agent_toolbelt",
        "terminal.menu.edit_prompt",
        "terminal.menu.find_within_block",
        "terminal.menu.find_within_blocks",
        "terminal.menu.fork_from_here_dev_only",
        "terminal.menu.fork_from_last_query",
        "terminal.menu.fork_from_query_prefix",
        "terminal.menu.hide_input_hint_text",
        "terminal.menu.insert_into_input",
        "terminal.menu.maximize_pane",
        "terminal.menu.minimize_pane",
        "terminal.menu.open_in_editor",
        "terminal.menu.open_in_warp",
        "terminal.menu.paste",
        "terminal.menu.rewind_to_before_here",
        "terminal.menu.save_as_prompt",
        "terminal.menu.save_as_workflow",
        "terminal.menu.scroll_to_bottom_of_block",
        "terminal.menu.scroll_to_bottom_of_blocks",
        "terminal.menu.scroll_to_top_of_block",
        "terminal.menu.scroll_to_top_of_blocks",
        "terminal.menu.select_all",
        "terminal.menu.share",
        "terminal.menu.share_block",
        "terminal.menu.share_conversation",
        "terminal.menu.show_containing_folder",
        "terminal.menu.show_in_finder",
        "terminal.menu.show_input_hint_text",
        "terminal.menu.split_pane_down",
        "terminal.menu.split_pane_left",
        "terminal.menu.split_pane_right",
        "terminal.menu.split_pane_up",
        "terminal.menu.stop_sharing_session",
        "terminal.menu.toggle_block_filter",
        "terminal.menu.toggle_bookmark",
        "wasm_nux.action.download",
        "wasm_nux.action.learn_more",
        "wasm_nux.action.open_in_warp",
        "wasm_nux.action.yes",
        "wasm_nux.download.description",
        "wasm_nux.download.title",
        "wasm_nux.object_kind.drive_objects",
        "wasm_nux.object_kind.shared_sessions",
        "wasm_nux.object_kind.warp_links",
        "wasm_nux.open_desktop.description",
        "wasm_nux.open_desktop.title",
        "wasm_nux.web_preference.description",
        "wasm_nux.web_preference.title",
        "terminal.agent_title.new_agent_conversation",
        "terminal.agent_title.new_cloud_agent",
        "terminal.ambient_agent.header.running",
        "theme_chooser.a11y.description",
        "theme_chooser.a11y.help",
        "theme_chooser.hint.current",
        "theme_chooser.hint.dark",
        "theme_chooser.hint.light",
        "theme_chooser.no_matching_themes",
        "terminal.pane_header.hide_details",
        "terminal.pane_header.show_details",
        "editor.autosuggestion.keybinding.custom",
        "search.a11y.error_finding_results",
        "search.a11y.item_with_binding",
        "search.a11y.loading_suggestions",
        "search.a11y.selected_item",
        "search.command_search.out_of_credits_contact_admin",
        "search.command_search.out_of_credits_prefix",
        "search.command_search.out_of_credits_suffix",
        "search.command_search.a11y.result_accepted",
        "search.command_search.a11y.result_accepted_help",
        "search.command_search.a11y.result_executed",
        "search.command_search.a11y.result_executed_help",
        "search.command_search.warp_ai.error.bad_prompt",
        "search.command_search.warp_ai.error.generic",
        "search.command_search.warp_ai.error.rate_limited",
        "search.command_search.warp_ai.open_body",
        "search.command_search.warp_ai.translate_body",
        "search.ai_context_menu.code.error.generic",
        "search.command_search.upgrade",
        "search.command_search.upgrade_ai_usage",
        "search.filter.display.tabs",
        "search.filter.placeholder.tabs",
        "search.loading",
        "code_review.action.cancel",
        "code_review.action.close_panel",
        "code_review.action.discard_all",
        "code_review.action.discard_changes",
        "code_review.action.hide_file_navigation",
        "code_review.action.initialize_codebase",
        "code_review.action.maximize",
        "code_review.action.minimize",
        "code_review.action.open_repository",
        "code_review.action.restore",
        "code_review.action.retry",
        "code_review.action.show_file_navigation",
        "code_review.action.undo",
        "code_review.comments.ai_credits_required",
        "code_review.comments.ai_must_be_enabled",
        "code_review.comments.all_terminals_busy",
        "code_review.comments.copy_text",
        "code_review.comments.edit",
        "code_review.comments.file_level_edit_disabled",
        "code_review.comments.no_non_outdated_to_send",
        "code_review.comments.one_comment",
        "code_review.comments.outdated_count",
        "code_review.comments.outdated_edit_disabled",
        "code_review.comments.outdated_many_omitted",
        "code_review.comments.outdated_one_omitted",
        "code_review.comments.remove",
        "code_review.comments.send_to_agent",
        "code_review.comments.send_to_agent_button",
        "code_review.comments.send_to_cli_agent",
        "code_review.comments.view_in_github",
        "code_review.discard.description.all_changes",
        "code_review.discard.description.all_uncommitted",
        "code_review.discard.description.file_changes_branch",
        "code_review.discard.description.file_changes_main",
        "code_review.discard.description.file_uncommitted",
        "code_review.discard.disabled.git_operation",
        "code_review.discard.disabled.no_changes",
        "code_review.discard.no_file_selected",
        "code_review.discard.no_files_to_discard",
        "code_review.discard.stash_changes",
        "code_review.discard.title.all_changes",
        "code_review.discard.title.all_uncommitted",
        "code_review.discard.title.file_changes",
        "code_review.discard.title.file_uncommitted",
        "code_review.diff_content.binary_unavailable",
        "code_review.diff_content.new_empty_file",
        "code_review.diff_content.renamed_without_changes",
        "code_review.diff_content.too_large",
        "code_review.diff_content.unable_to_load",
        "code_review.diff_target.uncommitted_changes",
        "code_review.git.commit",
        "code_review.git.create_pr",
        "code_review.git.no_actions_available",
        "code_review.git.no_changes",
        "code_review.git.publish",
        "code_review.git.push",
        "code_review.git_dialog.branch",
        "code_review.git_dialog.changes",
        "code_review.git_dialog.commit.commit_and_create_pr",
        "code_review.git_dialog.commit.commit_and_publish",
        "code_review.git_dialog.commit.commit_and_push",
        "code_review.git_dialog.commit.committed",
        "code_review.git_dialog.commit.committed_and_pushed",
        "code_review.git_dialog.commit.committing",
        "code_review.git_dialog.commit.enter_message",
        "code_review.git_dialog.commit.generating_message",
        "code_review.git_dialog.commit.include_unstaged",
        "code_review.git_dialog.commit.message",
        "code_review.git_dialog.commit.type_message",
        "code_review.git_dialog.confirm",
        "code_review.git_dialog.error.authentication_failed",
        "code_review.git_dialog.error.generic",
        "code_review.git_dialog.error.gh_not_authenticated",
        "code_review.git_dialog.error.gh_not_installed",
        "code_review.git_dialog.error.git_identity_missing",
        "code_review.git_dialog.error.network",
        "code_review.git_dialog.error.no_changes_to_commit",
        "code_review.git_dialog.error.no_remote_configured",
        "code_review.git_dialog.error.remote_has_new_changes",
        "code_review.git_dialog.error.remote_not_found",
        "code_review.git_dialog.file_plural",
        "code_review.git_dialog.file_singular",
        "code_review.git_dialog.loading",
        "code_review.git_dialog.pr.created",
        "code_review.git_dialog.pr.creating",
        "code_review.git_dialog.pr.default_branch",
        "code_review.git_dialog.pr.open_pr",
        "code_review.git_dialog.push.included_commits",
        "code_review.git_dialog.push.published",
        "code_review.git_dialog.push.publishing",
        "code_review.git_dialog.push.pushed",
        "code_review.git_dialog.push.pushing",
        "code_review.git_dialog.title.commit",
        "code_review.git_dialog.title.create_pr",
        "code_review.git_dialog.title.publish",
        "code_review.git_dialog.title.push",
        "code_review.header.code_review",
        "code_review.header.reviewing_code_changes",
        "code_review.header.reviewing_open_changes",
        "code_review.menu.add_comment",
        "code_review.menu.add_diff_set_context",
        "code_review.menu.show_saved_comment",
        "code_review.repo.unknown",
        "code_review.state.cannot_detect_diffs",
        "code_review.state.error_loading_diffs",
        "code_review.state.loading_open_changes",
        "code_review.state.no_open_changes",
        "code_review.state.no_open_changes_description",
        "code_review.state.not_git_repo",
        "code_review.state.remote",
        "code_review.state.repo_initialized_with_file",
        "code_review.state.wsl",
        "code_review.toast.cannot_attach_input_unavailable",
        "code_review.toast.cannot_attach_terminal_running",
        "code_review.toast.comments_failed",
        "code_review.toast.comments_sent",
        "code_review.toast.diff_removed",
        "code_review.tooltip.add_file_diff_as_context",
        "code_review.tooltip.copy_file_path",
        "code_review.tooltip.initialize_codebase",
        "code_review.tooltip.open_file",
        "code_review.tooltip.open_repository",
        "code_review.tooltip.unsaved_changes",
        "code_review.tooltip.view_changes",
        "context_chips.directory.parent",
        "context_chips.disabled.requires_command",
        "context_chips.disabled.requires_github_cli",
        "context_chips.disabled.requires_local_session",
        "context_chips.menu.copy_chip",
        "context_chips.node.install_latest_command",
        "context_chips.node.install_nvm",
        "context_chips.node.install_nvm_empty_description",
        "context_chips.node.install_nvm_empty_title",
        "context_chips.node.installed",
        "context_chips.node.no_versions_description",
        "context_chips.node.no_versions_title",
        "context_chips.quota.monthly_reset",
        "context_chips.tooltip.change_git_branch",
        "context_chips.tooltip.change_working_directory",
        "context_chips.tooltip.view_pull_request",
        "context_chips.tooltip.working_directory",
        "drive.confirmation.cancel",
        "drive.confirmation.delete_team.body",
        "drive.confirmation.delete_team.confirm",
        "drive.confirmation.delete_team.title",
        "drive.confirmation.empty_trash.body",
        "drive.confirmation.empty_trash.confirm",
        "drive.confirmation.empty_trash.title",
        "drive.confirmation.leave_team.body",
        "drive.confirmation.leave_team.confirm",
        "drive.confirmation.leave_team.title",
        "workspace.lightbox.no_images",
        "workspace.conversation.empty.description",
        "workspace.conversation.empty.title",
        "workspace.conversation.fallback_title",
        "workspace.conversation.fork",
        "workspace.conversation.fork_current",
        "workspace.conversation.new",
        "workspace.conversation.untitled",
        "workspace.conversation_list.error.delete_in_progress",
        "workspace.conversation_list.menu.delete",
        "workspace.conversation_list.menu.fork_new_pane",
        "workspace.conversation_list.menu.fork_new_tab",
        "workspace.conversation_list.menu.share",
        "workspace.conversation_list.no_matching_conversations",
        "workspace.conversation_list.search_placeholder",
        "workspace.conversation_list.section.active",
        "workspace.conversation_list.section.past",
        "workspace.conversation_list.show_less",
        "workspace.conversation_list.tooltip.ambient_delete_disabled",
        "workspace.conversation_list.view_all",
        "workspace.header_toolbar.editor.available_items",
        "workspace.header_toolbar.editor.title",
        "workspace.header_toolbar.item.agent_management",
        "workspace.header_toolbar.item.code_review",
        "workspace.header_toolbar.item.notifications",
        "workspace.header_toolbar.item.tabs_panel",
        "workspace.header_toolbar.item.tools_panel",
        "workspace.hoa_onboarding.action.dismiss",
        "workspace.hoa_onboarding.action.finish",
        "workspace.hoa_onboarding.action.next",
        "workspace.hoa_onboarding.action.see_whats_new",
        "workspace.hoa_onboarding.agent_inbox.description",
        "workspace.hoa_onboarding.agent_inbox.learn_more",
        "workspace.hoa_onboarding.agent_inbox.title",
        "workspace.hoa_onboarding.tab_config.description",
        "workspace.hoa_onboarding.tab_config.title",
        "workspace.hoa_onboarding.vertical_tabs.description",
        "workspace.hoa_onboarding.vertical_tabs.switch_horizontal",
        "workspace.hoa_onboarding.vertical_tabs.title",
        "workspace.hoa_onboarding.welcome.badge_new",
        "workspace.hoa_onboarding.welcome.feature.agent_inbox.description",
        "workspace.hoa_onboarding.welcome.feature.agent_inbox.title",
        "workspace.hoa_onboarding.welcome.feature.code_review.description",
        "workspace.hoa_onboarding.welcome.feature.code_review.title",
        "workspace.hoa_onboarding.welcome.feature.tab_configs.description",
        "workspace.hoa_onboarding.welcome.feature.tab_configs.title",
        "workspace.hoa_onboarding.welcome.feature.vertical_tabs.description",
        "workspace.hoa_onboarding.welcome.feature.vertical_tabs.title",
        "workspace.hoa_onboarding.welcome.title",
        "workspace.launch_modal.oz.action.next",
        "workspace.launch_modal.oz.action.skip_for_now",
        "workspace.launch_modal.oz.action.try_it_out",
        "workspace.launch_modal.oz.agent_automations.content",
        "workspace.launch_modal.oz.agent_automations.short_label",
        "workspace.launch_modal.oz.agent_automations.tab",
        "workspace.launch_modal.oz.agent_automations.title",
        "workspace.launch_modal.oz.agent_management.content",
        "workspace.launch_modal.oz.agent_management.short_label",
        "workspace.launch_modal.oz.agent_management.tab",
        "workspace.launch_modal.oz.agent_management.title",
        "workspace.launch_modal.oz.checkbox.description",
        "workspace.launch_modal.oz.checkbox.sync_conversations",
        "workspace.launch_modal.oz.cloud_agents.content",
        "workspace.launch_modal.oz.cloud_agents.short_label",
        "workspace.launch_modal.oz.cloud_agents.tab",
        "workspace.launch_modal.oz.cloud_agents.title",
        "workspace.launch_modal.oz.launch_credits.content",
        "workspace.launch_modal.oz.launch_credits.short_label",
        "workspace.launch_modal.oz.launch_credits.tab",
        "workspace.launch_modal.oz.launch_credits.title",
        "workspace.launch_modal.oz.modal_subtext",
        "workspace.launch_modal.oz.modal_title",
        "workspace.rewind.cancel",
        "workspace.rewind.confirm",
        "workspace.rewind.description",
        "workspace.rewind.manual_files_note",
        "workspace.rewind.title",
        "workspace.sync_inputs.stop_synchronizing_any_panes",
        "workspace.sync_inputs.toggle_all_panes_all_tabs",
        "workspace.sync_inputs.toggle_all_panes_current_tab",
        "settings.nav.about",
        "settings.nav.account",
        "settings.nav.agent_mcp_servers",
        "settings.nav.agent_profiles",
        "settings.nav.ai",
        "settings.nav.appearance",
        "settings.nav.billing_and_usage",
        "settings.nav.cloud_environments",
        "settings.nav.code",
        "settings.nav.code_indexing",
        "settings.nav.editor_and_code_review",
        "settings.nav.features",
        "settings.nav.keyboard_shortcuts",
        "settings.nav.knowledge",
        "settings.nav.mcp_servers",
        "settings.nav.oz_cloud_api_keys",
        "settings.nav.privacy",
        "settings.nav.referrals",
        "settings.nav.shared_blocks",
        "settings.nav.teams",
        "settings.nav.third_party_cli_agents",
        "settings.nav.umbrella.agents",
        "settings.nav.umbrella.cloud_platform",
        "settings.nav.umbrella.code",
        "settings.nav.warp_agent",
        "settings.nav.warp_drive",
        "settings.nav.warpify",
        "quit_warning.action.cancel",
        "quit_warning.action.dont_save",
        "quit_warning.action.save",
        "quit_warning.action.show_running_processes",
        "quit_warning.action.yes_close",
        "quit_warning.action.yes_quit",
        "quit_warning.editor.this_file",
        "quit_warning.editor.unsaved_changes",
        "quit_warning.process.running.in_tabs",
        "quit_warning.process.running.in_windows",
        "quit_warning.process.running.plural",
        "quit_warning.process.running.scope.plural",
        "quit_warning.process.running.scope.singular",
        "quit_warning.process.running.singular",
        "quit_warning.scope.this_pane",
        "quit_warning.scope.this_tab",
        "quit_warning.scope.this_window",
        "quit_warning.shared_sessions.plural",
        "quit_warning.shared_sessions.scope.plural",
        "quit_warning.shared_sessions.scope.singular",
        "quit_warning.shared_sessions.singular",
        "quit_warning.title.close_pane",
        "quit_warning.title.close_tab",
        "quit_warning.title.close_tabs",
        "quit_warning.title.close_window",
        "quit_warning.title.quit_warp",
        "quit_warning.title.save_changes",
        "quit_warning.unsaved_file_changes",
        "quit_warning.unsaved_file_changes.scope",
        "reward.a11y.help",
        "reward.button.try_it_out",
        "reward.subtitle.received_referral",
        "reward.subtitle.sent_referral",
        "reward.title",
        "resource_center.keybindings.section.blocks",
        "resource_center.keybindings.section.essentials",
        "resource_center.keybindings.section.fundamentals",
        "resource_center.keybindings.section.input_editor",
        "resource_center.keybindings.section.terminal",
        "resource_center.keybindings.settings_instructions",
        "resource_center.keybindings.settings_link",
        "resource_center.keybindings.toggle_this_panel",
        "resource_center.additional_keybinding.hide_others",
        "resource_center.additional_keybinding.hide_warp",
        "resource_center.additional_keybinding.minimize",
        "resource_center.additional_keybinding.new_window",
        "resource_center.additional_keybinding.quit_warp",
        "resource_center.changelog.fetch_error",
        "resource_center.changelog.loading",
        "resource_center.changelog.read_all",
        "resource_center.content.custom_prompt.description",
        "resource_center.content.custom_prompt.title",
        "resource_center.content.how_warp_uses_warp.description",
        "resource_center.content.how_warp_uses_warp.title",
        "resource_center.content.ide.description",
        "resource_center.content.ide.title",
        "resource_center.content.read_article",
        "resource_center.content.view_documentation",
        "resource_center.feature.ai_command_search.description",
        "resource_center.feature.ai_command_search.title",
        "resource_center.feature.block_action.description",
        "resource_center.feature.block_action.title",
        "resource_center.feature.command_palette.description",
        "resource_center.feature.command_palette.title",
        "resource_center.feature.command_search.description",
        "resource_center.feature.command_search.title",
        "resource_center.feature.create_block.description",
        "resource_center.feature.create_block.title",
        "resource_center.feature.launch_configuration.description",
        "resource_center.feature.launch_configuration.title",
        "resource_center.feature.navigate_blocks.description",
        "resource_center.feature.navigate_blocks.title",
        "resource_center.feature.split_panes.description",
        "resource_center.feature.split_panes.title",
        "resource_center.feature.theme_picker.description",
        "resource_center.feature.theme_picker.title",
        "resource_center.footer.docs",
        "resource_center.footer.feedback",
        "resource_center.footer.slack",
        "resource_center.header.keyboard_shortcuts",
        "resource_center.header.warp_essentials",
        "resource_center.invite_friend",
        "resource_center.mark_all_as_read",
        "resource_center.section.advanced_setup",
        "resource_center.section.getting_started",
        "resource_center.section.maximize_warp",
        "resource_center.section.whats_new",
        "settings.action.add",
        "settings.action.clear",
        "settings.action.copied",
        "settings.action.copy",
        "settings.action.default",
        "settings.action.edit",
        "settings.action.enable",
        "settings.action.learn_more",
        "settings.action.reset_to_default",
        "settings.action.saving",
        "settings.action.share",
        "settings.about.copyright",
        "settings.ai.active.git_operations_autogen.description",
        "settings.ai.active.git_operations_autogen.label",
        "settings.ai.active.natural_language_autosuggestions.description",
        "settings.ai.active.natural_language_autosuggestions.label",
        "settings.ai.active.next_command.description",
        "settings.ai.active.next_command.label",
        "settings.ai.active.prompt_suggestions.description",
        "settings.ai.active.prompt_suggestions.label",
        "settings.ai.active.section",
        "settings.ai.active.shared_block_title_generation.description",
        "settings.ai.active.shared_block_title_generation.label",
        "settings.ai.active.suggested_code_banners.description",
        "settings.ai.active.suggested_code_banners.hide_again",
        "settings.ai.active.suggested_code_banners.label",
        "settings.ai.agent_attribution.description",
        "settings.ai.agent_attribution.enable",
        "settings.ai.agent_attribution.section",
        "settings.ai.agents.codebase_context.description",
        "settings.ai.agents.codebase_context.label",
        "settings.ai.agents.command_allowlist.description",
        "settings.ai.agents.command_denylist.description",
        "settings.ai.agents.context_window.label",
        "settings.ai.agents.description",
        "settings.ai.agents.mcp_allowlist.description",
        "settings.ai.agents.mcp_denylist.description",
        "settings.ai.agents.mcp_zero_state.add_server",
        "settings.ai.agents.mcp_zero_state.description",
        "settings.ai.agents.mcp_zero_state.learn_more",
        "settings.ai.agents.mcp_zero_state.or",
        "settings.ai.agents.models.section",
        "settings.ai.agents.permissions.section",
        "settings.ai.agents.profiles.description",
        "settings.ai.agents.profiles.section",
        "settings.ai.agents.section",
        "settings.ai.api_keys.anthropic",
        "settings.ai.api_keys.ask_admin_to_upgrade",
        "settings.ai.api_keys.contact_sales",
        "settings.ai.api_keys.description",
        "settings.ai.api_keys.enterprise_enable_suffix",
        "settings.ai.api_keys.google",
        "settings.ai.api_keys.openai",
        "settings.ai.api_keys.section",
        "settings.ai.api_keys.upgrade_build_plan",
        "settings.ai.api_keys.use_own_keys_suffix",
        "settings.ai.api_keys.warp_credit_fallback.description",
        "settings.ai.api_keys.warp_credit_fallback.label",
        "settings.ai.autodetection.denylist_placeholder",
        "settings.ai.autonomy.read_only",
        "settings.ai.autonomy.supervised",
        "settings.ai.aws_bedrock.credentials.auto_login.description",
        "settings.ai.aws_bedrock.credentials.auto_login.label",
        "settings.ai.aws_bedrock.credentials.description",
        "settings.ai.aws_bedrock.credentials.description_managed",
        "settings.ai.aws_bedrock.credentials.label",
        "settings.ai.aws_bedrock.credentials.login_command",
        "settings.ai.aws_bedrock.credentials.profile",
        "settings.ai.aws_bedrock.credentials.profile_placeholder",
        "settings.ai.aws_bedrock.credentials.refresh",
        "settings.ai.aws_bedrock.credentials.refresh_command_placeholder",
        "settings.ai.cli_agent_toolbar.auto_dismiss_rich_input.label",
        "settings.ai.cli_agent_toolbar.auto_open_rich_input.label",
        "settings.ai.cli_agent_toolbar.auto_toggle_rich_input.label",
        "settings.ai.cli_agent_toolbar.commands.description",
        "settings.ai.cli_agent_toolbar.commands.label",
        "settings.ai.cli_agent_toolbar.commands.placeholder",
        "settings.ai.cli_agent_toolbar.description_or",
        "settings.ai.cli_agent_toolbar.description_prefix",
        "settings.ai.cli_agent_toolbar.label",
        "settings.ai.cli_agent_toolbar.other_agent",
        "settings.ai.cli_agent_toolbar.requires_plugin.tooltip",
        "settings.ai.cli_agent_toolbar.section",
        "settings.ai.cli_agent_toolbar.select_coding_agent",
        "settings.ai.cli_agent_toolbar.submit_rich_input_on_ctrl_enter.label",
        "settings.ai.cloud_agent_computer_use.description",
        "settings.ai.cloud_agent_computer_use.label",
        "settings.ai.cloud_agent_computer_use.section",
        "settings.ai.conversation_layout.new_tab",
        "settings.ai.conversation_layout.split_pane",
        "settings.ai.header.create_account_prompt",
        "settings.ai.model_selector.auto_choice",
        "settings.ai.model_selector.auto_mode.description",
        "settings.ai.model_selector.auto_mode.title",
        "settings.ai.model_selector.banner.base_agent_active",
        "settings.ai.model_selector.banner.full_terminal_use_active",
        "settings.ai.model_selector.a11y.disabled",
        "settings.ai.model_selector.a11y.prefix",
        "settings.ai.model_selector.a11y.selected",
        "settings.ai.model_selector.disabled",
        "settings.ai.model_selector.discount_chip",
        "settings.ai.model_selector.header.model_command",
        "settings.ai.model_selector.manage_api_keys",
        "settings.ai.model_selector.manage_api_keys.tooltip",
        "settings.ai.model_selector.manage_defaults",
        "settings.ai.model_selector.message.select_and_save_to_profile",
        "settings.ai.model_selector.message.to_select",
        "settings.ai.model_selector.new_models_available",
        "settings.ai.model_selector.reasoning_level.description",
        "settings.ai.model_selector.reasoning_level.title",
        "settings.ai.model_selector.request_edit_access",
        "settings.ai.model_selector.selected",
        "settings.ai.model_selector.spec.billed_to_api",
        "settings.ai.model_selector.spec.cost",
        "settings.ai.model_selector.spec.description",
        "settings.ai.model_selector.spec.intelligence",
        "settings.ai.model_selector.spec.speed",
        "settings.ai.model_selector.spec.title",
        "settings.ai.model_selector.tab.base",
        "settings.ai.model_selector.tab.full_terminal_use",
        "settings.ai.model_selector.tooltip",
        "settings.ai.model_selector.upgrade_required.prefix",
        "settings.ai.profile_selector.a11y.prefix",
        "settings.ai.profile_selector.header",
        "settings.ai.profile_selector.manage_profiles",
        "settings.ai.profile_selector.selected",
        "settings.ai.profile_selector.tooltip",
        "settings.ai.input.autodetect_agent_prompts_in_terminal.label",
        "settings.ai.input.autodetect_terminal_commands_in_agent.label",
        "settings.ai.input.feedback.incorrect_detection",
        "settings.ai.input.feedback.incorrect_input_detection",
        "settings.ai.input.feedback.link",
        "settings.ai.input.include_agent_commands_in_history.label",
        "settings.ai.input.natural_language_denylist.description",
        "settings.ai.input.natural_language_denylist.label",
        "settings.ai.input.natural_language_detection.description",
        "settings.ai.input.natural_language_detection.label",
        "settings.ai.input.prompt_submission_mode.description",
        "settings.ai.input.prompt_submission_mode.label",
        "settings.ai.input.section",
        "settings.ai.input.show_agent_tips.label",
        "settings.ai.input.show_hint_text.label",
        "settings.ai.knowledge.manage_rules",
        "settings.ai.knowledge.rules.description",
        "settings.ai.knowledge.rules.label",
        "settings.ai.knowledge.suggested_rules.description",
        "settings.ai.knowledge.suggested_rules.label",
        "settings.ai.knowledge.warp_drive_context.description",
        "settings.ai.knowledge.warp_drive_context.label",
        "settings.ai.mcp.auto_spawn.description",
        "settings.ai.mcp.auto_spawn.label",
        "settings.ai.mcp.auto_spawn.supported_providers",
        "settings.ai.mcp.description",
        "settings.ai.mcp.manage_servers",
        "settings.ai.orchestration.description",
        "settings.ai.orchestration.label",
        "settings.ai.other.conversation_history.label",
        "settings.ai.other.conversation_layout.label",
        "settings.ai.other.orchestration_message_display.description",
        "settings.ai.other.orchestration_message_display.label",
        "settings.ai.other.oz_changelog.label",
        "settings.ai.other.section",
        "settings.ai.other.thinking.description",
        "settings.ai.other.thinking.label",
        "settings.ai.other.thinking.option.always_show",
        "settings.ai.other.thinking.option.never_show",
        "settings.ai.other.thinking.option.show_and_collapse",
        "settings.ai.other.use_agent_footer.description",
        "settings.ai.other.use_agent_footer.label",
        "settings.ai.permission.allow_specific_directories",
        "settings.ai.permission.always_allow",
        "settings.ai.permission.always_ask",
        "settings.ai.permissions.managed_by_workspace",
        "settings.ai.remote_session_org_policy",
        "settings.ai.usage.unlimited",
        "settings.ai.voice_input.activation_key.description",
        "settings.ai.voice_input.activation_key.label",
        "settings.ai.voice_input.description_prefix",
        "settings.ai.voice_input.description_suffix",
        "settings.ai.voice_input.section",
        "settings.billing.action.manage_billing",
        "settings.billing.action.sign_up",
        "settings.billing.addon_credits.auto_reload.description",
        "settings.billing.addon_credits.auto_reload.label",
        "settings.billing.addon_credits.auto_reload.tooltip",
        "settings.billing.addon_credits.auto_reload.warning.delinquent",
        "settings.billing.addon_credits.auto_reload.warning.exceed_limit",
        "settings.billing.addon_credits.auto_reload.warning.failed_reload",
        "settings.billing.addon_credits.buy",
        "settings.billing.addon_credits.buying",
        "settings.billing.addon_credits.contact_account_executive",
        "settings.billing.addon_credits.contact_team_admin",
        "settings.billing.addon_credits.description",
        "settings.billing.addon_credits.exceed_limit.link",
        "settings.billing.addon_credits.exceed_limit.prefix",
        "settings.billing.addon_credits.exceed_limit.suffix",
        "settings.billing.addon_credits.modal_title",
        "settings.billing.addon_credits.monthly_spend_limit",
        "settings.billing.addon_credits.monthly_spend_limit_tooltip",
        "settings.billing.addon_credits.monthly_limit_reached.admin",
        "settings.billing.addon_credits.monthly_limit_reached.non_admin",
        "settings.billing.addon_credits.monthly_limit_reached.title",
        "settings.billing.addon_credits.one_time_purchase",
        "settings.billing.addon_credits.purchase_suffix",
        "settings.billing.addon_credits.purchased_this_month",
        "settings.billing.addon_credits.selected_amount",
        "settings.billing.addon_credits.team_description",
        "settings.billing.addon_credits.title",
        "settings.billing.ambient_trial.buy_more",
        "settings.billing.ambient_trial.credits_remaining",
        "settings.billing.ambient_trial.new_agent",
        "settings.billing.ambient_trial.one_credit_remaining",
        "settings.billing.ambient_trial.title",
        "settings.billing.auto_reload_modal.description.bold",
        "settings.billing.auto_reload_modal.description.learn_more",
        "settings.billing.auto_reload_modal.description.prefix",
        "settings.billing.auto_reload_modal.description.suffix",
        "settings.billing.auto_reload_modal.title",
        "settings.billing.auto_reload_modal.toast.team_data_missing",
        "settings.billing.auto_reload_modal.toast.update_failed",
        "settings.billing.auto_reload_modal.toast.updated",
        "settings.billing.buy_credits_banner.auto_reload_failed",
        "settings.billing.discount_badge",
        "settings.billing.credits.legend.combined_tooltip",
        "settings.billing.credits.many",
        "settings.billing.credits.one",
        "settings.billing.credits.zero",
        "settings.billing.enterprise_usage.admin_link",
        "settings.billing.enterprise_usage.admin_prefix",
        "settings.billing.enterprise_usage.admin_suffix",
        "settings.billing.enterprise_usage.header",
        "settings.billing.enterprise_usage.non_admin",
        "settings.billing.overage.admin_header",
        "settings.billing.overage.description",
        "settings.billing.overage.total",
        "settings.billing.overage.user_description",
        "settings.billing.overage.user_header_disabled",
        "settings.billing.overage.user_header_enabled",
        "settings.billing.overage.view_details",
        "settings.billing.overage_limit.description",
        "settings.billing.overage_limit.error.invalid_currency",
        "settings.billing.overage_limit.error.out_of_range",
        "settings.billing.overage_limit.label",
        "settings.billing.overage_limit.modal_title",
        "settings.billing.overage_limit.note",
        "settings.billing.overage_limit.tooltip",
        "settings.billing.out_of_credits.admin",
        "settings.billing.out_of_credits.exceed_limit.link",
        "settings.billing.out_of_credits.exceed_limit.prefix",
        "settings.billing.out_of_credits.exceed_limit.suffix",
        "settings.billing.out_of_credits.non_admin",
        "settings.billing.out_of_credits.title",
        "settings.billing.page.title",
        "settings.billing.plan.title",
        "settings.billing.sort.display_name_az",
        "settings.billing.sort.display_name_za",
        "settings.billing.sort.tooltip",
        "settings.billing.sort.usage_ascending",
        "settings.billing.sort.usage_descending",
        "settings.billing.tab.overview",
        "settings.billing.tab.usage_history",
        "settings.billing.toast.addon_credits_purchased",
        "settings.billing.toast.update_workspace_failed",
        "settings.billing.upgrade.bring_own_key",
        "settings.billing.upgrade.build",
        "settings.billing.upgrade.business",
        "settings.billing.upgrade.business_suffix",
        "settings.billing.upgrade.contact_admin_billing_issue",
        "settings.billing.upgrade.enterprise",
        "settings.billing.upgrade.enterprise_suffix",
        "settings.billing.upgrade.flexible_pricing_suffix",
        "settings.billing.upgrade.generic",
        "settings.billing.upgrade.increased_access_suffix",
        "settings.billing.upgrade.lightspeed",
        "settings.billing.upgrade.max",
        "settings.billing.upgrade.more_ai_credits_suffix",
        "settings.billing.upgrade.more_ai_usage_suffix",
        "settings.billing.upgrade.more_credits_models_suffix",
        "settings.billing.upgrade.or",
        "settings.billing.upgrade.regain_access_suffix",
        "settings.billing.upgrade.switch_build",
        "settings.billing.upgrade.turbo",
        "settings.billing.usage.credits_description",
        "settings.billing.usage.credits_header",
        "settings.billing.usage.prorated_current_user",
        "settings.billing.usage.prorated_other_user",
        "settings.billing.usage.resets",
        "settings.billing.usage.resets_on",
        "settings.billing.usage.restricted_billing_issue",
        "settings.billing.usage.team_total",
        "settings.billing.usage.title",
        "settings.billing.usage.used_unlimited",
        "settings.billing.usage_history.empty.description",
        "settings.billing.usage_history.empty.title",
        "settings.billing.usage_history.last_30_days",
        "settings.billing.usage_history.load_more",
        "settings.billing.value.not_set",
        "settings.code.action.index_new_folder",
        "settings.code.auto_index.description",
        "settings.code.auto_index.label",
        "settings.code.auto_index.limit_reached",
        "settings.code.category.editor_code_review",
        "settings.code.category.indexing",
        "settings.code.codebase_indexing.description",
        "settings.code.codebase_indexing.disabled_admin",
        "settings.code.codebase_indexing.disabled_ai",
        "settings.code.codebase_indexing.enabled_admin",
        "settings.code.codebase_indexing.label",
        "settings.code.editor.auto_open_code_review_panel.description",
        "settings.code.editor.auto_open_code_review_panel.label",
        "settings.code.editor.global_file_search.description",
        "settings.code.editor.global_file_search.label",
        "settings.code.editor.project_explorer.description",
        "settings.code.editor.project_explorer.label",
        "settings.code.editor.show_code_review_button.description",
        "settings.code.editor.show_code_review_button.label",
        "settings.code.editor.show_diff_stats.description",
        "settings.code.editor.show_diff_stats.label",
        "settings.code.indexing.status.no_index_created",
        "settings.code.indexing_ignore.description",
        "settings.code.initialization_settings",
        "settings.code.subpage.editor_code_review",
        "settings.code.subpage.indexing",
        "settings.code.title",
        "settings.command_palette.ai.oz_changelog",
        "settings.command_palette.ai.thinking.always_show",
        "settings.command_palette.ai.thinking.never_show",
        "settings.command_palette.ai.thinking.show_and_collapse",
        "settings.command_palette.ai.use_agent_footer",
        "settings.command_palette.debug.in_band_command_blocks",
        "settings.command_palette.debug.in_band_generators",
        "settings.command_palette.debug.initialization_block",
        "settings.command_palette.debug.memory_statistics",
        "settings.command_palette.debug.network_status",
        "settings.command_palette.debug.recording_mode",
        "settings.command_palette.disable",
        "settings.command_palette.enable",
        "settings.command_palette.input.pin_bottom",
        "settings.command_palette.input.pin_top",
        "settings.command_palette.input.start_top",
        "settings.command_palette.input.toggle_mode",
        "settings.command_palette.tab_bar.always_show",
        "settings.command_palette.tab_bar.hide_fullscreen",
        "settings.command_palette.tab_bar.on_hover",
        "settings.keybindings.command_column",
        "settings.keybindings.conflict_warning",
        "settings.keybindings.description",
        "settings.keybindings.not_synced_tooltip",
        "settings.keybindings.press_new_shortcut",
        "settings.keybindings.search_placeholder",
        "settings.keybindings.title",
        "settings.keybindings.use_shortcut_prefix",
        "settings.keybindings.use_shortcut_suffix",
        "settings.info.learn_more_tooltip",
        "settings.local_only.tooltip",
        "settings.execution_profile.add_profile",
        "settings.environment.action.new_environment",
        "settings.environment.agent_assisted.add_repo",
        "settings.environment.agent_assisted.all_repos_selected",
        "settings.environment.agent_assisted.available_indexed_repos",
        "settings.environment.agent_assisted.description",
        "settings.environment.agent_assisted.description_indexed",
        "settings.environment.agent_assisted.error.no_directory_selected",
        "settings.environment.agent_assisted.error.not_git_repo",
        "settings.environment.agent_assisted.loading_indexed_repos",
        "settings.environment.agent_assisted.local_repo_unavailable",
        "settings.environment.agent_assisted.no_indexed_repos",
        "settings.environment.agent_assisted.no_repos_selected",
        "settings.environment.agent_assisted.selected_repos",
        "settings.environment.agent_assisted.title",
        "settings.environment.card.env_id",
        "settings.environment.card.image",
        "settings.environment.card.last_edited",
        "settings.environment.card.last_used",
        "settings.environment.card.last_used_never",
        "settings.environment.card.repos",
        "settings.environment.card.setup_commands",
        "settings.environment.card.view_my_runs",
        "settings.environment.delete_confirmation.description",
        "settings.environment.delete_confirmation.title",
        "settings.environment.empty_state.authorize",
        "settings.environment.empty_state.get_started",
        "settings.environment.empty_state.launch_agent",
        "settings.environment.empty_state.loading",
        "settings.environment.empty_state.quick_setup.subtitle",
        "settings.environment.empty_state.quick_setup.title",
        "settings.environment.empty_state.retry",
        "settings.environment.empty_state.subtitle",
        "settings.environment.empty_state.suggested",
        "settings.environment.empty_state.title",
        "settings.environment.empty_state.use_agent.subtitle",
        "settings.environment.empty_state.use_agent.title",
        "settings.environment.error.create_not_logged_in",
        "settings.environment.error.save_missing_environment",
        "settings.environment.error.share_no_team",
        "settings.environment.error.share_not_synced",
        "settings.environment.form.create",
        "settings.environment.form.create_environment",
        "settings.environment.form.delete",
        "settings.environment.form.description.character_count",
        "settings.environment.form.description.label",
        "settings.environment.form.description.placeholder",
        "settings.environment.form.docker_image.label",
        "settings.environment.form.docker_image.open",
        "settings.environment.form.docker_image.placeholder",
        "settings.environment.form.edit_environment",
        "settings.environment.form.loading",
        "settings.environment.form.name.label",
        "settings.environment.form.name.placeholder",
        "settings.environment.form.orchestration.docker_image.label",
        "settings.environment.form.orchestration.docker_image.placeholder",
        "settings.environment.form.orchestration.name.placeholder",
        "settings.environment.form.orchestration.repos.placeholder_authed",
        "settings.environment.form.orchestration.setup_commands.helper",
        "settings.environment.form.orchestration.setup_commands.placeholder",
        "settings.environment.form.repos.auth_with_github",
        "settings.environment.form.repos.configure_access",
        "settings.environment.form.repos.empty",
        "settings.environment.form.repos.error.load_failed",
        "settings.environment.form.repos.error.load_failed_short",
        "settings.environment.form.repos.error.load_failed_with_error",
        "settings.environment.form.repos.helper",
        "settings.environment.form.repos.label",
        "settings.environment.form.repos.missing_repo",
        "settings.environment.form.repos.placeholder_authed",
        "settings.environment.form.repos.placeholder_unauthed",
        "settings.environment.form.retry",
        "settings.environment.form.save",
        "settings.environment.form.save_environment",
        "settings.environment.form.setup_commands.helper",
        "settings.environment.form.setup_commands.label",
        "settings.environment.form.setup_commands.placeholder",
        "settings.environment.form.suggest_image.auth_required",
        "settings.environment.form.suggest_image.authenticate",
        "settings.environment.form.suggest_image.button",
        "settings.environment.form.suggest_image.error.failed",
        "settings.environment.form.suggest_image.error.failed_with_error",
        "settings.environment.form.suggest_image.error.unknown",
        "settings.environment.form.suggest_image.generating",
        "settings.environment.form.suggest_image.launch_agent",
        "settings.environment.form.suggest_image.no_match",
        "settings.environment.form.suggest_image.tooltip",
        "settings.environment.page.description",
        "settings.environment.page.title",
        "settings.environment.search.no_matches",
        "settings.environment.search.placeholder",
        "settings.environment.section.personal",
        "settings.environment.section.shared_by_team",
        "settings.environment.section.shared_by_your_team",
        "settings.environment.toast.created_success",
        "settings.environment.toast.deleted_success",
        "settings.environment.toast.share_failed",
        "settings.environment.toast.shared_success",
        "settings.environment.toast.updated_success",
        "settings.mcp.confirmation.delete_local.description",
        "settings.mcp.confirmation.delete_local.title",
        "settings.mcp.confirmation.delete_shared.description",
        "settings.mcp.confirmation.delete_shared.title",
        "settings.mcp.confirmation.unshare.description",
        "settings.mcp.confirmation.unshare.title",
        "settings.mcp.card.action.edit",
        "settings.mcp.card.action.edit_config",
        "settings.mcp.card.action.log_out",
        "settings.mcp.card.action.set_up",
        "settings.mcp.card.action.share_server",
        "settings.mcp.card.action.show_logs",
        "settings.mcp.card.action.update_available",
        "settings.mcp.card.action.view_logs",
        "settings.mcp.card.debug.cloud_template_missing",
        "settings.mcp.card.debug.gallery_id",
        "settings.mcp.card.debug.none",
        "settings.mcp.card.debug.template_sync_id",
        "settings.mcp.card.status.authenticating",
        "settings.mcp.card.status.offline",
        "settings.mcp.card.status.shutting_down",
        "settings.mcp.card.status.starting_server",
        "settings.mcp.card.tools.available_count",
        "settings.mcp.card.tools.none_available",
        "settings.mcp.edit.delete_mcp",
        "settings.mcp.edit.edit_variables",
        "settings.mcp.edit.editing_disabled",
        "settings.mcp.edit.error.contains_secrets",
        "settings.mcp.edit.error.multiple_servers_in_single_edit",
        "settings.mcp.edit.error.no_server_specified",
        "settings.mcp.edit.json",
        "settings.mcp.edit.log_out",
        "settings.mcp.edit.remove_from_team",
        "settings.mcp.edit.title.add",
        "settings.mcp.edit.title.edit",
        "settings.mcp.edit.title.edit_named",
        "settings.mcp.install.install",
        "settings.mcp.install.no_server_selected",
        "settings.mcp.install.source.from_another_device",
        "settings.mcp.install.source.shared_from_team",
        "settings.mcp.install.title",
        "settings.mcp.list.available_to_install",
        "settings.mcp.list.description",
        "settings.mcp.list.detected_from_config_file",
        "settings.mcp.list.empty_state",
        "settings.mcp.list.file_based.auto_spawn_label",
        "settings.mcp.list.file_based.description",
        "settings.mcp.list.file_based.supported_providers_link",
        "settings.mcp.list.learn_more",
        "settings.mcp.list.no_search_results",
        "settings.mcp.list.search_placeholder",
        "settings.mcp.list.section.detected_from",
        "settings.mcp.list.section.my_mcps",
        "settings.mcp.list.section.shared_by_warp_and_devices",
        "settings.mcp.list.section.shared_by_warp_and_team",
        "settings.mcp.list.section.shared_from_warp",
        "settings.mcp.list.title_chip.from_another_device",
        "settings.mcp.list.title_chip.global",
        "settings.mcp.list.title_chip.shared_by_creator",
        "settings.mcp.list.title_chip.shared_by_team_member",
        "settings.mcp.list.update_success",
        "settings.mcp.page.error.cannot_install_from_link",
        "settings.mcp.page.error.finish_current_install",
        "settings.mcp.page.error.unknown_server",
        "settings.mcp.page.logged_out",
        "settings.mcp.page.logged_out_named",
        "settings.mcp.page.title",
        "settings.mcp.update.description",
        "settings.mcp.update.from",
        "settings.mcp.update.no_updates",
        "settings.mcp.update.publisher.another_device",
        "settings.mcp.update.publisher.team_member",
        "settings.mcp.update.server",
        "settings.mcp.update.title",
        "settings.mcp.update.version",
        "settings.execution_profile.apply_code_diffs",
        "settings.execution_profile.ask_questions",
        "settings.execution_profile.auto",
        "settings.execution_profile.auto_sync_plans",
        "settings.execution_profile.base_model",
        "settings.execution_profile.call_mcp_servers",
        "settings.execution_profile.call_web_tools",
        "settings.execution_profile.command_allowlist",
        "settings.execution_profile.command_denylist",
        "settings.execution_profile.computer_use",
        "settings.execution_profile.directory_allowlist",
        "settings.execution_profile.edit",
        "settings.execution_profile.editor.apply_code_diffs",
        "settings.execution_profile.editor.ask_questions",
        "settings.execution_profile.editor.base_model",
        "settings.execution_profile.editor.base_model_description",
        "settings.execution_profile.editor.call_mcp_servers",
        "settings.execution_profile.editor.call_web_tools",
        "settings.execution_profile.editor.call_web_tools_description",
        "settings.execution_profile.editor.command_allowlist",
        "settings.execution_profile.editor.command_allowlist_description",
        "settings.execution_profile.editor.command_allowlist_placeholder",
        "settings.execution_profile.editor.command_denylist",
        "settings.execution_profile.editor.command_denylist_description",
        "settings.execution_profile.editor.command_denylist_placeholder",
        "settings.execution_profile.editor.computer_use",
        "settings.execution_profile.editor.computer_use_model",
        "settings.execution_profile.editor.computer_use_model_description",
        "settings.execution_profile.editor.context_window",
        "settings.execution_profile.editor.context_window_description",
        "settings.execution_profile.editor.default_profile_name",
        "settings.execution_profile.editor.default_profile_name_locked",
        "settings.execution_profile.editor.delete_profile",
        "settings.execution_profile.editor.directory_allowlist",
        "settings.execution_profile.editor.directory_allowlist_description",
        "settings.execution_profile.editor.directory_allowlist_placeholder",
        "settings.execution_profile.editor.execute_commands",
        "settings.execution_profile.editor.full_terminal_use_model",
        "settings.execution_profile.editor.full_terminal_use_model_description",
        "settings.execution_profile.editor.interact_with_running_commands",
        "settings.execution_profile.editor.mcp_allowlist",
        "settings.execution_profile.editor.mcp_allowlist_description",
        "settings.execution_profile.editor.mcp_denylist",
        "settings.execution_profile.editor.mcp_denylist_description",
        "settings.execution_profile.editor.name",
        "settings.execution_profile.editor.permission_description.agent_decides",
        "settings.execution_profile.editor.permission_description.always_allow",
        "settings.execution_profile.editor.permission_description.always_ask",
        "settings.execution_profile.editor.permission_description.always_ask_running_command",
        "settings.execution_profile.editor.permission_description.ask_on_first_write",
        "settings.execution_profile.editor.permission_description.ask_questions_always",
        "settings.execution_profile.editor.permission_description.ask_questions_never",
        "settings.execution_profile.editor.permission_description.ask_unless_auto_approve",
        "settings.execution_profile.editor.permission_description.computer_use_always_allow",
        "settings.execution_profile.editor.permission_description.computer_use_always_ask",
        "settings.execution_profile.editor.permission_description.computer_use_never",
        "settings.execution_profile.editor.permission_description.unknown",
        "settings.execution_profile.editor.pane_title",
        "settings.execution_profile.editor.plan_auto_sync",
        "settings.execution_profile.editor.plan_auto_sync_description",
        "settings.execution_profile.editor.profile_name_placeholder",
        "settings.execution_profile.editor.read_files",
        "settings.execution_profile.editor.select_mcp_servers",
        "settings.execution_profile.editor.title",
        "settings.execution_profile.editor.unknown_mcp_server",
        "settings.execution_profile.editor.upgrade_footer",
        "settings.execution_profile.editor.upgrade_footer.link",
        "settings.execution_profile.execute_commands",
        "settings.execution_profile.full_terminal_use",
        "settings.execution_profile.interact_with_running_commands",
        "settings.execution_profile.long_context_pricing_warning.learn_more",
        "settings.execution_profile.mcp_allowlist",
        "settings.execution_profile.mcp_denylist",
        "settings.execution_profile.model.disable_reason.admin_disabled",
        "settings.execution_profile.model.disable_reason.out_of_requests",
        "settings.execution_profile.model.disable_reason.provider_outage",
        "settings.execution_profile.model.disable_reason.requires_upgrade",
        "settings.execution_profile.model.disable_reason.unavailable",
        "settings.execution_profile.model.disabled",
        "settings.execution_profile.model.profile_default",
        "settings.execution_profile.models",
        "settings.execution_profile.none",
        "settings.execution_profile.permission.agent_decides",
        "settings.execution_profile.permission.always_allow",
        "settings.execution_profile.permission.always_ask",
        "settings.execution_profile.permission.ask_on_first_write",
        "settings.execution_profile.permission.ask_unless_auto_approve",
        "settings.execution_profile.permission.never",
        "settings.execution_profile.permission.never_ask",
        "settings.execution_profile.permission.off",
        "settings.execution_profile.permission.on",
        "settings.execution_profile.permission.unknown",
        "settings.execution_profile.permissions",
        "settings.execution_profile.read_files",
        "settings.error.settings_file.heading",
        "settings.error.settings_file.heading_many",
        "settings.error.settings_file.invalid_multiple_description",
        "settings.error.settings_file.invalid_single_description",
        "settings.error.settings_file.parse_description",
        "settings.footer.fix_with_oz",
        "settings.footer.open_file",
        "settings.footer.open_settings_file",
        "settings.import.import",
        "settings.import.loading",
        "settings.import.new_session_notice",
        "settings.import.reset_to_defaults",
        "settings.import.summary.other_plural",
        "settings.import.summary.other_singular",
        "settings.import.summary.theme",
        "settings.import.summary.theme_comma",
        "settings.import.welcome",
        "settings.platform.api_keys.create",
        "settings.platform.api_keys.create_api_key",
        "settings.platform.api_keys.creating",
        "settings.platform.api_keys.date",
        "settings.platform.api_keys.default_name",
        "settings.platform.api_keys.deleted",
        "settings.platform.api_keys.description",
        "settings.platform.api_keys.description.agent",
        "settings.platform.api_keys.description.personal",
        "settings.platform.api_keys.description.team",
        "settings.platform.api_keys.documentation",
        "settings.platform.api_keys.empty.description",
        "settings.platform.api_keys.empty.title",
        "settings.platform.api_keys.error.create_failed",
        "settings.platform.api_keys.error.delete_failed",
        "settings.platform.api_keys.error.load_agents_failed",
        "settings.platform.api_keys.error.no_agent_selected",
        "settings.platform.api_keys.error.no_current_team",
        "settings.platform.api_keys.expiration.label",
        "settings.platform.api_keys.expiration.never",
        "settings.platform.api_keys.expiration.ninety_days",
        "settings.platform.api_keys.expiration.one_day",
        "settings.platform.api_keys.expiration.thirty_days",
        "settings.platform.api_keys.header.created",
        "settings.platform.api_keys.header.expires_at",
        "settings.platform.api_keys.header.key",
        "settings.platform.api_keys.header.last_used",
        "settings.platform.api_keys.header.name",
        "settings.platform.api_keys.header.scope",
        "settings.platform.api_keys.modal_title.new",
        "settings.platform.api_keys.modal_title.save",
        "settings.platform.api_keys.name",
        "settings.platform.api_keys.never",
        "settings.platform.api_keys.no_search_results",
        "settings.platform.api_keys.secret_copied",
        "settings.platform.api_keys.secret_once",
        "settings.platform.api_keys.search_placeholder",
        "settings.platform.api_keys.title",
        "settings.platform.api_keys.type",
        "settings.platform.api_keys.type.agent",
        "settings.platform.api_keys.type.personal",
        "settings.platform.api_keys.type.team",
        "settings.features.category.general",
        "settings.features.category.keys",
        "settings.features.category.notifications",
        "settings.features.category.session",
        "settings.features.category.system",
        "settings.features.category.terminal",
        "settings.features.category.terminal_input",
        "settings.features.category.text_editing",
        "settings.features.category.workflows",
        "settings.features.extra_meta_keys.left_alt",
        "settings.features.extra_meta_keys.left_option",
        "settings.features.extra_meta_keys.right_alt",
        "settings.features.extra_meta_keys.right_option",
        "settings.features.global_hotkey.docs_link",
        "settings.features.global_hotkey.configure",
        "settings.features.global_hotkey.label",
        "settings.features.global_hotkey.option.activation_hotkey",
        "settings.features.global_hotkey.option.disabled",
        "settings.features.global_hotkey.option.quake_mode",
        "settings.features.global_hotkey.unsupported_wayland",
        "settings.privacy.add_all",
        "settings.privacy.add_regex",
        "settings.privacy.add_regex_pattern",
        "settings.privacy.cloud_conversation_storage.description_disabled",
        "settings.privacy.cloud_conversation_storage.description_enabled",
        "settings.privacy.cloud_conversation_storage.title",
        "settings.privacy.crash_reports.description",
        "settings.privacy.crash_reports.title",
        "settings.privacy.custom_secret_redaction.description",
        "settings.privacy.enterprise_regex.empty",
        "settings.privacy.enterprise_secret_redaction.locked",
        "settings.privacy.enterprise_tab",
        "settings.privacy.invalid_regex",
        "settings.privacy.name_optional",
        "settings.privacy.name_placeholder",
        "settings.privacy.network_log.description",
        "settings.privacy.network_log.link",
        "settings.privacy.network_log.title",
        "settings.privacy.personal_tab",
        "settings.privacy.regex_pattern",
        "settings.privacy.regex_placeholder",
        "settings.privacy.custom_secret_redaction.title",
        "settings.privacy.data_management.description",
        "settings.privacy.data_management.link",
        "settings.privacy.data_management.title",
        "settings.privacy.organization_enabled",
        "settings.privacy.privacy_policy.link",
        "settings.privacy.privacy_policy.title",
        "settings.privacy.read_more_data_use",
        "settings.privacy.recommended",
        "settings.privacy.secret_redaction.description",
        "settings.privacy.secret_redaction.title",
        "settings.privacy.secret_visual_redaction.description",
        "settings.privacy.secret_visual_redaction.title",
        "settings.privacy.setting_managed_by_organization",
        "settings.privacy.telemetry.description",
        "settings.privacy.telemetry.description_enterprise",
        "settings.privacy.telemetry.free_tier_note",
        "settings.privacy.telemetry.title",
        "settings.privacy.zdr.tooltip",
        "settings.search.no_matches",
        "settings.search.no_matches_hint",
        "settings.search.placeholder",
        "settings.tooltip.organization_enforced",
        "settings.tooltip.workspace_override",
        "settings.title",
        "settings.transfer_ownership.description",
        "settings.transfer_ownership.transfer",
        "settings.warpify.ssh_warpification.label",
        "tooltip.secret_redaction.not_included",
        "tooltip.secret_redaction.pattern.default",
        "tooltip.secret_redaction.pattern.enterprise",
        "tooltip.secret_redaction.pattern.user",
        "tooltip.secret_redaction.will_not_include",
        "tooltip.secrets_not_sent_to_server",
        "settings.teams.action.cancel_invite",
        "settings.teams.action.contact_admin",
        "settings.teams.action.create",
        "settings.teams.action.delete_team",
        "settings.teams.action.demote_admin",
        "settings.teams.action.invite",
        "settings.teams.action.join",
        "settings.teams.action.leave_team",
        "settings.teams.action.manage_plan",
        "settings.teams.action.open_admin_panel",
        "settings.teams.action.promote_admin",
        "settings.teams.action.remove_domain",
        "settings.teams.action.remove_member",
        "settings.teams.action.reset_links",
        "settings.teams.action.set_domains",
        "settings.teams.action.transfer_ownership",
        "settings.teams.create.description",
        "settings.teams.create.discoverable_domain",
        "settings.teams.create.discoverable_same_domain",
        "settings.teams.create.existing_team_header",
        "settings.teams.create.header",
        "settings.teams.discovery.description",
        "settings.teams.discovery.teammates.many",
        "settings.teams.discovery.teammates.one",
        "settings.teams.error.add_domain",
        "settings.teams.error.billing_issue_admin.line_1",
        "settings.teams.error.billing_issue_admin.line_2_link",
        "settings.teams.error.billing_issue_admin.line_2_prefix",
        "settings.teams.error.billing_issue_admin.line_2_suffix",
        "settings.teams.error.billing_issue_admin_no_self_serve",
        "settings.teams.error.billing_issue_non_admin",
        "settings.teams.error.delete_domain",
        "settings.teams.error.delete_invite",
        "settings.teams.error.generate_billing_link",
        "settings.teams.error.generate_upgrade_link",
        "settings.teams.error.invalid_domains",
        "settings.teams.error.invalid_emails",
        "settings.teams.error.join_team",
        "settings.teams.error.leave_team",
        "settings.teams.error.load_invite_link",
        "settings.teams.error.rename_team",
        "settings.teams.error.send_invite",
        "settings.teams.error.toggle_discoverability",
        "settings.teams.error.toggle_invite_links",
        "settings.teams.error.transfer_ownership",
        "settings.teams.error.update_member_role",
        "settings.teams.invite.email.expiry",
        "settings.teams.invite.email.header",
        "settings.teams.invite.email.invalid",
        "settings.teams.invite.link.domain_invalid",
        "settings.teams.invite.link.domain_restrictions",
        "settings.teams.invite.link.header",
        "settings.teams.invite.link.reset_success",
        "settings.teams.invite.link.toggle",
        "settings.teams.limit.exceeded_admin_no_upgrade",
        "settings.teams.limit.exceeded_admin_upgrade",
        "settings.teams.limit.exceeded_non_admin",
        "settings.teams.limit.hit_admin",
        "settings.teams.limit.hit_admin_no_upgrade",
        "settings.teams.limit.hit_non_admin",
        "settings.teams.offline",
        "settings.teams.placeholder.domains",
        "settings.teams.placeholder.emails",
        "settings.teams.placeholder.new_team_name",
        "settings.teams.placeholder.team_name",
        "settings.teams.plan.free_usage_limits",
        "settings.teams.plan.shared_notebooks",
        "settings.teams.plan.shared_workflows",
        "settings.teams.plan.usage_limits",
        "settings.teams.pricing.additional_members",
        "settings.teams.pricing.additional_members_with_cost",
        "settings.teams.pricing.admin_prorated",
        "settings.teams.pricing.non_admin_prorated",
        "settings.teams.section.discoverability",
        "settings.teams.section.members",
        "settings.teams.section.restrict_domain",
        "settings.teams.status.admin",
        "settings.teams.status.expired",
        "settings.teams.status.owner",
        "settings.teams.status.past_due",
        "settings.teams.status.pending",
        "settings.teams.status.unpaid",
        "settings.teams.success.added_domain_restrictions",
        "settings.teams.success.deleted_invite",
        "settings.teams.success.invite_many",
        "settings.teams.success.invite_one",
        "settings.teams.success.joined_team",
        "settings.teams.success.joined_team_named",
        "settings.teams.success.left_team",
        "settings.teams.success.link_copied",
        "settings.teams.success.renamed_team",
        "settings.teams.success.toggle_discoverability",
        "settings.teams.success.toggle_invite_links",
        "settings.teams.success.transfer_ownership",
        "settings.teams.success.update_member_role",
        "settings.teams.title",
        "settings.warp_drive.description",
        "settings.warp_drive.label",
        "settings.warp_drive.sign_up",
        "settings.warp_drive.sign_up_required",
        "settings.appearance.category.themes",
        "settings.appearance.category.icon",
        "settings.appearance.category.language",
        "settings.appearance.category.window",
        "settings.appearance.category.input",
        "settings.appearance.category.panes",
        "settings.appearance.category.blocks",
        "settings.appearance.category.text",
        "settings.appearance.category.cursor",
        "settings.appearance.category.tabs",
        "settings.appearance.category.full_screen_apps",
        "settings.appearance.theme.create_custom_theme",
        "settings.appearance.theme.mode.light",
        "settings.appearance.theme.mode.dark",
        "settings.appearance.theme.mode.current",
        "settings.appearance.theme.sync_with_os.label",
        "settings.appearance.theme.sync_with_os.description",
        "settings.appearance.app_icon.label",
        "settings.appearance.app_icon.bundle_required",
        "settings.appearance.app_icon.restart_required_macos",
        "settings.appearance.app_icon.option.aurora",
        "settings.appearance.app_icon.option.classic_1",
        "settings.appearance.app_icon.option.classic_2",
        "settings.appearance.app_icon.option.classic_3",
        "settings.appearance.app_icon.option.comets",
        "settings.appearance.app_icon.option.cow",
        "settings.appearance.app_icon.option.default",
        "settings.appearance.app_icon.option.glass_sky",
        "settings.appearance.app_icon.option.glitch",
        "settings.appearance.app_icon.option.glow",
        "settings.appearance.app_icon.option.holographic",
        "settings.appearance.app_icon.option.mono",
        "settings.appearance.app_icon.option.neon",
        "settings.appearance.app_icon.option.original",
        "settings.appearance.app_icon.option.starburst",
        "settings.appearance.app_icon.option.sticker",
        "settings.appearance.app_icon.option.warp_1",
        "settings.appearance.command_palette.match_agent_font",
        "settings.appearance.command_palette.match_notebook_font_size",
        "settings.appearance.window.custom_size.label",
        "settings.appearance.window.custom_size.columns",
        "settings.appearance.window.custom_size.rows",
        "settings.appearance.window.opacity.label",
        "settings.appearance.window.opacity.unsupported",
        "settings.appearance.window.opacity.graphics_warning",
        "settings.appearance.window.opacity.graphics_settings_hint",
        "settings.appearance.window.blur_radius.label",
        "settings.appearance.window.blur_texture.label",
        "settings.appearance.window.blur_texture.unsupported",
        "settings.appearance.window.tools_panel_visibility_across_tabs.label",
        "settings.appearance.input.type.label",
        "settings.appearance.input.mode.label",
        "settings.appearance.panes.dim_inactive.label",
        "settings.appearance.panes.focus_follows_mouse.label",
        "settings.appearance.blocks.compact_mode.label",
        "settings.appearance.blocks.jump_to_bottom.label",
        "settings.appearance.blocks.show_dividers.label",
        "settings.appearance.text.agent_font.label",
        "settings.appearance.text.match_terminal.label",
        "settings.appearance.text.terminal_font.label",
        "settings.appearance.text.view_all_system_fonts.label",
        "settings.appearance.text.font_weight.label",
        "settings.appearance.text.font_size_px.label",
        "settings.appearance.text.line_height.label",
        "settings.appearance.text.reset_to_default.label",
        "settings.appearance.text.notebook_font_size.label",
        "settings.appearance.text.thin_strokes.label",
        "settings.appearance.text.minimum_contrast.label",
        "settings.appearance.text.ligatures.label",
        "settings.appearance.text.ligatures.tooltip",
        "settings.appearance.cursor.type.label",
        "settings.appearance.cursor.type.disabled_vim_mode",
        "settings.appearance.cursor.type.option.bar",
        "settings.appearance.cursor.type.option.block",
        "settings.appearance.cursor.type.option.underline",
        "settings.appearance.cursor.blinking.label",
        "settings.appearance.input.mode.option.pin_bottom",
        "settings.appearance.input.mode.option.pin_top",
        "settings.appearance.input.mode.option.start_top",
        "settings.appearance.input.type.option.warp",
        "settings.appearance.input.type.option.shell_ps1",
        "settings.appearance.text.thin_strokes.option.never",
        "settings.appearance.text.thin_strokes.option.low_dpi",
        "settings.appearance.text.thin_strokes.option.high_dpi",
        "settings.appearance.text.thin_strokes.option.always",
        "settings.appearance.text.minimum_contrast.option.always",
        "settings.appearance.text.minimum_contrast.option.named_colors",
        "settings.appearance.text.minimum_contrast.option.never",
        "settings.appearance.tabs.close_button_position.label",
        "settings.appearance.tabs.close_button_position.option.right",
        "settings.appearance.tabs.close_button_position.option.left",
        "settings.appearance.tabs.indicators.label",
        "settings.appearance.tabs.code_review_button.label",
        "settings.appearance.tabs.preserve_active_color.label",
        "settings.appearance.tabs.vertical_layout.label",
        "settings.appearance.tabs.restore_vertical_panel.label",
        "settings.appearance.tabs.restore_vertical_panel.description",
        "settings.appearance.tabs.latest_prompt_title.label",
        "settings.appearance.tabs.latest_prompt_title.description",
        "settings.appearance.tabs.header_toolbar_layout.label",
        "settings.appearance.tabs.directory_colors.add_button",
        "settings.appearance.tabs.directory_colors.add_directory",
        "settings.appearance.tabs.directory_colors.label",
        "settings.appearance.tabs.directory_colors.description",
        "settings.appearance.tabs.directory_colors.default_no_color",
        "settings.appearance.tabs.tab_bar.label",
        "settings.appearance.tabs.tab_bar.option.always",
        "settings.appearance.tabs.tab_bar.option.windowed",
        "settings.appearance.tabs.tab_bar.option.hover",
        "settings.appearance.full_screen_apps.custom_padding.label",
        "settings.appearance.full_screen_apps.uniform_padding_px.label",
        "settings.appearance.zoom.label",
        "settings.appearance.zoom.description",
        "settings.pane.close",
        "settings.pane.split_down",
        "settings.pane.split_left",
        "settings.pane.split_right",
        "settings.pane.split_up",
        "shared_session.role_change.action.cancel",
        "shared_session.role_change.action.cancel_request",
        "shared_session.role_change.action.approve",
        "shared_session.role_change.action.deny",
        "shared_session.role_change.action.make_editor",
        "shared_session.role_change.edit_requests",
        "shared_session.role_change.grant_warning.line_1",
        "shared_session.role_change.grant_warning.line_2",
        "shared_session.role_change.role.edit",
        "shared_session.role_change.role.view",
        "shared_session.role_change.viewer_requested_mode",
        "shared_session.role_change.waiting_for",
        "shared_session.participant.change_role",
        "shared_session.participant.make_editor",
        "shared_session.participant.make_viewer",
        "shared_session.participant.revoke_all_edit_permissions",
        "shared_session.share_modal.denied.subheader",
        "shared_session.share_modal.denied.title",
        "shared_session.share_modal.denied.view_plans",
        "shared_session.share_modal.disabled.agents",
        "shared_session.share_modal.disabled.size_and_agents",
        "shared_session.share_modal.disabled.size_limit",
        "shared_session.share_modal.option.current_block",
        "shared_session.share_modal.option.current_screen",
        "shared_session.share_modal.option.selected_block",
        "shared_session.share_modal.option.start_of_session",
        "shared_session.share_modal.option.without_scrollback",
        "shared_session.share_modal.start_sharing",
        "shared_session.share_modal.title",
        "terminal.inline_menu.navigation.to_cycle_tabs",
        "terminal.inline_menu.navigation.to_dismiss",
        "terminal.inline_menu.navigation.to_navigate",
        "workflow.toast.generic_error",
        "workflow.toast.metadata_generation_failed",
        "workflow.toast.out_of_ai_credits",
        "workflow.toast.out_of_ai_credits_contact_admin",
        "workflow.enum.action.close",
        "workflow.enum.action.create",
        "workflow.enum.action.save",
        "workflow.enum.placeholder.dynamic_command",
        "workflow.enum.placeholder.name",
        "workflow.enum.placeholder.variant",
        "workflow.enum.title.edit",
        "workflow.enum.title.new",
        "workflow.enum.type.dynamic",
        "workflow.enum.type.static",
        "workflow.enum.variants.title",
        "workflow.action.autofill",
        "workflow.action.loading",
        "workflow.action.new_argument",
        "workflow.action.save",
        "workflow.arguments.placeholder.default_value_optional",
        "workflow.arguments.placeholder.description",
        "workflow.placeholder.command",
        "workflow.placeholder.description",
        "workflow.placeholder.title",
        "workflow.tooltip.ai_assist",
        "workflow.unsaved_changes.discard",
        "workflow.unsaved_changes.keep_editing",
        "workflow.unsaved_changes.message",
        "workspace.action.add_new_repo",
        "workspace.action.check_latest_and_retry",
        "workspace.action.undo",
        "workspace.action.view",
        "workspace.banner.autoupdate.unable_to_launch.description",
        "workspace.banner.autoupdate.unable_to_launch_deprecated.description",
        "workspace.banner.autoupdate.unable_to_update.description",
        "workspace.banner.autoupdate.unable_to_update_deprecated.description",
        "workspace.banner.autoupdate.update_manually",
        "workspace.autoupdate.package_manager.description_prefix",
        "workspace.autoupdate.package_manager.description_suffix",
        "workspace.autoupdate.package_manager.dist_upgrade_prefix",
        "workspace.autoupdate.package_manager.dist_upgrade_suffix",
        "workspace.autoupdate.package_manager.footer_prefix",
        "workspace.autoupdate.package_manager.footer_suffix",
        "workspace.autoupdate.package_manager.press_enter",
        "workspace.autoupdate.package_manager.report_issues",
        "workspace.autoupdate.package_manager.repository_configuration",
        "workspace.autoupdate.package_manager.title",
        "workspace.banner.reauth.description",
        "workspace.banner.reauth.heading",
        "workspace.banner.reauth.sign_in",
        "workspace.banner.version_deprecated.restart_now.button",
        "workspace.banner.version_deprecated.restart_now.description",
        "workspace.banner.version_deprecated.update_now.button",
        "workspace.banner.version_deprecated.update_now.description",
        "workspace.banner.view_changelog",
        "workspace.button.more_info",
        "workspace.conversation.linear_issue_fallback_title",
        "tab.menu.close_other_tabs",
        "tab.menu.close_tab",
        "tab.menu.close_tabs_below",
        "tab.menu.close_tabs_to_right",
        "tab.menu.copy_link",
        "tab.menu.group.close_all_tabs",
        "tab.menu.group.close_tabs_above",
        "tab.menu.group.move_down",
        "tab.menu.group.move_up",
        "tab.menu.group.new_tab_in_group",
        "tab.menu.group.ungroup_tabs",
        "tab.menu.move_down",
        "tab.menu.move_left",
        "tab.menu.move_right",
        "tab.menu.move_to_group",
        "tab.menu.move_up",
        "tab.menu.new_group_with_tab",
        "tab.menu.remove_from_group",
        "tab.menu.rename_tab",
        "tab.menu.reset_tab_name",
        "tab.menu.save_as_new_config",
        "tab.menu.share_session",
        "tab.menu.stop_sharing",
        "tab.menu.stop_sharing_all",
        "workspace.menu.agent",
        "workspace.menu.billing_and_usage",
        "workspace.menu.cloud_oz",
        "workspace.menu.current_version",
        "workspace.menu.documentation",
        "workspace.menu.feedback",
        "workspace.menu.install_update",
        "workspace.menu.invite_a_friend",
        "workspace.menu.keyboard_shortcuts",
        "workspace.menu.local_docker_sandbox",
        "workspace.menu.log_out",
        "workspace.menu.new_tab",
        "workspace.menu.new_tab_config",
        "workspace.menu.tab_configs",
        "workspace.menu.new_worktree_config",
        "workspace.menu.rearrange_toolbar_items",
        "workspace.menu.reopen_closed_session",
        "workspace.menu.settings",
        "workspace.menu.sign_up",
        "workspace.menu.slack",
        "workspace.menu.terminal",
        "workspace.menu.update_and_relaunch_warp",
        "workspace.menu.update_warp_manually",
        "workspace.menu.updating_to",
        "workspace.menu.upgrade",
        "workspace.menu.view_warp_logs",
        "workspace.menu.whats_new",
        "workspace.panel.agent_conversations",
        "workspace.panel.agent_management",
        "workspace.panel.global_search",
        "workspace.panel.notifications",
        "workspace.panel.project_explorer",
        "workspace.panel.tabs",
        "workspace.panel.tools",
        "workspace.panel.warp_drive",
        "workspace.placeholder.feedback",
        "workspace.placeholder.search_recent_repos_and_conversations",
        "workspace.placeholder.search_repos",
        "workspace.placeholder.search_sessions_agents_files",
        "workspace.placeholder.search_tabs",
        "workspace.session_config.tab_config_chip",
        "workspace.tab.install_update",
        "workspace.tab.introducing_oz",
        "workspace.tooltip.offline",
        "workspace.update_ready",
        "workspace.vertical_tabs.and",
        "pane.get_started.binding.terminal_session",
        "pane.get_started.new_session_in",
        "pane.get_started.subtitle",
        "pane.get_started.title",
        "pane.get_started.welcome",
        "pane.header.sharing.copy_link",
        "pane.header.sharing.read_only",
        "pane.header.sharing.read_only_sign_in",
        "pane.header.sharing.share",
        "pane.header.sharing.unsharable_conversation_tooltip",
        "prompt.editor.cancel",
        "prompt.editor.restore_default",
        "prompt.editor.same_line_prompt",
        "prompt.editor.save_changes",
        "prompt.editor.separator",
        "prompt.editor.shell_prompt_section",
        "prompt.editor.title",
        "prompt.editor.warp_prompt_section",
        "search.navigation.current_session",
        "notebook.a11y.label",
        "search.env_var_collection.a11y.label",
        "search.notebook.a11y.label",
        "search.notebook_embedding.not_visible",
        "welcome.binding.add_repository",
        "welcome.binding.add_repository_with_binding",
        "welcome.binding.terminal_session",
        "welcome.binding.terminal_session_with_binding",
        "welcome.title.new_tab",
        "workflow.title.untitled",
        "workspace.vertical_tabs.badge.unsaved",
        "workspace.vertical_tabs.empty.no_search_results",
        "workspace.vertical_tabs.empty.no_tabs_open",
        "workspace.vertical_tabs.group.new",
        "workspace.vertical_tabs.group.tab_count.plural",
        "workspace.vertical_tabs.group.tab_count.singular",
        "common.rename",
        "workspace.binding.rename_current_pane",
        "workspace.binding.set_a11y_concise_announcements",
        "workspace.binding.set_a11y_verbose_announcements",
        "workspace.vertical_tabs.more",
        "workspace.vertical_tabs.new_session",
        "workspace.vertical_tabs.pane_kind.code",
        "workspace.vertical_tabs.pane_kind.code_diff",
        "workspace.vertical_tabs.pane_kind.environment_variables",
        "workspace.vertical_tabs.pane_kind.environments",
        "workspace.vertical_tabs.pane_kind.execution_profile",
        "workspace.vertical_tabs.pane_kind.file",
        "workspace.vertical_tabs.pane_kind.notebook",
        "workspace.vertical_tabs.pane_kind.other",
        "workspace.vertical_tabs.pane_kind.plan",
        "workspace.vertical_tabs.pane_kind.rules",
        "workspace.vertical_tabs.pane_kind.settings",
        "workspace.vertical_tabs.pane_kind.terminal",
        "workspace.vertical_tabs.pane_kind.workflow",
        "workspace.vertical_tabs.settings.additional_metadata",
        "workspace.vertical_tabs.settings.branch",
        "workspace.vertical_tabs.settings.command_conversation",
        "workspace.vertical_tabs.settings.density",
        "workspace.vertical_tabs.settings.diff_stats",
        "workspace.vertical_tabs.settings.focused_session",
        "workspace.vertical_tabs.settings.github_cli_required",
        "workspace.vertical_tabs.settings.pane_title_as",
        "workspace.vertical_tabs.settings.panes",
        "workspace.vertical_tabs.settings.pr_link",
        "workspace.vertical_tabs.settings.show",
        "workspace.vertical_tabs.settings.show_details_on_hover",
        "workspace.vertical_tabs.settings.summary",
        "workspace.vertical_tabs.settings.tab_item",
        "workspace.vertical_tabs.settings.tabs",
        "workspace.vertical_tabs.settings.view_as",
        "workspace.vertical_tabs.settings.working_directory",
        "workspace.vertical_tabs.tooltip.tab_configs",
        "workspace.vertical_tabs.tooltip.view_options",
        "workspace.vertical_tabs.untitled_tab",
        "workspace.toast.cannot_open_new_terminal_session",
        "workspace.toast.command_still_running",
        "workspace.toast.conversation_deleted",
        "workspace.toast.conversation_forking_failed",
        "workspace.toast.disabled_all_synchronized_inputs",
        "workspace.toast.enable_local_network_access",
        "workspace.toast.failed_to_create_log_bundle",
        "workspace.toast.failed_to_delete_conversation",
        "workspace.toast.failed_to_load_conversation",
        "workspace.toast.failed_to_load_conversation_data",
        "workspace.toast.failed_to_load_conversation_for_forking",
        "workspace.toast.failed_to_load_tab_config",
        "workspace.toast.failed_to_remove_tab_config",
        "workspace.toast.failed_to_sample_process",
        "workspace.toast.forked_conversation",
        "workspace.toast.mouse_reporting_disabled",
        "workspace.toast.mouse_reporting_enabled",
        "workspace.toast.no_terminal_pane_for_context",
        "workspace.toast.notification_permission_denied",
        "workspace.toast.open_warp_redirected_to_download",
        "workspace.toast.open_repository_failed",
        "workspace.toast.oz_cli_install_failed",
        "workspace.toast.oz_cli_installed",
        "workspace.toast.oz_cli_uninstall_failed",
        "workspace.toast.oz_cli_uninstalled",
        "workspace.toast.out_of_ai_credits",
        "workspace.toast.plan_already_in_context",
        "workspace.toast.plan_synced_to_drive",
        "workspace.toast.press_to_undo",
        "workspace.toast.process_sample_saved",
        "workspace.toast.remote_control_link_copied",
        "workspace.toast.sampling_process",
        "workspace.toast.staging_api_call_failed",
        "workspace.toast.sync_all_inputs_disabled",
        "workspace.toast.sync_all_inputs_enabled",
        "workspace.toast.sync_tab_inputs_disabled",
        "workspace.toast.sync_tab_inputs_enabled",
        "workspace.toast.troubleshoot_notifications",
        "workspace.toast.upgrade_for_more_credits",
        "workspace.toast.warp_updated",
        "workspace.toast.workflow_no_longer_available",
        "workspace.welcome.ai_assistant.body",
        "workspace.conversation.default_title",
        "workspace.pane.untitled",
        "terminal.warpify.never_warpify_host",
        "workspace.tab.settings",
        "workflow.env_vars.selector.none",
        "workspace.worktree.config_name",
        "workspace.worktree.new",
        "workspace.worktree.new_with_branch",
        "workspace.worktree.new_with_name",
    ];
    let en_us = Catalog::from_json(LocaleId::EnUs, BUNDLED_EN_US).unwrap();
    let zh_cn = Catalog::from_json(LocaleId::ZhCn, BUNDLED_ZH_CN).unwrap();

    for key in required_keys {
        assert_ne!(en_us.get(key), None, "missing en-US key {key}");
        assert_ne!(zh_cn.get(key), None, "missing zh-CN key {key}");
    }
}

#[test]
fn bundled_catalogs_have_matching_keys() {
    let en_us = bundled_en_us_map();
    let zh_cn = bundled_zh_cn_map();

    let missing_zh_cn = en_us
        .keys()
        .filter(|key| !zh_cn.contains_key(*key))
        .collect::<Vec<_>>();
    let extra_zh_cn = zh_cn
        .keys()
        .filter(|key| !en_us.contains_key(*key))
        .collect::<Vec<_>>();

    assert!(
        missing_zh_cn.is_empty(),
        "missing zh-CN keys: {missing_zh_cn:?}"
    );
    assert!(extra_zh_cn.is_empty(), "extra zh-CN keys: {extra_zh_cn:?}");
}

#[test]
fn bundled_catalogs_have_matching_placeholders() {
    let en_us = bundled_en_us_map();
    let zh_cn = bundled_zh_cn_map();

    let mismatches = en_us
        .iter()
        .filter_map(|(key, en_value)| {
            let en_placeholders = placeholders(en_value.as_str()?);
            let zh_placeholders = placeholders(zh_cn.get(key)?.as_str()?);
            (en_placeholders != zh_placeholders).then_some((key, en_placeholders, zh_placeholders))
        })
        .collect::<Vec<_>>();

    assert!(
        mismatches.is_empty(),
        "placeholder mismatches: {mismatches:?}"
    );
}

#[test]
fn bundled_settings_schema_default_text_mappings_have_zh_cn_translations() {
    let en_us = bundled_en_us_map();
    let zh_cn = bundled_zh_cn_map();

    let schema_default_mapping_keys = en_us
        .keys()
        .filter(|key| {
            key.starts_with("settings.schema.defs.")
                || key.starts_with("settings.schema.properties.")
        })
        .collect::<Vec<_>>();

    assert!(
        !schema_default_mapping_keys.is_empty(),
        "expected settings schema default-text mapping keys"
    );

    let failures = schema_default_mapping_keys
        .iter()
        .filter_map(|key| {
            let default_text = en_us.get(*key)?.as_str()?;
            let zh_text = zh_cn.get(*key)?.as_str()?;
            let localized_texts = en_us
                .iter()
                .filter_map(|(candidate_key, value)| {
                    (value.as_str() == Some(default_text))
                        .then(|| zh_cn.get(candidate_key)?.as_str())
                        .flatten()
                        .filter(|text| *text != default_text)
                })
                .collect::<BTreeSet<_>>();

            if zh_text == default_text {
                return Some(format!("{key}: zh-CN text matches en-US default"));
            }

            (localized_texts.len() != 1).then(|| {
                format!(
                    "{key}: en-US default text must map to exactly one zh-CN text, got {localized_texts:?}"
                )
            })
        })
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "settings schema default-text localization failures: {failures:#?}"
    );
}

#[test]
fn settings_schema_translation_keys_match_json_schema_paths() {
    let cases = [
        (
            &["$defs", "AccessibilityVerbosity"][..],
            "description",
            "settings.schema.defs.accessibility_verbosity.description",
        ),
        (
            &["$defs", "AgentModeCodingPermissionsType"][..],
            "description",
            "settings.schema.defs.agent_mode_coding_permissions_type.description",
        ),
        (
            &["$defs", "CLIAgentToolbarChipSelection"][..],
            "description",
            "settings.schema.defs.cliagent_toolbar_chip_selection.description",
        ),
        (
            &["$defs", "InputMode", "oneOf", "0"][..],
            "description",
            "settings.schema.defs.input_mode.one_of.0.description",
        ),
        (
            &["$defs", "CustomTheme", "properties", "path"][..],
            "description",
            "settings.schema.defs.custom_theme.properties.path.description",
        ),
    ];
    let en_us = bundled_en_us_map();
    let zh_cn = bundled_zh_cn_map();

    for (path, field, expected_key) in cases {
        let actual_key = settings_schema_translation_key(path, field);
        assert_eq!(actual_key, expected_key);
        assert!(
            en_us.contains_key(&actual_key),
            "missing en-US catalog key {actual_key}"
        );
        assert!(
            zh_cn.contains_key(&actual_key),
            "missing zh-CN catalog key {actual_key}"
        );
    }
}

#[test]
fn bundled_catalogs_include_onboarding_copy_keys() {
    let keys = [
        "onboarding.callout.meet_input.title",
        "onboarding.callout.meet_input.body",
        "onboarding.callout.talk_to_agent.title",
        "onboarding.callout.talk_to_agent.prompt",
        "onboarding.callout.terminal_mode.title",
        "onboarding.callout.agent_mode.with_project_body",
        "onboarding.common.skip",
        "onboarding.common.next",
        "onboarding.common.finish",
        "onboarding.common.submit",
    ];

    assert_bundled_keys_exist(&keys);
}

#[test]
fn bundled_catalogs_only_use_intentional_empty_values() {
    let en_us = bundled_en_us_map();
    let zh_cn = bundled_zh_cn_map();

    assert_eq!(
        empty_translation_keys(&en_us),
        ALLOWED_EMPTY_TRANSLATION_KEYS
    );
    assert_eq!(
        empty_translation_keys(&zh_cn),
        ALLOWED_EMPTY_TRANSLATION_KEYS
    );
}

#[test]
fn app_localization_key_literals_are_catalog_backed() {
    let mut keys = BTreeSet::new();
    for relative_root in ["app/src", "crates/warp_cli/src", "crates/warp_tui/src"] {
        collect_app_localization_key_literals(&workspace_root().join(relative_root), &mut keys);
    }

    assert!(
        !keys.is_empty(),
        "expected app localization key literals to be discovered"
    );
    let keys = keys.into_iter().collect::<Vec<_>>();
    assert_bundled_keys_exist(&keys);
}

#[test]
fn app_ui_calls_do_not_use_direct_english_literals() {
    let app_src = workspace_root().join("app/src");
    let mut violations = Vec::new();
    collect_direct_ui_literal_violations(&app_src, &mut violations);

    assert!(
        violations.is_empty(),
        "direct user-visible English literals in UI calls: {violations:#?}"
    );
}

#[test]
fn tui_ui_calls_do_not_use_direct_english_literals() {
    let tui_src = workspace_root().join("crates/warp_tui/src");
    let mut violations = Vec::new();
    for pattern in [
        "TuiText::new(",
        "TuiInlineMenuStatus::Loading(",
        "TuiInlineMenuStatus::Empty(",
        "show_transient_hint(",
        "show_success_hint(",
        "render_warping_indicator(",
    ] {
        collect_direct_first_argument_literal_violations_in_dir(
            &tui_src,
            pattern,
            &mut violations,
            None,
        );
    }

    assert!(
        violations.is_empty(),
        "direct user-visible English literals in TUI calls: {violations:#?}"
    );
}

#[test]
fn tui_high_risk_formatting_surfaces_do_not_restore_removed_english_literals() {
    let cases = [
        (
            "crates/warp_tui/src/zero_state.rs",
            &[
                "{running} connected",
                "{starting} starting",
                "{authenticating} needs auth",
                "{stopping} stopping",
                "{failed} failed",
                "{offline} offline",
                "Config error \u{b7} run /mcp",
                "Not configured \u{b7} /mcp",
                "No servers configured \u{b7} run /mcp",
            ][..],
        ),
        (
            "crates/warp_tui/src/tui_markdown.rs",
            &[
                "Image: {description}",
                "Image: {}",
                "[Image without description]",
                "[Unsupported embedded content]",
            ][..],
        ),
        (
            "crates/warp_tui/src/tui_markdown/table.rs",
            &["[Empty table]", "[Table has no rows]"][..],
        ),
        (
            "crates/warp_tui/src/agent_block_sections.rs",
            &["Tasks {}", "Completed {}{position}"][..],
        ),
        ("app/src/tui/mcp.rs", &["Failed to start"][..]),
    ];

    for (relative_path, literals) in cases {
        let path = workspace_root().join(relative_path);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        for literal in literals {
            assert!(
                !content.contains(literal),
                "{relative_path} should use catalog copy instead of direct English literal: {literal}"
            );
        }
    }
}

#[test]
fn tui_attachment_and_generic_permission_surfaces_do_not_bypass_catalogs() {
    let cases = [
        (
            "crates/warp_tui/src/attachment_bar/model.rs",
            &[
                "\"image\".to_owned()",
                "\"clipboard-image.png\".to_owned()",
                "Image attachments are unavailable.",
                "Wait for the current image attachment to finish.",
                "Image attachment limit is {MAX_IMAGE_COUNT_FOR_QUERY} per query.",
                "The selected model does not support image attachments.",
            ][..],
        ),
        (
            "crates/warp_tui/src/attachment_bar/image_processing.rs",
            &[
                "Could not read image {}.",
                "Image path is not a file: {}.",
                "Image is too large: {}.",
                "Unsupported image type for {}. Use PNG, JPG, GIF, or WebP.",
                "Could not process image {}.",
                "Image has no valid filename: {}.",
                "The system clipboard is unavailable.",
                "The clipboard does not contain an image.",
                "The clipboard does not contain a supported image.",
                "The clipboard image is too large.",
                "The clipboard image could not be processed.",
            ][..],
        ),
        (
            "crates/warp_tui/src/attachment_bar/view.rs",
            &[
                "TuiText::new(\"loading image",
                "AttachmentType::Image => \"[image]\"",
                "AttachmentType::File => \"[file]\"",
                "TuiText::new(\" \u{b7} loading",
            ][..],
        ),
        (
            "crates/warp_tui/src/tui_generic_tool_call_view.rs",
            &[
                "Is it OK if I read these files?",
                "Is it OK if I upload this artifact?",
                "Is it OK if I search this codebase?",
                "Is it OK if I search these files?",
                "Is it OK if I find files matching these patterns?",
                "Is it OK if I call this MCP tool?",
                "Is it OK if I read this MCP resource?",
                "Is it OK if I use the computer?",
                "Is it OK if I write this input to the running command?",
                "Should I start a new conversation?",
                "Is it OK if I hand control of the running command to you?",
                "Continue the agent's next step in a fresh conversation.",
                "  in {path}",
                "action.user_friendly_name()",
            ][..],
        ),
    ];

    let violations = selected_snippet_violations(&cases);

    assert!(
        violations.is_empty(),
        "TUI attachment and generic permission copy must use catalogs: {violations:#?}"
    );
}

#[test]
fn terminal_input_toasts_do_not_use_direct_english_literals() {
    let path = workspace_root().join("app/src/terminal/input.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    let removed_literals = [
        "Failed to prepare cloud handoff:",
        "File {display_path} already exists and will be overwritten",
        "Conversation exported to {display_path}",
        "Permission denied writing to {}. Check file permissions.",
        "Directory not found: {}",
        "File {} already exists",
        "Failed to export to {}: {}",
        "Cannot run `{truncated_command}` (command already running).",
        "was not attached",
        "files were not attached",
        "No agent harnesses are available. Contact your team admin.",
        "Preparing handoff",
        "Cannot send queries as a read-only viewer.",
        "Too many attachments for this conversation.",
    ];
    for literal in removed_literals {
        assert!(
            !content.contains(literal),
            "terminal input toast should use catalog copy instead of direct English literal: {literal}"
        );
    }

    let required_keys = [
        "terminal.input.cloud_handoff.prepare_failed",
        "terminal.input.conversation_export.overwrite_warning",
        "terminal.input.conversation_export.success",
        "terminal.input.conversation_export.error.permission_denied",
        "terminal.input.conversation_export.error.directory_not_found",
        "terminal.input.conversation_export.error.file_exists",
        "terminal.input.conversation_export.error.failed",
        "terminal.input.toast.attachment_skipped.plural",
        "terminal.input.toast.attachment_skipped.singular",
        "terminal.input.toast.command_already_running",
        "terminal.input.toast.no_agent_harnesses",
        "terminal.input.toast.preparing_handoff",
        "terminal.input.toast.read_only_viewer",
        "terminal.input.toast.too_many_attachments",
    ];
    for key in required_keys {
        assert!(
            content.contains(key),
            "terminal input toast should reference catalog key {key}"
        );
    }
}

#[test]
fn terminal_input_placeholders_use_catalog_copy() {
    let path = workspace_root().join("app/src/terminal/input.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    for literal in [
        "Tell the agent what to build...",
        "Kick off a cloud agent",
        "Run commands",
        "Search queries",
        "Search conversations",
        "Search skills",
        "Search models",
        "Search profiles",
        "Search commands",
        "Search prompts",
        "Search indexed repos",
        "Search plans",
        "Enter prompt for {}...",
        "Hand off to {}",
        "Handoff to cloud",
        "Type '#' for AI command suggestions",
    ] {
        assert!(
            !content.contains(literal),
            "terminal input placeholder should use catalog copy instead of direct English literal: {literal}"
        );
    }

    for key in [
        "terminal.input.hint.tell_agent_what_to_build",
        "terminal.input.hint.kick_off_cloud_agent",
        "terminal.input.hint.run_commands",
        "terminal.input.hint.enter_prompt_for_agent",
        "terminal.input.hint.handoff_to_cloud",
        "terminal.input.hint.handoff_to_environment",
        "terminal.input.hint.ai_command_search",
        "terminal.input.placeholder.search_queries",
        "terminal.input.placeholder.search_queries_to_rewind",
        "terminal.input.placeholder.search_conversations",
        "terminal.input.placeholder.search_skills",
        "terminal.input.placeholder.search_models",
        "terminal.input.placeholder.search_profiles",
        "terminal.input.placeholder.search_commands",
        "terminal.input.placeholder.search_prompts",
        "terminal.input.placeholder.search_indexed_repos",
        "terminal.input.placeholder.search_plans",
        "terminal.input.agent_hint.",
        "LocalizationUpdater::handle(ctx)",
    ] {
        assert!(
            content.contains(key),
            "terminal input placeholder should reference catalog key or refresh path {key}"
        );
    }
}

#[test]
fn settings_shared_tooltips_use_catalog_copy() {
    let settings_page_path = workspace_root().join("app/src/settings_view/settings_page.rs");
    let settings_page = fs::read_to_string(&settings_page_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", settings_page_path.display()));

    assert!(
        !settings_page.contains("Click to learn more in docs"),
        "settings info tooltip must come from the localization catalog"
    );
    assert!(
        !settings_page.contains("LocaleId::EnUs"),
        "shared settings tooltips must not force the English locale"
    );
    for required in [
        "settings.info.learn_more_tooltip",
        "visible local-only icon must provide localized tooltip",
    ] {
        assert!(
            settings_page.contains(required),
            "settings tooltip rendering must retain localization invariant: {required}"
        );
    }

    let features_path = workspace_root().join("app/src/settings_view/features_page.rs");
    let features = fs::read_to_string(&features_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", features_path.display()));
    let call_marker = "tab_key_span.add_child(render_local_only_icon";
    let call_start = features
        .find(call_marker)
        .expect("tab behavior local-only icon call should exist");
    let open_paren = call_start + call_marker.len();
    let close_paren = matching_paren_end(&features, open_paren)
        .expect("tab behavior local-only icon call should have balanced parentheses");
    let tab_icon_arguments = top_level_arguments(&features[open_paren + 1..close_paren]);
    let tooltip_argument = tab_icon_arguments
        .get(2)
        .expect("tab behavior local-only icon call should include a tooltip argument");
    assert!(
        tooltip_argument.contains("\"settings.local_only.tooltip\"")
            && !tooltip_argument.contains("None"),
        "direct local-only icon rendering must provide localized tooltip text"
    );
}

#[test]
fn source_field_scanner_accepts_unicode_whitespace_boundaries() {
    let content = "说明：\u{3000}key: \"value\"";

    assert_eq!(field_string_literal(content, "key"), Some("value"));
}

#[test]
fn workspace_toasts_do_not_use_direct_english_literals() {
    let path = workspace_root().join("app/src/workspace/view.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    let removed_literals = [
        "Failed to load conversation for forking.",
        "You {verb} synchronized inputs in all tabs.",
        "You {verb} synchronized inputs in this tab.",
        "Press {} to undo.",
        ".unwrap_or_else(|| \"Conversation\".to_string())",
        "Failed to determine home directory",
        "Failed to parse opencode.json: {e}",
        "Failed to read opencode.json: {e}",
        "opencode.json has unexpected structure (plugin is not an array)",
        "Failed to create config directory: {e}",
        "OpenCode plugin set to: {new_entry}",
        "Failed to write opencode.json: {e}",
        "Failed to serialize opencode.json: {e}",
        "let error_message = format!(\"Failed to create log bundle: {err}\")",
        "format!(\"Process sample saved to {output_path}\")",
        "\"Failed to sample process (check logs)\".to_string()",
    ];
    for literal in removed_literals {
        assert!(
            !content.contains(literal),
            "workspace toast should use catalog copy instead of direct English literal: {literal}"
        );
    }

    let required_keys = [
        "workspace.toast.failed_to_load_conversation_for_forking",
        "workspace.toast.press_to_undo",
        "workspace.toast.sync_all_inputs_disabled",
        "workspace.toast.sync_all_inputs_enabled",
        "workspace.toast.sync_tab_inputs_disabled",
        "workspace.toast.sync_tab_inputs_enabled",
        "workspace.conversation.fallback_title",
        "workspace.opencode.failed_create_config_dir",
        "workspace.opencode.failed_home_dir",
        "workspace.opencode.failed_parse_config",
        "workspace.opencode.failed_read_config",
        "workspace.opencode.failed_serialize_config",
        "workspace.opencode.failed_write_config",
        "workspace.opencode.plugin_set",
        "workspace.opencode.unexpected_plugin_structure",
        "workspace.toast.failed_to_create_log_bundle",
        "workspace.toast.failed_to_sample_process",
        "workspace.toast.process_sample_saved",
    ];
    for key in required_keys {
        assert!(
            content.contains(key),
            "workspace toast should reference catalog key {key}"
        );
    }
}

#[test]
fn bundled_skill_entrypoints_can_be_localized_by_metadata() {
    let path = workspace_root().join("app/src/ai/skills/skill_manager.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    assert!(
        content.contains("read_bundled_skills_for_locale"),
        "bundled skills should be loaded through a locale-aware reader"
    );
    assert!(
        content.contains("description_zh_CN"),
        "bundled skills should support localized description metadata"
    );
    assert!(
        content.contains("localized_bundled_skill_description"),
        "bundled skills should apply localized metadata while keeping original skill content"
    );
}

#[test]
fn bundled_and_channel_gated_skill_entrypoints_include_zh_cn_descriptions() {
    let roots = [
        workspace_root().join("resources/bundled/skills"),
        workspace_root()
            .join("resources")
            .join("bundled")
            .join("mcp_skills")
            .join("figma"),
        workspace_root()
            .join("resources")
            .join("channel-gated-skills"),
    ];
    let mut violations = Vec::new();

    for root in roots {
        collect_skill_entrypoint_description_violations(&root, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "skill entrypoints missing description_zh_CN: {violations:#?}"
    );
}

#[test]
fn bundled_skill_localized_descriptions_preserve_trigger_semantics() {
    let roots = [
        workspace_root().join("resources/bundled/skills"),
        workspace_root()
            .join("resources")
            .join("bundled")
            .join("mcp_skills")
            .join("figma"),
        workspace_root()
            .join("resources")
            .join("channel-gated-skills"),
    ];
    let mut violations = Vec::new();

    for root in roots {
        collect_skill_description_semantic_violations(&root, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "localized skill descriptions changed trigger semantics: {violations:#?}"
    );
}

#[test]
fn extracts_skill_front_matter_with_lf_or_crlf_line_endings() {
    let lf_content = "---\nname: sample\ndescription: Sample description\n---\n\n# Sample\n";
    let crlf_content = lf_content.replace('\n', "\r\n");

    assert_eq!(
        skill_front_matter(lf_content).as_deref(),
        Some("name: sample\ndescription: Sample description")
    );
    assert_eq!(
        skill_front_matter(&crlf_content).as_deref(),
        Some("name: sample\ndescription: Sample description")
    );
}

#[test]
fn bundled_create_skill_html_sets_runtime_document_language() {
    for (name, content) in [
        ("eval review", CREATE_SKILL_EVAL_REVIEW_HTML),
        ("eval viewer", CREATE_SKILL_EVAL_VIEWER_HTML),
    ] {
        assert!(
            content.contains("document.documentElement.lang"),
            "{name} should set the html lang attribute from the active language"
        );
        assert!(
            content.contains("zh-CN"),
            "{name} should use zh-CN for Simplified Chinese content"
        );
    }

    assert!(
        !CREATE_SKILL_EVAL_REVIEW_HTML
            .contains("document.title = `${t(\"title\")} - __SKILL_NAME_PLACEHOLDER__`;"),
        "eval review template should not insert raw skill names into JavaScript"
    );
    assert!(
        CREATE_SKILL_EVAL_REVIEW_HTML
            .contains("document.getElementById(\"skill-name\").textContent"),
        "eval review title should read the skill name from DOM text"
    );
}

#[test]
fn bundled_tab_config_templates_keep_locale_specific_comments_separate() {
    for (name, content) in [
        ("default worktree tab config", DEFAULT_WORKTREE_TAB_CONFIG),
        ("new tab config template", NEW_TAB_CONFIG_TEMPLATE),
    ] {
        assert!(
            !content.contains('你'),
            "{name} base template should keep English comments for default-locale users"
        );
        assert!(
            !content.contains("name_zh_CN"),
            "{name} base template should not include zh-CN-only display metadata"
        );
    }

    for (name, content) in [
        (
            "default worktree zh-CN tab config",
            DEFAULT_WORKTREE_TAB_CONFIG_ZH_CN,
        ),
        (
            "new zh-CN tab config template",
            NEW_TAB_CONFIG_TEMPLATE_ZH_CN,
        ),
    ] {
        assert!(
            content.contains('你'),
            "{name} should include Chinese user-facing comments"
        );
        assert!(
            content.contains("name_zh_CN"),
            "{name} should keep localized display name metadata"
        );
    }
}

#[test]
fn local_agent_task_sync_persists_canonical_status_messages() {
    assert!(
        LOCAL_AGENT_TASK_SYNC_MODEL_SOURCE.contains("map_conversation_status_to_canonical_english"),
        "task sync should persist canonical English status messages"
    );
    assert!(
        !LOCAL_AGENT_TASK_SYNC_MODEL_SOURCE.contains("map_conversation_status_for_locale"),
        "task sync should not expose locale-parametrized status mapping for shared task rows"
    );
}

#[test]
fn ambient_agent_sdk_localizes_canonical_task_status_messages() {
    let source_lines = AGENT_SDK_AMBIENT_SOURCE
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>();

    assert!(
        AGENT_SDK_AMBIENT_SOURCE.contains("localized_task_status_message_for_locale"),
        "ambient agent SDK output should localize canonical task status messages before rendering"
    );
    assert!(
        AGENT_SDK_AMBIENT_SOURCE
            .contains("ambient_task_status_message(locale, &status_msg.message)"),
        "ambient agent SDK output should pass task status messages through its localization helper"
    );
    assert!(
        !source_lines
            .windows(2)
            .any(|lines| { lines == ["&status_msg.message,", "MAX_LINE_WIDTH,"] }),
        "ambient agent SDK output should not render canonical task status messages directly"
    );
}

#[test]
fn ambient_agent_task_cancel_toasts_use_catalog_copy() {
    for literal in [
        "\"Task cancelled\".to_string()",
        "format!(\"Failed to cancel task: {e}\")",
    ] {
        assert!(
            !AMBIENT_AGENT_TASK_SOURCE.contains(literal),
            "ambient task cancellation toast should not use direct English copy: {literal}"
        );
    }

    for key in [
        "agent.task_status.cancel_failed",
        "agent.task_status.cancelled_toast",
    ] {
        assert!(
            AMBIENT_AGENT_TASK_SOURCE.contains(key),
            "ambient task cancellation toast should reference catalog key {key}"
        );
    }
}

#[test]
fn warp_drive_sorting_menu_uses_catalog_copy() {
    for literal in ["Last updated", "Last trashed", "A to Z", "Z to A", "Type"] {
        assert!(
            !DRIVE_SOURCE.contains(&format!("=> \"{literal}\"")),
            "Warp Drive sorting menu should not return direct English copy: {literal}"
        );
    }

    for key in [
        "drive.sort.a_to_z",
        "drive.sort.last_trashed",
        "drive.sort.last_updated",
        "drive.sort.type",
        "drive.sort.z_to_a",
    ] {
        assert!(
            DRIVE_SOURCE.contains(key),
            "Warp Drive sorting menu should reference catalog key {key}"
        );
    }
}

#[test]
fn warpify_banner_action_uses_catalog_copy() {
    assert!(
        !WARPIFY_BANNER_SOURCE.contains("\"Warpify subshell\""),
        "Warpify banner action should not use direct English copy"
    );
    assert!(
        WARPIFY_BANNER_SOURCE.contains("terminal.use_agent_footer.action.warpify_subshell"),
        "Warpify banner action should reference its catalog copy"
    );
}

#[test]
fn mcp_provider_section_labels_use_catalog_copy() {
    assert!(
        MCP_LIST_PAGE_SOURCE.contains("provider.display_name_for_app(app)"),
        "MCP provider section should resolve its provider label for the active locale"
    );
    assert!(
        !MCP_LIST_PAGE_SOURCE.contains("provider.display_name())"),
        "MCP provider section should not interpolate the canonical English provider label"
    );

    for key in [
        "settings.mcp.provider.claude",
        "settings.mcp.provider.codex",
        "settings.mcp.provider.other_agents",
        "settings.mcp.provider.warp",
    ] {
        assert!(
            MCP_SOURCE.contains(key),
            "MCP provider display helper should reference catalog key {key}"
        );
    }
}

#[test]
fn default_model_switch_prompts_use_catalog_copy() {
    for literal in [
        "You added your own {provider_name} API key",
        "You added the \\\"{}\\\" custom endpoint",
    ] {
        assert!(
            !AI_SETTINGS_PAGE_SOURCE.contains(literal),
            "default model switch prompt should not use direct English copy: {literal}"
        );
    }

    for key in [
        "settings.ai.set_default_model.custom_endpoint_description",
        "settings.ai.set_default_model.provider_key_description",
    ] {
        assert!(
            AI_SETTINGS_PAGE_SOURCE.contains(key),
            "default model switch prompt should reference catalog key {key}"
        );
    }
}

#[test]
fn mcp_deeplink_install_error_uses_catalog_copy() {
    assert!(
        !MCP_SERVERS_PAGE_SOURCE
            .contains("MCP server '{gallery_title}' cannot be installed from this link."),
        "MCP deeplink installation failure should not use direct English copy"
    );
    assert!(
        MCP_SERVERS_PAGE_SOURCE.contains("settings.mcp.page.error.cannot_install_from_link"),
        "MCP deeplink installation failure should reference its catalog copy"
    );
}

#[test]
fn slash_command_feedback_uses_catalog_copy() {
    for literal in [
        "requires AI to be enabled",
        "cannot start new conversation while terminal command is running",
        "Please provide a tab name after /rename-tab",
        "/rename-conversation requires an active conversation",
        "Please provide a color after /set-tab-color",
        "Unknown tab color",
        "Please describe the project you want to create after /create-new-project",
        "The /open-file command only works for files, not directories",
        "File not found:",
        "The /open-file command is not supported in this build",
        "No active conversation to export",
        "Conversation exported to clipboard",
        "Export conversation to file unsupported in web",
        "Session is already being shared",
        "Cannot show conversation cost:",
        "Nothing to hand off",
        "/fork requires an active conversation",
        "/continue-locally requires an active conversation",
        "/continue-locally is only available for cloud Oz conversations",
        "/fork-and-compact requires an active conversation",
        "/compact-and requires an active conversation",
        "/queue requires an active conversation",
        "/queue requires a prompt argument",
    ] {
        assert!(
            !SLASH_COMMANDS_SOURCE.contains(literal),
            "slash command feedback should not use direct English copy: {literal}"
        );
    }

    for key in [
        "terminal.slash.error.ai_required",
        "terminal.slash.new_conversation.command_running",
        "terminal.slash.rename_tab.name_required",
        "terminal.slash.rename_conversation.no_active",
        "terminal.slash.set_tab_color.required",
        "terminal.slash.set_tab_color.unknown",
        "terminal.slash.create_project.description_required",
        "terminal.slash.open_file.files_only",
        "terminal.slash.open_file.not_found",
        "terminal.slash.open_file.unsupported_build",
        "terminal.slash.export.no_active",
        "terminal.slash.export.copied",
        "terminal.slash.export.file_unsupported_web",
        "terminal.slash.remote_control.already_shared",
        "terminal.slash.cost.no_active",
        "terminal.slash.cost.empty",
        "terminal.slash.cost.in_progress",
        "terminal.slash.handoff.no_active",
        "terminal.slash.fork.no_active",
        "terminal.slash.continue_locally.no_active",
        "terminal.slash.continue_locally.cloud_only",
        "terminal.slash.fork_and_compact.no_active",
        "terminal.slash.compact_and.no_active",
        "terminal.slash.queue.no_active",
        "terminal.slash.queue.prompt_required",
    ] {
        assert!(
            SLASH_COMMANDS_SOURCE.contains(key),
            "slash command feedback should reference catalog key {key}"
        );
    }
}

#[test]
fn terminal_agent_view_load_error_uses_catalog_copy() {
    assert!(
        !TERMINAL_AGENT_VIEW_SOURCE
            .contains("format!(\"Failed to load conversation with id: {conversation_id}\")"),
        "terminal agent view load failure should not use direct English copy"
    );
    assert!(
        TERMINAL_AGENT_VIEW_SOURCE.contains("terminal.agent_view.error.load_conversation"),
        "terminal agent view load failure should reference its catalog copy"
    );
}

#[test]
fn terminal_agent_view_entry_errors_use_catalog_copy() {
    assert!(
        !TERMINAL_AGENT_VIEW_SOURCE.contains("show_error_toast(e.to_string(), ctx)"),
        "terminal agent view entry errors should not expose canonical English Display text"
    );
    assert!(
        TERMINAL_AGENT_VIEW_SOURCE.contains("show_error_toast(e.localized_message(ctx), ctx)"),
        "terminal agent view entry errors should resolve copy for the active locale"
    );

    for key in [
        "terminal.agent_view.error.already_in_agent_view",
        "terminal.agent_view.error.command_running",
    ] {
        assert!(
            AGENT_VIEW_CONTROLLER_SOURCE.contains(key),
            "agent view entry error should reference catalog key {key}"
        );
    }
}

#[test]
fn drive_import_errors_are_structured_and_localized_at_render_time() {
    for literal in [
        "Failed to parse file: {e}",
        "Failed to upload file to server",
        "Failed to upload folder to server",
        "DismissibleToast::error(format!(\"{err}\"))",
    ] {
        assert!(
            !DRIVE_IMPORT_NODES_SOURCE.contains(literal)
                && !DRIVE_IMPORT_MODAL_SOURCE.contains(literal),
            "Drive import state should not store direct English error copy: {literal}"
        );
    }

    for key in [
        "drive.import.error.failed_parse_file",
        "drive.import.error.failed_upload_file",
        "drive.import.error.failed_upload_folder",
        "drive.import.error.file_picker",
    ] {
        assert!(
            DRIVE_IMPORT_NODES_SOURCE.contains(key) || DRIVE_IMPORT_MODAL_SOURCE.contains(key),
            "Drive import rendering should reference catalog key {key}"
        );
    }
    assert!(
        DRIVE_IMPORT_NODES_SOURCE.contains("error.localized_message(context.app)"),
        "Drive import errors should resolve copy during rendering for runtime locale changes"
    );
}

#[test]
fn drive_export_file_picker_errors_use_catalog_copy() {
    assert!(
        !DRIVE_EXPORT_SOURCE.contains("DismissibleToast::error(format!(\"{err}\"))"),
        "Drive export file picker errors should not bypass localization"
    );
    assert!(
        DRIVE_EXPORT_SOURCE.contains("drive.export.error.file_picker"),
        "Drive export file picker errors should retain localized context"
    );
}

#[test]
fn agent_assisted_environment_picker_errors_are_structured_and_localized() {
    for literal in [
        "FilePickerError::DialogFailed(\"No directory selected\".to_string())",
        "DismissibleToast::error(format!(\"{error}\"))",
    ] {
        assert!(
            !AGENT_ASSISTED_ENVIRONMENT_SOURCE.contains(literal),
            "agent environment picker errors should not persist direct English copy: {literal}"
        );
    }
    for expected in [
        "DirectoryPickerError::NoSelection",
        "DirectoryPickerError::Platform",
        "settings.environment.agent_assisted.error.no_directory_selected",
        "settings.environment.agent_assisted.error.file_picker",
    ] {
        assert!(
            AGENT_ASSISTED_ENVIRONMENT_SOURCE.contains(expected),
            "agent environment picker errors should preserve a localized category: {expected}"
        );
    }
}

#[test]
fn remaining_file_picker_toasts_use_shared_localized_context() {
    for relative_path in [
        "app/src/workspace/view.rs",
        "app/src/pane_group/pane/get_started_view.rs",
        "app/src/editor/view/mod.rs",
        "app/src/settings_view/code_page.rs",
        "app/src/themes/theme_creator_modal.rs",
        "app/src/ai/facts/view/rule.rs",
        "app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs",
    ] {
        let source = std::fs::read_to_string(workspace_root().join(relative_path))
            .expect("file picker UI source should be readable");
        assert!(
            !source.contains("DismissibleToast::error(format!(\"{err}\"))"),
            "file picker UI should not display an unlocalized error in {relative_path}"
        );
        assert!(
            source.contains("file_picker_error_for_app"),
            "file picker UI should add localized context in {relative_path}"
        );
    }
}

#[test]
fn platform_api_key_load_errors_use_catalog_copy() {
    let source =
        std::fs::read_to_string(workspace_root().join("app/src/settings_view/platform_page.rs"))
            .expect("platform settings source should be readable");
    assert!(
        !source.contains("DismissibleToast::error(format!(\"{err}\"))"),
        "platform API key load errors should not bypass localization"
    );
    assert!(
        source.contains("settings.platform.api_keys.error.load_failed"),
        "platform API key load errors should retain localized context"
    );
}

#[test]
fn labeled_error_toasts_use_localized_separators() {
    for (relative_path, direct_format) in [
        ("app/src/settings_view/ai_page.rs", "\"{}: {err}\""),
        (
            "app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs",
            "format!(\"{error_label}: {message}\")",
        ),
    ] {
        let source = std::fs::read_to_string(workspace_root().join(relative_path))
            .expect("labeled error source should be readable");
        assert!(
            !source.contains(direct_format),
            "labeled errors should not hard-code punctuation in {relative_path}"
        );
        assert!(
            source.contains("labeled_error_for_app"),
            "labeled errors should resolve punctuation from the catalog in {relative_path}"
        );
    }
}

#[test]
fn inline_web_search_failure_titles_use_catalog_copy() {
    for literal in [
        "\"Web search failed\".to_string()",
        "format!(\"Web search failed for",
    ] {
        assert!(
            !INLINE_WEB_SEARCH_SOURCE.contains(literal),
            "inline web search failure titles should not use direct English copy: {literal}"
        );
    }

    for key in ["agent.web_search.failed", "agent.web_search.failed_query"] {
        assert!(
            INLINE_WEB_SEARCH_SOURCE.contains(key),
            "inline web search failure rendering should reference catalog key {key}"
        );
    }
}

#[test]
fn streamed_mcp_tool_titles_are_structured_and_localized_at_render_time() {
    for literal in [
        "format!(\"MCP Tool: {name}\")",
        "format!(\"MCP Tool: {name} (",
    ] {
        assert!(
            !AI_BLOCK_SOURCE.contains(literal),
            "MCP stream handling should not persist direct English title copy: {literal}"
        );
    }

    for expected in [
        "pub name: String",
        "view.update_mcp_request(name.to_string(), mcp_args)",
        "agent.requested_command.mcp_tool.label",
        "agent.requested_command.mcp_tool.label_with_input",
        "self.display_text(app)",
    ] {
        assert!(
            REQUESTED_COMMAND_SOURCE.contains(expected) || AI_BLOCK_SOURCE.contains(expected),
            "MCP tool title path should preserve structured data and localize during rendering: {expected}"
        );
    }
}

#[test]
fn suggested_new_conversation_buttons_resolve_catalog_copy_during_rendering() {
    for literal in [
        "\"Start a new conversation\".to_owned()",
        "\"Continue current conversation\".to_owned()",
    ] {
        assert!(
            !AI_BLOCK_SOURCE.contains(literal),
            "suggested conversation buttons should not store direct English copy: {literal}"
        );
    }
    for expected in [
        "agent.output.new_conversation.start",
        "agent.output.new_conversation.continue_current",
        "localization::text_for_app(app, accept_text_key)",
        "localization::text_for_app(app, reject_text_key)",
        "buttons.update(ctx, |_, ctx| ctx.notify())",
    ] {
        assert!(
            AI_BLOCK_SOURCE.contains(expected),
            "suggested conversation buttons should refresh localized copy: {expected}"
        );
    }
}

#[test]
fn custom_endpoint_model_descriptions_are_localized_at_display_boundaries() {
    assert!(
        !LLMS_SOURCE.contains("Custom \u{00b7}"),
        "custom endpoint model metadata should not persist English decorated copy"
    );
    assert!(
        LLMS_SOURCE.contains("description: Some(endpoint.name.clone())"),
        "custom endpoint model metadata should retain the endpoint name"
    );
    for source in [MODEL_DATA_SOURCE, PROFILE_MODEL_SELECTOR_SOURCE] {
        assert!(
            source.contains("model_description_for_app"),
            "custom endpoint descriptions should be localized at each display boundary"
        );
    }
}

#[test]
fn onboarding_ui_calls_do_not_use_direct_english_literals() {
    let onboarding_src = workspace_root().join("crates/onboarding/src");
    let mut violations = Vec::new();
    collect_direct_ui_literal_violations_with_patterns(
        &onboarding_src,
        UI_LITERAL_PATTERNS,
        &mut violations,
    );
    collect_direct_ui_literal_violations_with_patterns(
        &onboarding_src,
        ONBOARDING_UI_LITERAL_PATTERNS,
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "direct user-visible English literals in onboarding UI calls: {violations:#?}"
    );
}

#[test]
fn onboarding_callout_direct_english_literals_are_localized() {
    let cases = [
        (
            "crates/onboarding/src/callout/view.rs",
            &["title:", "text:", "label:"][..],
        ),
        (
            "crates/onboarding/src/callout/model.rs",
            &[
                "OnboardingQuery::AgentPrompt(",
                "OnboardingQuery::TerminalCommand(",
            ][..],
        ),
    ];
    let mut violations = Vec::new();

    for (relative_path, patterns) in cases {
        let path = workspace_root().join(relative_path);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        collect_direct_literal_after_patterns(relative_path, &content, patterns, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "onboarding callout user-visible English literals must use catalog copy: {violations:#?}"
    );
}

#[test]
fn ui_components_calls_do_not_use_direct_english_literals() {
    let ui_components_src = workspace_root().join("crates/ui_components/src");
    let mut violations = Vec::new();
    collect_direct_ui_literal_violations(&ui_components_src, &mut violations);

    assert!(
        violations.is_empty(),
        "direct user-visible English literals in ui_components calls: {violations:#?}"
    );
}

#[test]
fn context_chip_disabled_tooltips_do_not_use_direct_english_literals() {
    let path = workspace_root().join("app/src/context_chips/context_chip.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let violations = [
        "fn tooltip_text(&self)",
        "fn tooltip_override_text(&self)",
        "Requires a local session",
        "Requires the GitHub CLI",
        "Requires the `",
    ]
    .into_iter()
    .filter(|snippet| content.contains(snippet))
    .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "context chip disabled tooltips must use AppContext localization: {violations:#?}"
    );
}

#[test]
fn app_menu_custom_items_do_not_use_direct_english_literals() {
    let path = workspace_root().join("app/src/app_menus.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let mut violations = Vec::new();

    for pattern in ["CustomMenuItem::new(", "CustomMenuItem::new_with_submenu("] {
        collect_direct_first_argument_literal_violations(
            "app/src/app_menus.rs",
            &content,
            pattern,
            &mut violations,
            None,
        );
    }

    assert!(
        violations.is_empty(),
        "app menu custom items must use AppContext localization for titles: {violations:#?}"
    );
}

#[test]
fn selected_misc_ui_surfaces_do_not_use_direct_english_literals() {
    let cases = [
        (
            "app/src/auth/paste_auth_token_modal.rs",
            &["Label(\"Cancel\".into())", "Label(\"Continue\".into())"][..],
        ),
        (
            "app/src/auth/login_slide.rs",
            &[
                "Get started with Warp Drive",
                "Get started with AI",
                "Create an account",
                "Privacy Settings\".into()",
                "Label(\"Back\".into())",
                "Label(\"Continue\".into())",
                "Label(\"Skip for now\".into())",
                "Continue without signing in?",
                "Click here to paste your token from the browser\".into()",
            ][..],
        ),
        (
            "app/src/auth/auth_view_body.rs",
            &[
                "const AUTH_TOKEN_INPUT_PLACEHOLDER_TEXT",
                "Click here to paste your token from the browser\".into()",
                "In order to use Warp’s AI features",
                "In order to create more objects in Warp Drive",
                "In order to share, please create an account",
            ][..],
        ),
        (
            "app/src/auth/mod.rs",
            &[
                "You have {num_long_running_commands}",
                "You have {num_shared_sessions}",
                "You have {num_unsaved_objects}",
                "You have {num_unsaved_files}",
            ][..],
        ),
        (
            "app/src/settings_view/features_page.rs",
            &[
                "Category::new(\"General\"",
                "Category::new(\"Session\"",
                "Category::new(\"Keys\"",
                "wrappable_text(\"Warp is the default terminal\"",
                "\"Make Warp the default terminal\".to_string()",
                "wrappable_text(\"Allowed Values: 1-20\"",
                "wrappable_text(\"Changes will apply to new windows.\"",
                "Enabling this setting disables global hotkey support.",
                "secondary_text.push_str(\"\\n\\nRestart Warp",
                "let label = \"Start Warp at login",
                "val.display_name(),",
                "DefaultSessionMode::Terminal.display_name()",
                "other.display_name().to_string()",
                "Use an improved implementation of find to keep the UI responsive",
            ][..],
        ),
        (
            "app/src/settings_view/appearance_page.rs",
            &[
                "Category::new(\"Window\"",
                "Category::new(\"Tools panel\"",
                "Category::new(\"Input\"",
                "\"Create your own custom theme\".to_string()",
                "\"Use Window Blur (Acrylic texture)\".to_string()",
                "The selected hardware may not support rendering transparent windows.",
                "The selected graphics settings may not support rendering transparent windows.",
                "When enabled, reopening or restoring a window opens the vertical tabs panel",
                "Show the latest user prompt instead of the generated conversation title",
                "Some(\"Adjusts the default zoom level across all windows\")",
                "RadioButtonItem::text(\"Shell (PS1)\")",
            ][..],
        ),
        (
            "app/src/settings_view/main_page.rs",
            &["\"Contact support\".into()"][..],
        ),
        (
            "app/src/settings/mod.rs",
            &[
                "\"Your settings file contains an error.\".to_owned()",
                "\"Your settings file contains errors.\".to_owned()",
                "format!(\"{self}. Open the file to fix it.\")",
                "format!(\"{self}. The default value is being used.\")",
                "format!(\"{self}. Default values are being used.\")",
            ][..],
        ),
        (
            "app/src/settings_view/settings_file_footer.rs",
            &["\"Open file\",", "\"Fix with Oz\","][..],
        ),
        (
            "app/src/drive/import/modal_body.rs",
            &["\"Learn about file support and formatting\".to_string()"][..],
        ),
        (
            "app/src/drive/export.rs",
            &[
                "format!(\"Failed to export {name}\")",
                "\"Export failed\".to_string()",
            ][..],
        ),
        (
            "app/src/billing/shared_objects_creation_denied_modal.rs",
            &[
                "format!(\"Shared {object_type}s restricted\")",
                "format!(\"Shared {object_type}s limit reached\")",
            ][..],
        ),
        (
            "app/src/terminal/view/ssh_remote_server_choice_view.rs",
            &["\"Manage Warpify settings\".into()"][..],
        ),
        (
            "app/src/tab_configs/session_config_modal.rs",
            &[
                "\"Create your first tab config\"",
                "Set up a reusable starting point for your tabs.",
            ][..],
        ),
        (
            "app/src/context_chips/display_menu.rs",
            &[
                "Search directories...",
                "Search branches...",
                "Search environments...",
            ][..],
        ),
        (
            "app/src/view_components/markdown_toggle_view.rs",
            &["\"Rendered\".into()", "\"Raw\".into()"][..],
        ),
        (
            "app/src/util/tooltips.rs",
            &["*Secrets are not sent to Warp's server."][..],
        ),
        (
            "app/src/terminal/view/init_environment/mod.rs",
            &["Environment setup cancelled"][..],
        ),
        (
            "app/src/notebooks/notebook/details_bar.rs",
            &["{editor} is editing"][..],
        ),
        (
            "app/src/notebooks/file/mod.rs",
            &["Could not read {}", "Command from {}"][..],
        ),
        ("app/src/notebooks/notebook.rs", &["Command from {}"][..]),
        (
            "app/src/root_view.rs",
            &["pane_group.set_title(\"Create Environment\""][..],
        ),
        (
            "app/src/ai/agent_management/details_action_buttons.rs",
            &[
                "Open conversation",
                "Cancel task",
                "Fork conversation",
                "View details",
                "Copy link to run",
            ][..],
        ),
        (
            "app/src/code/file_tree/view/render.rs",
            &["String::from(\"File\")", "String::from(\"Folder\")"][..],
        ),
        (
            "app/src/remote_server/codebase_index_model.rs",
            &[
                "The remote codebase index is missing its root hash.",
                "Remote codebase search is not available.",
            ][..],
        ),
        (
            "app/src/ai/agent_management/cloud_setup_guide_view.rs",
            &[
                "Workflow::new(\"Create Environment\"",
                "Workflow::new(\"Create Environment (CLI)\"",
                "Workflow::new(\"Create Slack Integration\"",
                "Workflow::new(\"Create Linear Integration\"",
                "GitHub link or local filepath to the repository",
                "Name for the environment",
                "Docker image to use for the environment",
                "ID of the environment to integrate with",
            ][..],
        ),
        (
            "app/src/workspace/view.rs",
            &[
                "Command from Warp AI",
                "Command from Oz",
                "\"Notifications\".to_string()",
                "Failed to load tab config {friendly_path}",
                "Failed to load model config {friendly_path}",
                "ToastLink::new(\"Open file\"",
                "ToastLink::new(\"View changelog\"",
                "String::from(\"Warp updated!\")",
                "ToastLink::new(\"View\"",
                "ToastLink::new(\"Undo\"",
                "Check out the latest version and try again.",
                "const UPDATE_READY_TEXT",
                "const VERSION_DEPRECATION_BANNER_TEXT",
                "const VERSION_DEPRECATION_WITHOUT_PERMISSIONS_BANNER_TEXT",
                "\"Untitled pane\".to_string()",
                "\"New Group\".to_string()",
                "Some(\"Search repos\".to_string())",
                "Text::new_inline(\" + Add new repo\"",
                "Some(\"Install Update\".to_owned())",
                "let title = \"New API key\".to_string()",
                "\"Code review panel\".to_string()",
                "text: \"Fix with Oz\"",
                "text: \"Open file\"",
                "heading: Some(\"Your login has expired.\"",
                "text: \"Sign in\"",
                "text: \"Update Warp manually\"",
                "text: \"Update now\"",
                "text: \"Restart app and update now\"",
                "text: \"More info\"",
            ][..],
        ),
        (
            "app/src/workspace/view/cloud_agent_capacity_modal/mod.rs",
            &[
                "Concurrent cloud agent limit reached",
                "This cloud run is queued because your team has reached the maximum number of concurrent cloud agents.",
                "You're out of AI credits",
                "This cloud run stopped because your team has used all available AI credits for the current billing period.",
                "Upgrade your plan for more concurrent cloud agents.",
                "Upgrade your plan to continue running cloud agents.",
                "Paid plans start at ${price}/month and include everything in your free trial plus:",
                "Paid plans include everything in your free trial plus:",
                "The Business plan starts at ${price}/month and includes everything on your current plan plus:",
                "The Business plan includes everything on your current plan plus:",
                "AI credits per month",
                "Extended AI credits per month",
                "the number of concurrent cloud agents",
                "Bring your own API key",
                "Upgrade plan",
                "Open billing",
            ][..],
        ),
        (
            "app/src/ai/agent_management/notifications/view.rs",
            &["Notifications\".to_string()", "No notifications"][..],
        ),
        (
            "app/src/ai/agent_management/notifications/toast_stack.rs",
            &["Open conversation"][..],
        ),
        (
            "app/src/ai/blocklist/block.rs",
            &[
                "String::from(\"Copied to clipboard\")",
                "String::from(\"Thank you for the feedback!\")",
            ][..],
        ),
        (
            "app/src/ai/blocklist/block/cli.rs",
            &[
                "\"Allow\".to_string()",
                "\"Refine\".to_string()",
                "\"Take over\".to_string()",
                "\"Take control\".to_string()",
            ][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/ask_user_question_view.rs",
            &["\"Skip all\".to_string()", "\"Next\".to_string()"][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/suggested_unit_tests.rs",
            &["Don't show me suggested code banners again"][..],
        ),
        (
            "app/src/drive/index.rs",
            &[
                "Items in the trash will be deleted forever after 30 days.",
                "Drag or move a personal workflow or notebook here to share it with your team.",
                "You've run out of {object_type}s on your plan.",
                "Upgrade for access to more notebooks, workflows, shared sessions, and AI credits.",
                "Shared objects have been restricted due to a subscription payment issue.",
                "Please update your payment information to restore access.",
                "Please contact support@warp.dev to restore access.",
                "Please contact a team admin to restore access.",
                "Sort by",
                "Retry sync",
                "Create team",
                "\"TRASH\"",
                "\"Notebooks\"",
                "\"Workflows\"",
                "\"Environment Variables\"",
                "\"Folders\"",
                "\"Agent Workflows\"",
                "\"Rules\"",
                "\"MCP Server\"",
                "\"MCP Servers\"",
            ][..],
        ),
        (
            "app/src/drive/sharing/dialog/mod.rs",
            &["Live session started at {} on {}", "Unknown"][..],
        ),
        (
            "app/src/settings_view/appearance_page.rs",
            &[
                "Automatically switch between light and dark themes when your system does.",
                "You may need to restart Warp for MacOS to apply the preferred icon style.",
                "\"Line height\".to_string()",
                "\"Font weight\".to_string()",
                "\"Font size (px)\".to_string()",
            ][..],
        ),
        (
            "app/src/settings_view/features_page.rs",
            &["\"Characters considered part of a word\".to_string()"][..],
        ),
        (
            "app/src/notebooks/editor/view.rs",
            &[
                "new_without_help(\"Shift-tab\"",
                "new_without_help(\"Edit Link\"",
                "new_without_help(\"Copy Link\"",
                "format!(\"Open link: {}\"",
                "new_without_help(\"Delete line left\"",
                "new_without_help(\"Delete line right\"",
                "new_without_help(\"Delete word left\"",
                "new_without_help(\"Delete word right\"",
                "new_without_help(\"Cut line left\"",
                "new_without_help(\"Cut line right\"",
                "new_without_help(\"Cut word left\"",
                "new_without_help(\"Cut word right\"",
                "new_without_help(\"Show character palette\"",
                "new_without_help(\"Show find bar\"",
                "new_without_help(\"Open block-insertion menu\"",
                "new_without_help(\"Open embedded object search menu\"",
                "format!(\"Insert {} block\"",
                "\"De-select command\"",
                "\"Switch from selecting commands to selecting text\"",
                "format!(\"Change code block language to {code_block_type}\"",
                "new_without_help(\"Copy code block\"",
                "new_without_help(\"Toggle task list\"",
            ][..],
        ),
        (
            "app/src/notebooks/editor/omnibar.rs",
            &[
                "block_type.label()",
                "format!(\"Convert to {}\"",
                "\"Remove link\"",
            ][..],
        ),
        (
            "app/src/terminal/view/use_agent_footer/mod.rs",
            &["Don't show again"][..],
        ),
        (
            "app/src/ai/blocklist/block/status_bar.rs",
            &[
                "Cloud agent run cancelled",
                "The primary model failed. Retrying with the fallback model.",
                "The primary model ({primary}) failed. Retrying with the fallback model.",
                "Missing GitHub authentication.",
                "Warping with {name}.",
                "Warping with another model.",
                "Setting up environment",
                "\"Exit\"",
                "\"Exit agent input\"",
            ][..],
        ),
        (
            "app/src/ai/blocklist/block/cli.rs",
            &["This response won't count towards your usage."][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs",
            &["let key = toggle_key.display_name();"][..],
        ),
        (
            "app/src/ai/blocklist/block/view_impl/output.rs",
            &[
                "Grant access to upload this artifact?",
                "Conversation summarized",
                "Always allow file access for coding tasks",
                "Always allow file access for this repo",
                "Thought for {}",
                "\"Thinking\".to_string()",
                "Failed to read files",
                "Sorry you had a bad experience with this interaction. We've refunded you 1 credit.",
                "Sorry you had a bad experience with this interaction. We've refunded you {request_refunded_count} credits.",
                "Manage AI Autonomy permissions",
                "Search in {}",
                "Search in {} failed because the codebase isn't indexed",
                "Search in {} failed",
                "Search in {} cancelled",
                "No relevant files found.",
                "Stopped task {}/{}",
                "Stopped task: \\\"{task_name}\\\"",
                "\"Stopped task\".to_string()",
                "New conversation started",
                "Continuing current conversation",
                "New conversation suggestion cancelled",
                "It seems like the topic changed. Would you like to make a new conversation?",
                "Upload artifact: {}",
                "Description: {description}",
                "Status: uploaded artifact {artifact_uid}",
                "Uploaded file: {filepath}",
                "Status: upload failed: {error}",
                "Open skill",
                "OK if I read this MCP resource?",
                "OK if I use computer control for this task?",
                "\"Listing messages\".to_string()",
                "\"Grepping for patterns\".to_string()",
                "Grepping for patterns: {joined}",
                "Reading {count} messages",
                "This suggestion is being edited in another tab.",
                "Comment addressed: \\\"",
                "Could not apply changes to file.",
                "Text::new_inline(\"References\"",
                "FormattedTextFragment::plain_text(\"Suggestions:\")",
                "Text::new_inline(\"Debug output\"",
                "FormattedTextFragment::plain_text(\"Searched \"",
                "\"Searched\"",
                "\"Searching\"",
                "\"conversation\"",
                "\"agent run\"",
                "Searching in {}",
                "Searching in {",
                "FormattedTextFragment::plain_text(\"this conversation\")",
            ][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/orchestration_controls.rs",
            &[
                "LocalHarnessSetupState::MissingHarness { tooltip } => tooltip.to_string()",
                "LocalHarnessSetupState::ProductDisabled { message } => {\n                        message.to_string()",
                "return Some(tooltip.to_string())",
                "return Some(message.to_string())",
            ][..],
        ),
        (
            "app/src/code_review/code_review_view.rs",
            &["Text::new(\n                    reason.to_string()"][..],
        ),
        (
            "app/src/code_review/code_review_header/mod.rs",
            &["unwrap_or(\"Reviewing open changes\".to_string())"][..],
        ),
        (
            "app/src/resource_center/view.rs",
            &[
                "ResourceCenterFooterItem::Docs => \"Docs\"",
                "ResourceCenterFooterItem::Slack => \"Join our Slack community\"",
                "ResourceCenterFooterItem::Feedback => \"Feedback\"",
                "Some(ResourceCenterPage::Keybindings) => \"Keyboard Shortcuts\".to_string()",
                "\"Warp Essentials\".to_string()",
            ][..],
        ),
        (
            "app/src/search/search_bar.rs",
            &[
                "&[(\"filter\", loading_filter.display_name())]",
                "editor.set_placeholder_text(filter.placeholder_text(), ctx)",
            ][..],
        ),
        (
            "app/src/search/filter_chip_renderer.rs",
            &["Text::new_inline(self.display_name()"],
        ),
        (
            "app/src/search/command_palette/filter_chip_renderer.rs",
            &["self.display_name(),"],
        ),
        (
            "app/src/search/command_palette/conversations/data_source.rs",
            &[
                "ConversationSection::ActivePane => \"Active pane conversations\"",
                "ConversationSection::OtherActive => \"Other active conversations\"",
                "ConversationSection::Past => \"Past conversations\"",
            ][..],
        ),
        (
            "app/src/settings_view/billing_and_usage/billing_cycle_usage_rows.rs",
            &["let label = filter.label();"][..],
        ),
        (
            "app/src/settings_view/ai_page.rs",
            &[
                "DropdownItem::new(\n                    mode.display_name()",
                "DropdownItem::new(\n                        mode.display_name()",
                "DropdownItem::new(\n                            val.display_name()",
                ".voice_input_toggle_key\n                        .value()\n                        .display_name()",
            ][..],
        ),
        (
            "app/src/ai/agent_management/notifications/view.rs",
            &[
                "filter.label().to_string()",
                "format!(\"{} ({count})\", filter.label())",
            ][..],
        ),
        (
            "app/src/terminal/view.rs",
            &[
                "A terminal program tried to access your clipboard.",
                "\"Allow\".to_string()",
                "\"Don't show again\".to_string()",
                "A terminal program tried to write to your clipboard.",
                "A terminal program tried to read your clipboard.",
                "Allow clipboard writes",
                "Allow clipboard reads and writes",
                "Jump to the bottom of this block",
                "Can not invoke environment variable subshell in a non-local session",
                "Bundled skills cannot be edited",
                "Editing skills is not supported in this build",
            ][..],
        ),
        (
            "app/src/context_chips/display_chip.rs",
            &[
                "format!(\"Tracking {upstream}",
                "\"Branch was rebased; upstream name is unavailable\"",
                "\"No upstream configured\"",
                "\".. (Parent Directory)\".to_string()",
                "\"Monthly AI credits reset!\".to_string()",
            ][..],
        ),
        (
            "app/src/context_chips/display_menu.rs",
            &["\"(none)\".to_string()"],
        ),
        (
            "app/src/pane_group/pane/view/header/mod.rs",
            &["\"Open files and review code diffs\".to_string()"],
        ),
        (
            "app/src/drive/items/item.rs",
            &[
                "let mut owner_label = \"From \".to_string()",
                "owner_label.push_str(\"unknown user\")",
                "map_or(\"unknown team\"",
            ][..],
        ),
        (
            "app/src/quit_warning/mod.rs",
            &[
                "You have {} {} running",
                "You are sharing {} {}",
                "Do you want to save the changes you made to",
                "You have unsaved file changes",
                "\"Yes, close\"",
                "\"Yes, quit\"",
                "\"Show running processes\"",
                "\"Close pane?\"",
                "\"Close tab?\"",
                "\"Close tabs?\"",
                "\"Close window?\"",
                "\"Quit Warp?\"",
                "\"Save changes?\"",
            ][..],
        ),
        (
            "app/src/editor/view/voice.rs",
            &[
                "[(\"key\", toggle_key.display_name())]",
                "modifier_key.display_name().to_lowercase()",
            ][..],
        ),
        (
            "app/src/terminal/view/queued_prompts_panel.rs",
            &["format!(\"{count} queued\")", "Text::new(\"to send\""][..],
        ),
        (
            "app/src/workspace/view.rs",
            &[
                "Access your tab configs here.",
                "Continue this local Warp Agent task in the cloud from the current conversation state.",
                "\"New worktree config\" =>",
                "\"New tab config\" =>",
                "Ask Warp AI to explain errors, suggest commands or write scripts.",
                "Agent management panel",
                "Tabs panel",
                "Project explorer",
                "Global search",
                "Agent conversations",
                "Tools panel",
                "Warp Essentials",
                "Some(\"Introducing Oz\".to_string())",
                "Open: {}",
                "Failed to open this handoff in Cloud Mode.",
                "Tooltip::text(\"Code review panel\"",
                "Search sessions, agents, files...",
                "New Tab",
                "Tab configs",
                "Some features may be unavailable offline",
                "set_fallback_display_title(\"Linear Issue\".to_string())",
            ][..],
        ),
        (
            "app/src/terminal/view.rs",
            &[
                "set_fallback_display_title(\"Project setup\".to_string())",
                "format!(\"Started at: {}\"",
                "format!(\"\\nCompleted at: {}\"",
            ],
        ),
        (
            "app/src/workspace/view/global_search/view.rs",
            &["The result set only contains a subset of all matches."][..],
        ),
        (
            "app/src/terminal/input/models/data_source.rs",
            &["Model: {}", "(selected)", "(disabled)"][..],
        ),
        (
            "app/src/terminal/input/profiles/search_item.rs",
            &["Profile: {profile_name}", "\"(selected)\""][..],
        ),
        (
            "app/src/terminal/input/skills/data_source.rs",
            &["format!(\"Skill: {}\""][..],
        ),
    ];

    let mut violations = Vec::new();
    violations.extend(selected_snippet_violations(&cases));

    assert!(
        violations.is_empty(),
        "selected UI surfaces must use AppContext localization: {violations:#?}"
    );
}

#[test]
fn settings_render_helpers_do_not_use_direct_english_literals() {
    let mut violations = Vec::new();
    collect_direct_literal_after_patterns_in_dir(
        &workspace_root().join("app/src/settings_view"),
        &["render_body_item::<", "render_dropdown_item("],
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "settings rendering helpers must receive catalog-backed copy: {violations:#?}"
    );
}

#[test]
fn conversation_search_prefix_catalog_copy_preserves_spacing() {
    let prefix_keys = [
        "agent.output.conversation_search.searched_prefix",
        "agent.output.conversation_search.searching_prefix",
        "agent.output.conversation_search.conversation_prefix",
        "agent.output.conversation_search.agent_run_prefix",
    ];

    for (locale, catalog) in [
        ("en-US", bundled_en_us_map()),
        ("zh-CN", bundled_zh_cn_map()),
    ] {
        for key in prefix_keys {
            let value = catalog
                .get(key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("missing bundled {locale} key {key}"));
            assert!(
                value.ends_with(' '),
                "{locale}:{key} must keep a trailing space because output.rs appends the next formatted fragment directly"
            );
        }
    }
}

#[test]
fn current_i18n_multiline_ui_calls_do_not_use_direct_english_literals() {
    let cases = [
        (
            "app/src/view_components/feature_popup.rs",
            &["Text::new("][..],
        ),
        (
            "app/src/ai/execution_profiles/mod.rs",
            &["FormattedTextFragment::plain_text("][..],
        ),
        (
            "app/src/terminal/input/slash_commands/search_item.rs",
            &["Text::new("][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/requested_command.rs",
            &["render_autonomy_checkbox_setting_speedbump_footer("][..],
        ),
        (
            "app/src/ai/blocklist/block/view_impl/common.rs",
            &["render_visual_card("][..],
        ),
        (
            "app/src/settings/import/view.rs",
            &["Text::new_inline("][..],
        ),
        (
            "app/src/settings_view/execution_profile_view.rs",
            &["render_run_agents_permission_line_with_icon("][..],
        ),
        ("app/src/drive/items/item.rs", &["Span::new("][..]),
    ];

    let mut violations = Vec::new();
    for (relative_path, patterns) in cases {
        let path = workspace_root().join(relative_path);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        collect_direct_literal_after_patterns(relative_path, &content, patterns, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "current multiline UI calls must use catalog copy: {violations:#?}"
    );
}

#[test]
fn execution_profile_and_model_selector_ui_uses_catalog_copy() {
    let cases = [
        (
            "app/src/terminal/profile_model_selector.rs",
            &[
                "Choose an AI execution profile",
                "Choose an agent model",
                "Follow-ups use the original run's model",
                "Request edit access to change model",
                "auto-select the best model for the task",
                "New models available",
                "\"Intelligence\".to_string()",
                "\"Speed\".to_string()",
                "\"Cost\".to_string()",
                "\"Model Specs\".to_string()",
                "Warp\u{2019}s benchmarks for how well a model performs",
                "\"Auto mode\"",
                "Auto will select the best model for the task",
                "\"Reasoning level\"",
                "Increased reasoning levels consume more credits",
                "active_profile.data().display_name()",
                "MenuItemFields::new(profile.display_name())",
            ][..],
        ),
        (
            "app/src/ai/execution_profiles/editor/mod.rs",
            &[
                "\"Default\".to_string()",
                "if let EditorEvent::Edited(_) = event",
                "editor.set_buffer_text(&display_name, ctx)",
            ][..],
        ),
        (
            "app/src/settings_view/execution_profile_view.rs",
            &["profile.display_name()"][..],
        ),
        (
            "app/src/terminal/input/profiles/data_source.rs",
            &["profile_info.data().display_name()"][..],
        ),
    ];

    let violations = selected_snippet_violations(&cases);

    assert!(
        violations.is_empty(),
        "execution profile and model selector UI must use catalog-backed copy: {violations:#?}"
    );
}

#[test]
fn current_ui_display_helpers_do_not_bypass_localization() {
    let cases = [
        (
            "app/src/terminal/rich_history.rs",
            &["entry.output_status.display_text(),"][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/orchestration_pill_bar.rs",
            &["status.to_string()", "\"Agent\".to_string()"][..],
        ),
        (
            "app/src/settings/import/view.rs",
            &["setting.setting_type.get_name()"][..],
        ),
        (
            "app/src/settings_view/features_page.rs",
            &[
                "val.dropdown_item_label()",
                "val.as_dropdown_label()",
                "global_hotkey_mode.as_dropdown_label()",
                "new_tab_placement_dropdown_item_label(",
                "init_global_hotkey_dropdown(",
                "TabBehavior::Completions.dropdown_item_label()",
                ".unwrap_or(\"Default\")",
                "Some(\"Accept Autosuggestion\")",
                "Some(\"Open Completions Menu\")",
            ][..],
        ),
        (
            "app/src/settings_view/appearance_page.rs",
            &[
                "input_mode_dropdown_item_label(",
                "thin_strokes_dropdown_item_label(",
                "enforce_minimum_contrast_dropdown_item_label(",
                "workspace_decoration_visibility_dropdown_item_label(",
                "tab_close_button_position_dropdown_item_label(",
            ][..],
        ),
        (
            "app/src/settings_view/custom_router_view.rs",
            &[
                "\"1 rule\".to_string()",
                "format!(\"{rule_count} rules\")",
                "\"Default:\"",
                "\"Easy:\"",
                "\"Medium:\"",
                "\"Hard:\"",
            ][..],
        ),
        (
            "app/src/settings_view/mcp_servers_page.rs",
            &["Finish the current MCP install before opening another install link."][..],
        ),
        (
            "app/src/workspace/view/right_panel.rs",
            &[".unwrap_or_else(|| \"Unknown\".to_string())"][..],
        ),
        (
            "app/src/terminal/view/pending_user_query.rs",
            &[".unwrap_or_else(|| \"User\".to_owned())"][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/agent_view_block.rs",
            &[
                "Some(\"Open in different pane\")",
                "Some(\"Restored\")",
                "Some(\"Continued\")",
                ".unwrap_or(\"Untitled conversation\".to_string())",
            ][..],
        ),
        (
            "app/src/ai/conversation_navigation/mod.rs",
            &[".unwrap_or_else(|| \"Untitled conversation\".to_string())"][..],
        ),
        (
            "app/src/terminal/input/inline_history/data_source.rs",
            &[".unwrap_or_else(|| \"Untitled conversation\".to_string())"][..],
        ),
        (
            "app/src/code/view.rs",
            &[
                ".unwrap_or_else(|| \"Untitled\".to_string())",
                "None => \"Untitled\".to_string()",
                "secondary.push_str(\" (new)\")",
            ][..],
        ),
        (
            "app/src/code/editor/find/view.rs",
            &[
                "Find bar for searching text in the editor.",
                "Find bar with {} matches found.",
                "Replace field focused. Type replacement text",
                "Find field focused. Type to search text.",
            ][..],
        ),
        (
            "app/src/code/find_references_view.rs",
            &[
                "\"Showing 1 reference\".to_string()",
                "format!(\"Showing {total_refs} references\")",
            ][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/code_diff_view.rs",
            &[
                "format!(\"{file_name} (new)\")",
                "format!(\"{file_name} (deleted)\")",
                "\"No file name\".to_string()",
            ][..],
        ),
        (
            "app/src/notebooks/file/mod.rs",
            &[
                ".unwrap_or_else(|| \"Unnamed\".to_string())",
                "self.render_title(appearance, font_settings)",
            ][..],
        ),
        (
            "app/src/settings_view/execution_profile_view.rs",
            &[
                "\"Run agents:\"",
                "RunAgentsPermission::NeverAllow | RunAgentsPermission::Unknown => \"Never\"",
                "RunAgentsPermission::AlwaysAllow => \"Always allow\"",
                "RunAgentsPermission::AlwaysAsk => \"Always ask\"",
            ][..],
        ),
        (
            "app/src/drive/items/ai_fact_collection.rs",
            &["Some(\"Rules\".to_string())"][..],
        ),
        (
            "app/src/drive/items/mcp_server_collection.rs",
            &["Some(\"MCP Servers\".to_string())"][..],
        ),
        (
            "app/src/ai/agent_management/agent_management_model.rs",
            &[
                "\"Agent task\".to_owned()",
                "\"Notification from Codex\"",
                "\"Task completed.\"",
                "\"Task was cancelled.\"",
                "format!(\"{} completed\", agent.display_name())",
            ][..],
        ),
        (
            "app/src/code_review/comment_rendering.rs",
            &["\"Review Comment\".to_string()"][..],
        ),
        (
            "app/src/ai/agent/comment.rs",
            &["\"Review Comment\".to_string()", "\"Invalid File Name\""][..],
        ),
        (
            "app/src/code_review/comment_list_view.rs",
            &["\"CLI agent\""][..],
        ),
        (
            "app/src/workspace/view/vertical_tabs.rs",
            &[
                "Text::new_inline(status.to_string(),",
                "\"Untitled tab\"",
                "\"New session\"",
                "\"Terminal\".to_string()",
                "\"Unsaved\".to_string()",
                "typed.kind_label().to_string()",
                "format!(\"and {extra_open_tabs} more\")",
            ][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/ask_user_question_view.rs",
            &[
                "AskUserQuestionAnswerItem::display_text",
                "DropdownItem::new(p.label()",
                "set_selected_by_name(permission.label()",
            ][..],
        ),
    ];

    let violations = selected_snippet_violations(&cases);

    assert!(
        violations.is_empty(),
        "UI display helpers must localize before rendering: {violations:#?}"
    );
}

#[test]
fn agent_tips_notebook_links_and_recorder_toasts_use_catalog_copy() {
    let mut required_keys = [
        "agent.tips.prefix",
        "agent.tips.voice",
        "agent.tips.action.open_palette",
        "agent.tips.action.open_warp_drive",
        "agent.tips.action.show_diff_view",
        "common.open",
        "notebook.link.action.new_session",
        "notebook.link.action.new_session_tooltip",
        "notebook.link.action.open_in_terminal_session",
        "notebook.link.action.open_in_editor",
        "notebook.link.action.edit_markdown_file",
        "notebook.editor.a11y.secondary_click",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();
    required_keys.extend((1..=36).map(|index| format!("agent.tips.default.{index:02}")));

    assert_bundled_keys_exist(&required_keys);

    let cases = [
        (
            "app/src/ai/agent_tips.rs",
            &[
                "format!(\"Tip: {}",
                "\"Open palette\".to_string()",
                "\"Warp Drive.\".to_string()",
                "\"Show diff view\".to_string()",
            ][..],
        ),
        (
            "app/src/workspace/view/tab_grouping.rs",
            &[".unwrap_or_else(|| \"Untitled group\".to_string())"][..],
        ),
        (
            "app/src/workflows/workflow_view.rs",
            &[
                "display_error_toast(\"Error saving aliases\".to_string()",
                "\"This workflow cannot be saved because it contains secrets\".to_string()",
                "String::from(\"Could not create workflow\")",
                "\"Prompt copied.\".to_string()",
                "\"Command copied.\".to_string()",
                "\"Looks like you're out of AI credits. Contact a team admin to upgrade for more credits.\".to_string()",
            ][..],
        ),
        (
            "app/src/notebooks/link.rs",
            &[
                "label: \"New session\"",
                "tooltip: Some(\"Open a new terminal session in this directory\"",
                "accessibility_content: \"Open in terminal session\"",
                "label: \"Open in editor\"",
                "accessibility_content: \"Edit Markdown file\"",
            ][..],
        ),
        (
            "app/src/notebooks/editor/view.rs",
            &["format!(\"Secondary click on {}\""][..],
        ),
        (
            "app/src/terminal/recorder.rs",
            &[
                "ToastLink::new(\"Open\"",
                "format!(\"PTY recording started: {display_path}\")",
                "format!(\"PTY recording stopped: {display_path}\")",
            ][..],
        ),
    ];

    let violations = selected_snippet_violations(&cases);

    assert!(
        violations.is_empty(),
        "agent tips, notebook link actions, and recorder toasts must use catalog copy: {violations:#?}"
    );
}

#[test]
fn ai_settings_high_risk_wrappers_do_not_use_direct_english_literals() {
    let relative_path = "app/src/settings_view/ai_page.rs";
    let path = workspace_root().join(relative_path);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let mut violations = Vec::new();

    collect_direct_literal_after_patterns(
        relative_path,
        &content,
        AI_SETTINGS_HIGH_RISK_UI_PATTERNS,
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "AI settings high-risk wrappers must use AppContext localization: {violations:#?}"
    );
}

#[test]
fn ai_settings_mode_command_bindings_use_dynamic_localized_descriptions() {
    let source_lines = AI_SETTINGS_PAGE_SOURCE
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>();
    let dynamic_description_call_count = source_lines
        .windows(3)
        .filter(|lines| {
            lines[0] == "localized_binding_description("
                && lines[1] == "mode.command_palette_description(),"
                && lines[2] == "mode.command_palette_description_key(),"
        })
        .count();

    assert!(
        AI_SETTINGS_PAGE_SOURCE.contains("fn localized_binding_description"),
        "AI settings should construct catalog-backed dynamic binding descriptions"
    );
    assert!(
        AI_SETTINGS_PAGE_SOURCE.contains(".with_dynamic_override("),
        "AI settings binding descriptions should refresh after runtime language changes"
    );
    assert!(
        dynamic_description_call_count >= 3,
        "AI settings mode command bindings should use dynamic localized descriptions"
    );
    assert!(
        !AI_SETTINGS_PAGE_SOURCE
            .contains("text_for_app(app, mode.command_palette_description_key())"),
        "AI settings mode command bindings should not capture localized labels at registration time"
    );
}

#[test]
fn static_slash_command_descriptions_have_catalog_keys() {
    let command_names = static_slash_command_names_from_source();
    assert!(!command_names.is_empty(), "expected static slash commands");

    let keys = command_names
        .iter()
        .map(|name| slash_command_localization_key(name, "description"))
        .collect::<Vec<_>>();

    assert_bundled_keys_exist(&keys);
}

#[test]
fn static_slash_command_argument_hint_keys_exist_in_catalogs() {
    let keys = static_slash_command_hint_keys_from_source();
    assert!(!keys.is_empty(), "expected static slash command hint keys");

    assert_bundled_keys_exist(&keys);
}

#[test]
fn static_slash_command_english_hint_fallbacks_have_catalog_keys() {
    let relative_path = "app/src/search/slash_command_menu/static_commands/commands.rs";
    let path = workspace_root().join(relative_path);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let mut violations = Vec::new();
    let mut cursor = 0;

    while let Some(found_at) = content[cursor..].find(".with_hint_text(\"") {
        let invocation_start = cursor + found_at;
        let mut scan_end = (invocation_start + 192).min(content.len());
        while scan_end < content.len() && !content.is_char_boundary(scan_end) {
            scan_end += 1;
        }
        if !content[invocation_start..scan_end].contains(".with_hint_text_key(") {
            violations.push(format!(
                "{}:{}: with_hint_text literal has no catalog key",
                relative_path,
                line_number_for_offset(&content, invocation_start)
            ));
        }
        cursor = invocation_start + ".with_hint_text(\"".len();
    }

    assert!(
        violations.is_empty(),
        "static slash command hint literals must pair with with_hint_text_key: {violations:#?}"
    );
}

#[test]
fn binding_description_new_does_not_use_direct_english_literals() {
    let app_src = workspace_root().join("app/src");
    let mut violations = Vec::new();
    collect_direct_first_argument_literal_violations_in_dir(
        &app_src,
        "BindingDescription::new(",
        &mut violations,
        Some(binding_description_catalog_map_source()),
    );

    assert!(
        violations.is_empty(),
        "BindingDescription::new must use a fallback variable plus dynamic catalog override: {violations:#?}"
    );
}

#[test]
fn editable_binding_descriptions_do_not_use_direct_english_literals() {
    let mut violations = Vec::new();
    for relative_root in ["app/src", "crates/warp_tui/src"] {
        collect_binding_description_literal_violations_in_dir(
            &workspace_root().join(relative_root),
            "EditableBinding::new(",
            1,
            &mut violations,
            Some(binding_description_catalog_map_source()),
        );
    }

    assert!(
        violations.is_empty(),
        "EditableBinding descriptions must use catalog-backed BindingDescription values: {violations:#?}"
    );
}

#[test]
fn plugin_instruction_keys_exist_in_catalogs() {
    let keys = plugin_instruction_keys_from_source();
    assert!(!keys.is_empty(), "expected plugin instruction keys");

    assert_bundled_keys_exist(&keys);

    let violations = plugin_instruction_key_violations_from_source();
    assert!(
        violations.is_empty(),
        "plugin instruction fallbacks must have catalog keys: {violations:#?}"
    );
}

#[test]
fn static_prompt_suggestion_keys_exist_in_catalogs() {
    let keys = static_prompt_suggestion_keys_from_source();
    assert!(!keys.is_empty(), "expected static prompt suggestion keys");

    assert_bundled_keys_exist(&keys);
}

#[test]
fn zero_state_prompt_suggestion_keys_exist_in_catalogs() {
    let keys = zero_state_prompt_suggestion_keys_from_source();
    assert!(
        !keys.is_empty(),
        "expected zero-state prompt suggestion keys"
    );

    assert_bundled_keys_exist(&keys);
}

#[test]
fn selected_search_accessibility_and_web_home_keys_exist_in_catalogs() {
    let required_keys = [
        "search.a11y.help.confirm",
        "search.a11y.help.confirm_with_binding",
        "search.a11y.item_with_binding",
        "search.a11y.type.ai_query",
        "search.a11y.type.block",
        "search.a11y.type.code_symbol",
        "search.a11y.type.command",
        "search.a11y.type.conversation",
        "search.a11y.type.history_item",
        "search.a11y.type.project",
        "search.a11y.type.repo",
        "search.a11y.type.rule",
        "search.a11y.type.section",
        "search.a11y.type.secret",
        "search.a11y.type.skill",
        "search.a11y.type.warp_ai",
        "search.a11y.type.workflow",
        "search.a11y.type.workflow_with_description",
        "workspace.home.content",
        "workspace.home.title",
        "agent.plan_and_todo.tooltip.unaware_of_plan_edits",
        "agent.plan_and_todo.tooltip.view_plan",
        "agent.plan_and_todo.tooltip.view_todo_list",
        "agent.suggested_workflow.tooltip.prompt",
        "agent.usage.credit.plural",
        "agent.usage.credit.singular",
        "input_suggestions.time.day_many",
        "input_suggestions.time.day_one",
        "input_suggestions.time.hour_many",
        "input_suggestions.time.hour_one",
        "input_suggestions.time.just_now",
        "input_suggestions.time.minute",
        "input_suggestions.time.month_many",
        "input_suggestions.time.month_one",
        "input_suggestions.time.week_many",
        "input_suggestions.time.week_one",
        "input_suggestions.time.year_many",
        "input_suggestions.time.year_one",
    ];

    assert_bundled_keys_exist(&required_keys);
}

#[test]
fn search_items_with_accessibility_labels_have_app_aware_overrides() {
    fn collect_missing_overrides(dir: &Path, violations: &mut Vec<String>) {
        let entries = fs::read_dir(dir)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
        for entry in entries {
            let path = entry.expect("failed to read search source entry").path();
            if path.is_dir() {
                collect_missing_overrides(&path, violations);
                continue;
            }
            let is_production_rust = path.extension().and_then(|ext| ext.to_str()) == Some("rs")
                && !path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with("_tests.rs") || name == "test.rs");
            if !is_production_rust {
                continue;
            }

            let content = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
            if content.contains("fn accessibility_label(&self)")
                && !content.contains("fn accessibility_label_for_app")
            {
                violations.push(
                    path.strip_prefix(workspace_root())
                        .unwrap_or(path.as_path())
                        .display()
                        .to_string(),
                );
            }
        }
    }

    let mut violations = Vec::new();
    collect_missing_overrides(&workspace_root().join("app/src/search"), &mut violations);
    assert!(
        violations.is_empty(),
        "search items with accessibility labels need app-aware localization: {violations:#?}"
    );
}

#[test]
fn working_directory_voice_and_separator_keys_exist_in_catalogs() {
    let required_keys = [
        "settings.features.working_directory.directory_placeholder",
        "settings.features.working_directory.label",
        "settings.features.working_directory.new_tab",
        "settings.features.working_directory.new_window",
        "settings.features.working_directory.option.advanced",
        "settings.features.working_directory.option.custom",
        "settings.features.working_directory.option.home",
        "settings.features.working_directory.option.previous",
        "settings.features.working_directory.split_pane",
        "terminal.block_list.separator.previous_session",
        "terminal.block_list.separator.restored",
        "terminal.block_list.separator.with_timestamp",
        "terminal.input.voice.listening",
        "terminal.input.voice.transcribing",
    ];

    assert_bundled_keys_exist(&required_keys);
}

#[test]
fn binding_search_accessibility_label_does_not_embed_selected_state() {
    let catalogs = [
        ("en-US", bundled_en_us_map(), "Selected"),
        ("zh-CN", bundled_zh_cn_map(), "已选择"),
    ];

    for (locale, catalog, selected_prefix) in catalogs {
        let item_with_binding = catalog
            .get("search.a11y.item_with_binding")
            .and_then(|value| value.as_str())
            .unwrap_or_else(|| panic!("{locale}: missing search.a11y.item_with_binding"));

        assert!(
            !item_with_binding.contains(selected_prefix),
            "{locale}: search.a11y.item_with_binding must leave selected state to search.a11y.selected_item"
        );
    }
}

#[test]
fn ambient_agent_model_selector_default_label_uses_catalog_copy() {
    let default_label_key = "settings.ai.model_selector.default_model";
    let occurrences = AMBIENT_AGENT_MODEL_SELECTOR_SOURCE
        .match_indices(default_label_key)
        .count();

    assert!(
        occurrences >= 2,
        "ambient model selector should use {default_label_key} for both button and menu labels"
    );
    assert!(
        !AMBIENT_AGENT_MODEL_SELECTOR_SOURCE.contains("MenuItemFields::new(\"default\")"),
        "ambient model selector menu should not render the default label from a direct string"
    );
    assert!(
        !AMBIENT_AGENT_MODEL_SELECTOR_SOURCE.contains("\"default\".to_string()"),
        "ambient model selector button should not fall back to a direct default label"
    );
}

#[test]
fn current_i18n_regression_targets_are_catalog_backed() {
    let required_keys = [
        "agent.error.context_window_exceeded",
        "agent.error.llm_unavailable",
        "agent.error.request_incomplete",
        "agent.error.response_stream_internal",
        "agent.error.response_stream_other",
        "agent.input_footer.platform_plugin_install_no_effect",
        "agent.input_footer.platform_plugin_update_no_effect",
        "agent.input_footer.plugin_auto_install_unsupported",
        "agent.input_footer.plugin_auto_update_unsupported",
        "agent.input_footer.plugin_command_failed",
        "agent.input_footer.plugin_command_run_failed",
        "agent.input_footer.plugin_update_no_effect",
        "agent.input_footer.toolbar_item.attach_file",
        "agent.input_footer.toolbar_item.autodetection",
        "agent.input_footer.toolbar_item.context_chip",
        "agent.input_footer.toolbar_item.context_usage",
        "agent.input_footer.toolbar_item.fast_forward",
        "agent.input_footer.toolbar_item.file_explorer",
        "agent.input_footer.toolbar_item.handoff_to_cloud",
        "agent.input_footer.toolbar_item.model_selector",
        "agent.input_footer.toolbar_item.rich_input",
        "agent.input_footer.toolbar_item.settings",
        "agent.input_footer.toolbar_item.voice_input",
        "agent.history.output_status.cancelled",
        "agent.history.output_status.completed",
        "agent.history.output_status.failed",
        "agent.history.output_status.pending",
        "agent_management.agent_type_selector.cloud.description",
        "agent_management.agent_type_selector.cloud.title",
        "agent_management.agent_type_selector.local.description",
        "agent_management.agent_type_selector.local.title",
        "agent_management.notifications.cli_completed_title",
        "agent_management.notifications.cli_needs_attention_title",
        "agent_management.notifications.codex_message",
        "agent_management.notifications.something_went_wrong",
        "agent_management.notifications.task_cancelled",
        "agent_management.notifications.task_completed",
        "agent_management.notifications.waiting_for_input",
        "agent.orchestration.error.no_targets",
        "agent.orchestration.error.source_conversation_not_found",
        "agent.orchestration.run_agents.accept",
        "agent.orchestration.run_agents.reject",
        "agent.output.mermaid_diagram",
        "agent.ask_user_question.other",
        "agent.ask_user_question.select_all_suffix",
        "agent.ask_user_question.speedbump.allow_questions",
        "agent.ask_user_question.summary.answer_prefix",
        "agent.ask_user_question.summary.question_prefix",
        "agent.ask_user_question.summary.skipped",
        "agent.search_codebase.cancelled",
        "agent.search_codebase.cancelled_in_repo",
        "agent.search_codebase.error.codebase_unavailable",
        "agent.search_codebase.error.current_directory_unavailable",
        "agent.search_codebase.error.indexing",
        "agent.search_codebase.error.missing_files",
        "agent.search_codebase.error.missing_git_repo",
        "agent.search_codebase.error.remote_host_not_connected",
        "agent.search_codebase.error.remote_indexing",
        "agent.search_codebase.error.remote_no_repo",
        "agent.search_codebase.error.remote_not_enabled",
        "agent.search_codebase.error.remote_not_indexed",
        "agent.search_codebase.error.remote_read_failed",
        "agent.search_codebase.error.remote_read_unknown",
        "agent.search_codebase.error.remote_server_not_connected",
        "agent.search_codebase.error.remote_unavailable_for_path",
        "agent.search_codebase.error.remote_unexpected_unavailable",
        "agent.search_codebase.error.remote_unavailable",
        "agent.search_codebase.error.search_failed",
        "agent.search_codebase.searched_codebase",
        "agent.search_codebase.searched_codebase_in_repo",
        "agent.search_codebase.searching",
        "agent.search_codebase.searching_in_repo",
        "agent.search_results.results_label",
        "agent.search_results.urls_label",
        "agent.cli.search_action.current_directory",
        "agent.cli.search_action.file_glob.multiple",
        "agent.cli.search_action.file_glob.single",
        "agent.cli.search_action.grep.multiple",
        "agent.cli.search_action.grep.single",
        "agent.task_status.aws_bedrock_credentials_expired_or_invalid",
        "agent.task_status.blocked",
        "agent.task_status.cancelled",
        "agent.task_status.context_window_exceeded",
        "agent.task_status.error",
        "agent.task_status.internal_error",
        "agent.task_status.invalid_api_key",
        "agent.task_status.quota_limit",
        "agent.task_status.server_overloaded",
        "agent.web_fetch.failed_url",
        "agent.view_block.deleted",
        "agent.view_block.deleted_conversation",
        "agent.zero_state.free_cloud_agent_credits",
        "agent_sdk.environment.error.fetch_images",
        "ai_document.title.default",
        "code.remote_disconnected.banner",
        "code.toast.save_failed_remote_disconnected",
        "code_review.comment.invalid_file_name",
        "code_review.comment.review_comment",
        "code_review.comments.cli_agent",
        "conversation_details.tooltip.cancel_task",
        "conversation_details.tooltip.copy_link_to_run",
        "conversation_details.tooltip.fork_conversation",
        "conversation_details.tooltip.open_conversation",
        "conversation_details.tooltip.view_details",
        "drive.collection.mcp_servers",
        "drive.collection.rules",
        "editor.ai_context_menu.search_files_tooltip",
        "editor.toast.image_limit.per_conversation",
        "editor.toast.image_limit.per_query",
        "editor.toast.image_limit.plural",
        "editor.toast.image_limit.single",
        "editor.toast.image_processing_failed.plural",
        "editor.toast.image_processing_failed.single",
        "editor.toast.image_processing_failed.single_only",
        "editor.toast.image_too_large.plural",
        "editor.toast.image_too_large.single",
        "editor.toast.image_too_large.single_only",
        "feature_popup.badge.new",
        "notebook.editor.a11y.selected_workflow",
        "remote.codebase_search.host_disconnected",
        "remote.codebase_search.missing_root_hash",
        "remote.codebase_search.unavailable",
        "remote.host.unknown",
        "settings.ai.custom_endpoint.usage_fallback",
        "settings.ai.aws_bedrock.credentials.status.disabled.detail",
        "settings.ai.aws_bedrock.credentials.status.disabled.title",
        "settings.ai.aws_bedrock.credentials.status.failed.title",
        "settings.ai.aws_bedrock.credentials.status.loaded.detail",
        "settings.ai.aws_bedrock.credentials.status.loaded.detail_with_expiration",
        "terminal.notification.ai_summary.permission.confirmation",
        "terminal.notification.ai_summary.permission.edit_file",
        "terminal.notification.ai_summary.permission.interact_shell",
        "terminal.notification.ai_summary.permission.read_files",
        "terminal.notification.ai_summary.permission.run_command",
        "terminal.notification.ai_summary.permission.search_codebase",
        "terminal.notification.default_title",
        "terminal.status.checking",
        "terminal.status.initializing",
        "terminal.status.installing",
        "terminal.status.installing_with_progress",
        "terminal.status.starting_shell",
        "terminal.status.updating",
        "settings.ai.aws_bedrock.credentials.status.loaded.title",
        "settings.ai.aws_bedrock.credentials.status.missing.detail",
        "settings.ai.aws_bedrock.credentials.status.missing.title",
        "settings.ai.aws_bedrock.credentials.status.refreshing.detail",
        "settings.ai.aws_bedrock.credentials.status.refreshing.title",
        "settings.billing.addon_credits.auto_reload.managed.title",
        "settings.environment.create.toast.failed",
        "settings.execution_profile.long_context_pricing_warning.message",
        "settings.execution_profile.run_agents",
        "settings.features.code_editor_line_numbers.absolute",
        "settings.features.code_editor_line_numbers.relative",
        "settings.import.setting.copy_on_select",
        "settings.import.setting.cursor_blinking",
        "settings.import.setting.default_shell",
        "settings.import.setting.font",
        "settings.import.setting.hotkey_mode",
        "settings.import.setting.mouse_scroll_reporting",
        "settings.import.setting.opacity",
        "settings.import.setting.option_as_meta",
        "settings.import.setting.theme",
        "settings.import.setting.window_size",
        "settings.import.setting.working_directory",
        "settings.mcp.install.no_server_selected",
        "settings.mcp.oauth.headless_authentication_required",
        "settings.mcp.toast.authenticated_server",
        "settings.theme_creator.error.process_image",
        "settings.theme_creator.error.process_image_with_error",
        "tab_config.new_worktree.branch_name",
        "terminal.rewind.a11y.current",
        "terminal.rewind.current",
        "terminal.slash.command.or_separator",
        "terminal.ssh.install_tmux.run_script_header",
        "terminal.ssh.install_tmux.title",
        "terminal.ssh_error.start_extension_failed",
        "terminal.shared_session.error.access_removed",
        "terminal.shared_session.error.command_execution_failed",
        "terminal.shared_session.error.command_in_progress",
        "terminal.shared_session.error.control_action_failed",
        "terminal.shared_session.error.guests_already_added",
        "terminal.shared_session.error.guests_not_warp_users",
        "terminal.shared_session.error.initialize_internal",
        "terminal.shared_session.error.insufficient_permissions_request_edit",
        "terminal.shared_session.error.invalid_conversation",
        "terminal.shared_session.error.login_required",
        "terminal.shared_session.error.quota_exceeded",
        "terminal.shared_session.error.scrollback_too_large",
        "terminal.shared_session.error.session_ended",
        "terminal.shared_session.error.session_ended_internal",
        "terminal.shared_session.error.session_ended_sharer_inactive",
        "terminal.shared_session.error.session_terminated_internal",
        "terminal.shared_session.error.size_limit_exceeded",
        "terminal.shared_session.error.write_failed",
        "terminal.shared_session.toast.reconnect_failed",
        "terminal.shared_session.toast.share_again_failed",
        "terminal.warpify.success.auto_warpify_instructions",
        "terminal.warpify.success.learn_more",
        "terminal.warpify.success.remote_subshell_description",
        "terminal.warpify.success.title",
        "workspace.handoff.auto_cloud_prompt",
        "workspace.codex_modal.initial_prompt",
        "launch_config.save_modal.a11y.description",
        "launch_config.save_modal.a11y.title",
        "launch_config.save_modal.action.open_file",
        "launch_config.save_modal.action.save",
        "launch_config.save_modal.description",
        "launch_config.save_modal.description_with_keybinding",
        "launch_config.save_modal.documentation_link",
        "launch_config.save_modal.error.file_already_exists",
        "launch_config.save_modal.error.other",
        "launch_config.save_modal.path_prefix",
        "launch_config.save_modal.sentence_suffix",
        "launch_config.save_modal.success_prefix",
        "launch_config.save_modal.title",
        "network_log.action.refresh",
        "network_log.title",
        "terminal.block_filter.a11y.content",
        "terminal.block_filter.a11y.hint",
        "terminal.block_filter.placeholder",
        "terminal.block_filter.tooltip.case_sensitive",
        "terminal.block_filter.tooltip.context_lines",
        "terminal.block_filter.tooltip.invert",
        "terminal.block_filter.tooltip.regex",
        "workflow.env_vars.new",
        "workflow.env_vars.title",
        "workflow.info_box.command_edited",
        "workflow.info_box.reset",
        "workflow.info_box.view_context",
        "workspace.autoupdate.package_manager.description_prefix",
        "workspace.autoupdate.package_manager.description_suffix",
        "workspace.autoupdate.package_manager.dist_upgrade_prefix",
        "workspace.autoupdate.package_manager.dist_upgrade_suffix",
        "workspace.autoupdate.package_manager.footer_prefix",
        "workspace.autoupdate.package_manager.footer_suffix",
        "workspace.autoupdate.package_manager.press_enter",
        "workspace.autoupdate.package_manager.report_issues",
        "workspace.autoupdate.package_manager.repository_configuration",
        "workspace.autoupdate.package_manager.title",
    ];

    assert_bundled_keys_exist(&required_keys);

    let cases = [
        (
            "app/src/settings_view/mcp_servers/installation_modal.rs",
            &["No MCP server selected"][..],
        ),
        (
            "app/src/settings_view/billing_and_usage_page_v2.rs",
            &[
                "const MANAGED_AUTO_RELOAD_HEADER",
                "\"Auto-reload is enabled\"",
            ][..],
        ),
        ("app/src/util/path.rs", &["\"Remote host\""][..]),
        (
            "app/src/ai/document/ai_document_model.rs",
            &["\"Planning document\""][..],
        ),
        (
            "app/src/ai/blocklist/controller.rs",
            &[
                "Request did not successfully complete",
                "Response stream finished unexpectedly (with finish reason `Other`).",
                "Input exceeded context window limit.",
                "The LLM is currently unavailable.",
            ][..],
        ),
        (
            "app/src/ai/blocklist/action_model/execute/search_codebase.rs",
            &[
                "The current directory isn't within a git repository",
                "The current git repository is still being indexed",
                "The search failed. Try another way",
                "Remote codebase search is unavailable.",
            ][..],
        ),
        (
            "app/src/ai/get_relevant_files/remote_search/native.rs",
            &[
                "Remote codebase search is not enabled.",
                "Remote codebase search is unavailable because the remote server is not connected.",
                "Remote codebase search is unavailable because the remote host is not connected.",
                "The current remote directory is not in a known codebase.",
                "The remote codebase at {} is not indexed yet.",
                "The remote codebase at {} is still being indexed. Try again later.",
                "Failed to read remote search result files: {failed}",
                "\"unknown error\"",
                "Remote codebase search is unavailable for {}: {message}",
                "Remote codebase search was unexpectedly unavailable.",
            ][..],
        ),
        (
            "app/src/ai/get_relevant_files/remote_search/wasm.rs",
            &["Remote codebase search is not available in this environment."][..],
        ),
        (
            "app/src/ai/blocklist/orchestration_events.rs",
            &["Source conversation not found", "No target agents provided"][..],
        ),
        (
            "app/src/terminal/view.rs",
            &[
                "\"Failed to start SSH extension\"",
                "Execute this plan",
                "An unknown error occurred",
            ][..],
        ),
        (
            "app/src/terminal/input/rewind/search_item.rs",
            &["query_text: \"Current\"", "Text::new_inline(\"Current\""][..],
        ),
        (
            "app/src/tab_configs/session_config.rs",
            &["New worktree branch name"][..],
        ),
        (
            "app/src/terminal/cli_agent_sessions/plugin_manager/mod.rs",
            &[
                "Auto-install is not supported for this agent",
                "Auto-update is not supported for this agent",
                "format!(\"'{}' failed",
                "format!(\"Failed to run",
            ][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/ask_user_question_view.rs",
            &[
                "\"Other...\"",
                "\" (select all that apply)\"",
                "\"Allow the agent to ask questions:\"",
                "format!(\"Q: {}\"",
                "format!(\"A: {}\"",
                "\"Skipped\".to_string()",
            ][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/search_codebase.rs",
            &[
                "format!(\"Searching for",
                "format!(\"Searching codebase",
                "format!(\"Search for",
                "format!(\"Searched codebase",
                "\"results\"",
            ][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/web_search.rs",
            &["\"URLs\""][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/web_fetch.rs",
            &["\"URLs\"", "\"\u{2717} {display_text}\""][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/create_environment_modal.rs",
            &["Failed to create environment"][..],
        ),
        (
            "app/src/code/local_code_editor.rs",
            &["Save and auto-reload are unavailable while the remote session is disconnected"][..],
        ),
        (
            "crates/mcp/src/oauth.rs",
            &["MCP server requires OAuth authentication. Please authenticate this server"][..],
        ),
        (
            "app/src/ai/agent_sdk/environment.rs",
            &["Failed to fetch images"][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/run_agents_card_view.rs",
            &["\"Accept\"", "\"Reject\""][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/agent_view_block.rs",
            &["\"Deleted conversation\"", "\"Deleted\""][..],
        ),
        (
            "app/src/terminal/warpify/success_block.rs",
            &[
                "Run the following to automatically Warpify in the future:",
                "In remote subshells, Warp runs commands in the background",
                "\"Session Warpified\"",
                "\"Learn more\".into()",
            ][..],
        ),
        (
            "app/src/themes/theme_creator_body.rs",
            &["Failed to process selected image"][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/zero_state_block.rs",
            &["free cloud agent credits", "\"RECENT ACTIVITY\""][..],
        ),
        (
            "app/src/editor/view/mod.rs",
            &[
                "limit is {MAX_IMAGE_COUNT_FOR_QUERY} per query",
                "limit is {MAX_IMAGES_PER_CONVERSATION} per conversation",
                "Image cannot be attached - file is too large.",
                "Image cannot be attached - error processing.",
                "images weren't attached - error processing.",
            ][..],
        ),
        (
            "app/src/notebooks/editor/model.rs",
            &["Selected workflow: {command}"][..],
        ),
        (
            "app/src/ai/blocklist/block/cli.rs",
            &[
                "\"the current directory\"",
                "Grep for `{}`",
                "Grep for the following patterns",
                "Search for files that match",
                "Find files that match the following patterns",
            ][..],
        ),
        (
            "app/src/ai/blocklist/local_agent_task_sync_model.rs",
            &[
                "Agent encountered an error",
                "Cancelled by user",
                "The agent got stuck waiting for user confirmation",
            ][..],
        ),
        (
            "app/src/launch_configs/save_modal.rs",
            &[
                "Save Configuration",
                "Open YAML File",
                "Save Current Configuration",
                "Link to Documentation",
                "Failed to save. A launch configuration with the same name already exists.",
                "An issue was encountered while saving.",
                "This will save your current configuration of windows",
                "The YAML file is saved to",
                "Save Config Modal",
                "Type the name of the file to which you want to save",
            ][..],
        ),
        (
            "app/src/autoupdate/linux.rs",
            &[
                "Run {package_manager_name} to update",
                "or a compatible tool, the pre-filled command will update Warp for you.",
                "The command below includes a one-time configuration",
                "function ensures the Warp package repository is enabled",
                "Review the command below, then",
                "press enter",
                "Please report any issues",
            ][..],
        ),
        (
            "app/src/workflows/info_box.rs",
            &[
                "ENV_VAR_SPAN: &str = \"Environment variables\"",
                "NEW_ENV_VAR_BUTTON_LABEL: &str = \"New environment variables\"",
                "Command edited.",
                "String::from(\"Reset\")",
                "\"View Context\"",
            ][..],
        ),
        (
            "app/src/server/network_log_view.rs",
            &[
                "NETWORK_LOG_HEADER_TEXT: &str = \"Network log\"",
                "REFRESH_TOOLTIP: &str = \"Refresh\"",
            ][..],
        ),
        (
            "app/src/terminal/block_filter.rs",
            &[
                "Filter block output",
                "Show context lines around matches",
                "Regex toggle",
                "Case sensitive search",
                "Invert filter",
                "Type searched phrase.",
                "Press escape to quit",
            ][..],
        ),
        (
            "app/src/drive/index.rs",
            &[
                "View teams to join",
                "View team to join",
                "Text::new_inline(\"Or\"",
            ][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/agent_message_bar.rs",
            &["Message::from_text(\"Starting shell...\")"][..],
        ),
    ];

    let violations = selected_snippet_violations(&cases);

    assert!(
        violations.is_empty(),
        "current i18n regression targets must use catalog copy: {violations:#?}"
    );
}

#[test]
fn shared_session_agent_notifications_and_aws_status_use_catalog_copy() {
    let required_keys = [
        "agent_management.notifications.cli_needs_attention_title",
        "agent_management.notifications.something_went_wrong",
        "agent_management.notifications.waiting_for_input",
        "settings.ai.aws_bedrock.credentials.error.oidc_mint_token_failed",
        "settings.ai.aws_bedrock.credentials.error.oidc_missing_credentials",
        "settings.ai.aws_bedrock.credentials.error.oidc_sts_failed",
        "settings.ai.aws_bedrock.credentials.error.oidc_task_id_required",
        "settings.ai.aws_bedrock.credentials.error.refresh_interrupted",
        "settings.ai.aws_bedrock.credentials.status.loaded.detail",
        "settings.ai.aws_bedrock.credentials.status.loaded.detail_with_expiration",
        "terminal.shared_session.error.access_removed",
        "terminal.shared_session.error.command_execution_failed",
        "terminal.shared_session.error.command_in_progress",
        "terminal.shared_session.error.control_action_failed",
        "terminal.shared_session.error.guests_already_added",
        "terminal.shared_session.error.guests_not_warp_users",
        "terminal.shared_session.error.initialize_internal",
        "terminal.shared_session.error.insufficient_permissions_request_edit",
        "terminal.shared_session.error.invalid_conversation",
        "terminal.shared_session.error.login_required",
        "terminal.shared_session.error.quota_exceeded",
        "terminal.shared_session.error.scrollback_too_large",
        "terminal.shared_session.error.session_ended",
        "terminal.shared_session.error.session_ended_internal",
        "terminal.shared_session.error.session_ended_sharer_inactive",
        "terminal.shared_session.error.session_terminated_internal",
        "terminal.shared_session.error.size_limit_exceeded",
        "terminal.shared_session.error.write_failed",
        "terminal.shared_session.toast.reconnect_failed",
        "terminal.shared_session.toast.share_again_failed",
    ];

    assert_bundled_keys_exist(&required_keys);

    let cases = [
        (
            "app/src/ai/agent_management/agent_management_model.rs",
            &[
                "needs attention",
                "\"Waiting for input.\"",
                "\"Something went wrong.\"",
            ][..],
        ),
        (
            "app/src/settings_view/ai_page.rs",
            &[
                "state.user_facing_components()",
                "Loaded at {}, expires {}",
                "Loaded at {}",
            ][..],
        ),
        (
            "app/src/ai/aws_credentials.rs",
            &[
                "let message = \"AWS Bedrock inference requires",
                ".context(\"Failed to mint AWS Bedrock task identity token\")",
                "anyhow::anyhow!(\"STS AssumeRoleWithWebIdentity failed:",
                ".context(\"STS response did not include credentials\")",
                "Err(\"Credential refresh was interrupted\".to_string())",
            ][..],
        ),
        (
            "app/src/terminal/local_tty/terminal_manager.rs",
            &["Something went wrong. Please try sharing again."][..],
        ),
        (
            "app/src/terminal/local_tty/terminal_view_adaptor.rs",
            &[
                "const ACL_UPDATE_FAILURE_RESPONSE",
                "Something went wrong. Please try sharing again.",
                "Failed to update permissions for shared session",
            ][..],
        ),
        (
            "app/src/terminal/shared_session/sharer/network.rs",
            &[
                "Session sharing usage exceeded for the day",
                "Session limit (",
                "Session ended due to an internal error",
                "An internal error occurred",
                "Scrollback exceeds limit",
                "You must be logged in to share sessions",
                "One or more emails were not associated with Warp accounts",
                "One or more emails have already been added",
            ][..],
        ),
        (
            "app/src/terminal/shared_session/viewer/network.rs",
            &[
                "Please ask sharer",
                "Sharing ended due to sharer inactivity",
                "\"Session ended.\"",
                "Your access to the session was removed",
                "Insufficient permissions. Please request edit access.",
                "Failed to execute command. Please try again.",
                "Failed to make edit. Please try again.",
                "Invalid conversation. Please try again.",
                "A long running command is currently in progress",
                "Failed to perform action. Please try again.",
            ][..],
        ),
        (
            "app/src/terminal/input.rs",
            &[
                "pub const INPUT_A11Y_LABEL: &str",
                "pub const INPUT_A11Y_HELPER: &str",
                "Workflow command {} inserted.",
                "Press shift-tab to select the next workflow argument",
                "Selected Workflow argument {}",
                "Executed: {command}",
            ][..],
        ),
        (
            "app/src/terminal/view.rs",
            &[
                "Toggle Bookmark block",
                "Selected {} blocks.",
                "Selected all {} blocks.",
                "Scrolled to bottom of selected block",
                "Scrolled to top of selected block",
                "Scrolled to bottom of bottommost visible block",
                "Copied {} block outputs.",
                "Copied {} blocks.",
                "Open block filter editor for block {block_index}",
                "Showed initialization block",
                "Opened Warpify Settings",
                "Opened file search palette",
                "Open list of blocks attached as context to this AI query.",
                "Open overflow menu with copy options for this AI block.",
                "Show confirmation dialog to rewind to before this point",
                "Execute rewind to before this point",
                "Click on a block attached as context to this AI query.",
                "Use file picker to select a git repository",
                "You can press {} to Warpify this {} for more Warp features.",
                "You can Warpify this {lowercase_title} for more Warp features.",
                "format!(\"{title} recognized.\")",
                ".notifications_error_banner_title()",
                ".unwrap_or(\"Error sending notification\")",
                "Make sure you have enabled access for Warp notifications",
                "Oz needs your permission to run",
                "Oz needs your permission to read files",
                "Oz needs your permission to search your codebase",
                "Oz needs your permission to edit a file",
                "Oz needs your permission to interact with a running shell command",
                "Oz needs your confirmation to continue",
                "unwrap_or_else(|| \"Notification\".to_string())",
                "RemoteServerSetupState::Checking => \"Checking...\".to_string()",
                "format!(\"Installing... ({p}%)\")",
                "progress_percent: None,\n                        } => \"Installing...\".to_string()",
                "RemoteServerSetupState::Updating => \"Updating...\".to_string()",
                "RemoteServerSetupState::Initializing => \"Initializing...\".to_string()",
                "unwrap_or_else(|| \"Starting shell...\".to_string())",
            ][..],
        ),
        (
            "app/src/terminal/shared_session/viewer/terminal_manager.rs",
            &[
                "Failed to reconnect. Please try again later.",
                "One or more of the emails are not Warp users.",
                "One or more of the guests has already been added.",
            ][..],
        ),
    ];

    let violations = selected_snippet_violations(&cases);

    assert!(
        violations.is_empty(),
        "shared-session, notification, and AWS status copy must use catalogs: {violations:#?}"
    );
}

#[test]
fn agent_sdk_text_output_uses_catalog_copy() {
    let required_keys = [
        "agent_sdk.admin.login.already_logged_in",
        "agent_sdk.admin.login.already_logged_in_as",
        "agent_sdk.admin.login.already_logged_in_with_email",
        "agent_sdk.admin.login.authentication_failed",
        "agent_sdk.admin.login.authentication_failed_with_error",
        "agent_sdk.admin.login.enter_code",
        "agent_sdk.admin.login.open_url",
        "agent_sdk.admin.login.success",
        "agent_sdk.admin.logout.not_logged_in",
        "agent_sdk.admin.logout.success",
        "agent_sdk.admin.whoami.display_name",
        "agent_sdk.admin.whoami.email",
        "agent_sdk.admin.whoami.error.serialize",
        "agent_sdk.admin.whoami.missing_user_id",
        "agent_sdk.admin.whoami.ndjson_unsupported",
        "agent_sdk.admin.whoami.service_account_id",
        "agent_sdk.admin.whoami.team_id",
        "agent_sdk.admin.whoami.team_name",
        "agent_sdk.admin.whoami.user_id",
        "agent.output.upload_artifact.current_conversation_not_synced",
        "agent_sdk.api_key.confirm.expire",
        "agent_sdk.api_key.confirm.expire_cancelled",
        "agent_sdk.api_key.confirm.expire_help",
        "agent_sdk.api_key.error.create_failed",
        "agent_sdk.api_key.error.expiration_behavior_required",
        "agent_sdk.api_key.error.expiration_too_large",
        "agent_sdk.api_key.error.expire_failed",
        "agent_sdk.api_key.error.expire_non_interactive_requires_force",
        "agent_sdk.api_key.error.multiple_matches_specify_uid",
        "agent_sdk.api_key.error.not_found",
        "agent_sdk.api_key.output.created",
        "agent_sdk.api_key.output.expired",
        "agent_sdk.api_key.output.key_summary",
        "agent_sdk.api_key.output.multiple_matches",
        "agent_sdk.api_key.output.not_expired",
        "agent_sdk.api_key.output.raw_api_key",
        "agent_sdk.api_key.output.secret_shown_once",
        "agent_sdk.api_key.output.uid",
        "agent_sdk.api_key.prompt.select_key_to_expire",
        "agent_sdk.api_key.table.created",
        "agent_sdk.api_key.table.expires_at",
        "agent_sdk.api_key.table.key",
        "agent_sdk.api_key.table.last_used",
        "agent_sdk.api_key.table.name",
        "agent_sdk.api_key.table.never",
        "agent_sdk.api_key.table.scope",
        "agent_sdk.api_key.table.uid",
        "agent_sdk.artifact.error.get_failed",
        "agent_sdk.artifact.output.artifact_type",
        "agent_sdk.artifact.output.artifact_uid",
        "agent_sdk.artifact.output.content_type",
        "agent_sdk.artifact.output.created_at",
        "agent_sdk.artifact.output.description",
        "agent_sdk.artifact.output.download_text_header",
        "agent_sdk.artifact.output.download_url",
        "agent_sdk.artifact.output.downloaded",
        "agent_sdk.artifact.output.expires_at",
        "agent_sdk.artifact.output.filename",
        "agent_sdk.artifact.output.filepath",
        "agent_sdk.artifact.output.get_text_header",
        "agent_sdk.artifact.output.mime_type",
        "agent_sdk.artifact.output.path",
        "agent_sdk.artifact.output.size_bytes",
        "agent_sdk.artifact.output.upload_text_header",
        "agent_sdk.artifact.output.uploaded",
        "agent_sdk.artifact_upload.error.confirm_upload_failed",
        "agent_sdk.artifact_upload.error.conversation_not_cloud_task",
        "agent_sdk.artifact_upload.error.conversation_not_found",
        "agent_sdk.artifact_upload.error.conversation_resolution_required",
        "agent_sdk.artifact_upload.error.create_upload_target_failed",
        "agent_sdk.artifact_upload.error.env_run_id_missing",
        "agent_sdk.artifact_upload.error.env_run_id_not_unicode",
        "agent_sdk.artifact_upload.error.file_size_supported_range",
        "agent_sdk.artifact_upload.error.invalid_oz_run_id",
        "agent_sdk.artifact_upload.error.invalid_run_id",
        "agent_sdk.artifact_upload.error.load_conversation_for_headers",
        "agent_sdk.artifact_upload.error.multiple_conversations",
        "agent_sdk.artifact_upload.error.open_artifact_file",
        "agent_sdk.artifact_upload.error.read_artifact_file",
        "agent_sdk.artifact_upload.error.resolve_association_for_conversation_failed",
        "agent_sdk.artifact_upload.error.resolve_association_missing_source",
        "agent_sdk.artifact_upload.error.stat_artifact_file",
        "agent_sdk.cli.error.claude_auth_secret_requires_claude",
        "agent_sdk.cli.error.determine_working_directory",
        "agent_sdk.cli.error.invalid_value",
        "agent_sdk.cli.error.opencode_local_only",
        "agent_sdk.cli.error.resolve_working_directory",
        "agent_sdk.cli.error.unexpected_argument",
        "agent_sdk.cli.warning.team_api_key_free_credits",
        "agent_sdk.common.error.check_warp_logs",
        "agent_sdk.common.error.conversation_not_found_or_not_accessible",
        "agent_sdk.common.error.feature_not_enabled",
        "agent_sdk.common.error.invalid_ambient_task_id",
        "agent_sdk.common.error.invalid_oz_run_id",
        "agent_sdk.common.error.invalid_run_id",
        "agent_sdk.common.error.unknown_model_id",
        "agent_sdk.common.error.user_not_logged_in",
        "agent_sdk.common.error.user_not_on_team",
        "agent_sdk.common.error.warp_drive_sync_timeout",
        "agent_sdk.common.error.workspace_metadata_timeout",
        "agent_sdk.common.saved_prompt_summary",
        "agent_sdk.driver.error.team_metadata_refresh_timeout",
        "agent_sdk.driver.output.addressed_comments",
        "agent_sdk.driver.output.audio_content",
        "agent_sdk.driver.output.cancelled",
        "agent_sdk.driver.output.call_mcp_tool_failed",
        "agent_sdk.driver.output.codebase",
        "agent_sdk.driver.output.codebase_search_results",
        "agent_sdk.driver.output.command_completed",
        "agent_sdk.driver.output.command_denylisted",
        "agent_sdk.driver.output.command_finished",
        "agent_sdk.driver.output.command_still_running",
        "agent_sdk.driver.output.command_still_running_named",
        "agent_sdk.driver.output.command_write_failed",
        "agent_sdk.driver.output.completed_todos",
        "agent_sdk.driver.output.computer_use_action",
        "agent_sdk.driver.output.conversation_started",
        "agent_sdk.driver.output.created_plan",
        "agent_sdk.driver.output.created_pr",
        "agent_sdk.driver.output.current_directory",
        "agent_sdk.driver.output.editing_files",
        "agent_sdk.driver.output.fetch_conversation_error",
        "agent_sdk.driver.output.fetched_conversation",
        "agent_sdk.driver.output.fetched_web_pages",
        "agent_sdk.driver.output.fetching_conversation",
        "agent_sdk.driver.output.fetching_web_pages",
        "agent_sdk.driver.output.file_artifact_uploaded",
        "agent_sdk.driver.output.file_edits_failed",
        "agent_sdk.driver.output.files_updated_deleted",
        "agent_sdk.driver.output.find_failed",
        "agent_sdk.driver.output.finding_files",
        "agent_sdk.driver.output.grep_failed",
        "agent_sdk.driver.output.grepping_for_in",
        "agent_sdk.driver.output.image_content",
        "agent_sdk.driver.output.mcp_tool_call",
        "agent_sdk.driver.output.open_in_oz",
        "agent_sdk.driver.output.read_files_failed",
        "agent_sdk.driver.output.read_mcp_resource_failed",
        "agent_sdk.driver.output.reading",
        "agent_sdk.driver.output.reading_mcp_resource_name",
        "agent_sdk.driver.output.reading_mcp_resource_uri",
        "agent_sdk.driver.output.reading_skill",
        "agent_sdk.driver.output.received_agent_events",
        "agent_sdk.driver.output.received_messages",
        "agent_sdk.driver.output.requesting_computer_use",
        "agent_sdk.driver.output.run_id",
        "agent_sdk.driver.output.running_command",
        "agent_sdk.driver.output.search_codebase_failed",
        "agent_sdk.driver.output.searched_web_for",
        "agent_sdk.driver.output.searching_codebase",
        "agent_sdk.driver.output.searching_web",
        "agent_sdk.driver.output.searching_web_for",
        "agent_sdk.driver.output.sending_message_to",
        "agent_sdk.driver.output.sharing_session_at",
        "agent_sdk.driver.output.screenshot_captured",
        "agent_sdk.driver.output.skill_read",
        "agent_sdk.driver.output.skill_read_error",
        "agent_sdk.driver.output.skill_read_successfully",
        "agent_sdk.driver.output.starting_agent",
        "agent_sdk.driver.output.starting_recording",
        "agent_sdk.driver.output.stopping_recording",
        "agent_sdk.driver.output.updated_todo_list",
        "agent_sdk.driver.output.upload_artifact_failed",
        "agent_sdk.driver.output.uploaded_artifact",
        "agent_sdk.driver.output.uploaded_artifact_from",
        "agent_sdk.driver.output.uploading_artifact",
        "agent_sdk.driver.output.use_computer_error",
        "agent_sdk.driver.output.web_fetch_failed",
        "agent_sdk.driver.output.web_search_failed_for",
        "agent_sdk.driver.output.write_bytes_to_command",
        "agent_sdk.driver.error.repository_index_failed",
        "agent_sdk.driver.error.repository_index_pending",
        "agent_sdk.driver.error.repository_not_found",
        "agent_sdk.driver.snapshot.error.no_upload_target",
        "agent_sdk.driver.snapshot.error.read_file",
        "agent_sdk.ambient.error.attachment_upload_not_enabled",
        "agent_sdk.ambient.error.env_var_not_unicode",
        "agent_sdk.ambient.error.not_saved_prompt",
        "agent_sdk.ambient.error.open_agent_event_stream",
        "agent_sdk.ambient.error.parse_saved_prompt_id",
        "agent_sdk.ambient.error.prompt_skill_or_conversation_required",
        "agent_sdk.ambient.error.saved_prompt_not_found",
        "agent_sdk.ambient.error.streaming_requires_ndjson",
        "agent_sdk.ambient.error.too_many_attachments",
        "agent_sdk.ambient.error.unexpected_skill_argument",
        "agent_sdk.ambient.error.unsupported_feature",
        "agent_sdk.ambient.field.config",
        "agent_sdk.ambient.field.created",
        "agent_sdk.ambient.field.executed_as",
        "agent_sdk.ambient.field.session",
        "agent_sdk.ambient.field.status",
        "agent_sdk.ambient.field.title",
        "agent_sdk.ambient.message.body",
        "agent_sdk.ambient.message.delivered_at",
        "agent_sdk.ambient.message.from",
        "agent_sdk.ambient.message.marked_delivered",
        "agent_sdk.ambient.message.message_id",
        "agent_sdk.ambient.message.message_ids",
        "agent_sdk.ambient.message.read_at",
        "agent_sdk.ambient.message.sent_at",
        "agent_sdk.ambient.message.sent_count",
        "agent_sdk.ambient.message.subject",
        "agent_sdk.ambient.message_table.delivered_at",
        "agent_sdk.ambient.message_table.from",
        "agent_sdk.ambient.message_table.message_id",
        "agent_sdk.ambient.message_table.read_at",
        "agent_sdk.ambient.message_table.sent_at",
        "agent_sdk.ambient.message_table.subject",
        "agent_sdk.ambient.output.agent_run",
        "agent_sdk.ambient.output.agent_runs",
        "agent_sdk.ambient.output.agent_state",
        "agent_sdk.ambient.output.concurrent_limit_reached",
        "agent_sdk.ambient.output.error_message",
        "agent_sdk.ambient.output.no_environment",
        "agent_sdk.ambient.output.no_runs_found",
        "agent_sdk.ambient.output.run_failed_no_message",
        "agent_sdk.ambient.output.session_not_ready",
        "agent_sdk.ambient.output.spawned_run",
        "agent_sdk.ambient.output.upgrade_plan",
        "agent_sdk.ambient.output.view_agent_session",
        "agent_sdk.ambient.output.view_run",
        "agent_sdk.ambient.state.blocked",
        "agent_sdk.ambient.state.cancelled",
        "agent_sdk.ambient.state.claimed",
        "agent_sdk.ambient.state.failed",
        "agent_sdk.ambient.state.in_progress",
        "agent_sdk.ambient.state.queued",
        "agent_sdk.ambient.state.succeeded",
        "agent_sdk.ambient.state.unknown",
        "agent_sdk.ambient.watch.disconnected",
        "agent_sdk.ambient.watch.hydrate_failed",
        "agent_sdk.ambient.watch.reconnect_failed",
        "agent_sdk.ambient.watch.reconnected",
        "agent_sdk.ambient.watch.skipping_event_without_ref",
        "agent_sdk.ambient.watch.skipping_malformed_event",
        "agent_sdk.ambient.watch.stream_closed",
        "agent_sdk.agent_config.output.authorization_required",
        "agent_sdk.agent_config.output.fetching_from_environments",
        "agent_sdk.agent_config.output.no_agents_found",
        "agent_sdk.agent_config.output.opening_browser",
        "agent_sdk.agent_config.output.rerun_after_authorizing",
        "agent_sdk.environment.custom_image",
        "agent_sdk.environment.error.no_images",
        "agent_sdk.environment.output.created",
        "agent_sdk.environment.output.deleted",
        "agent_sdk.environment.output.updated",
        "agent_sdk.environment.select_base_image",
        "agent_sdk.federate.error.subject_template_required",
        "agent_sdk.federate.error.write_gcp_token",
        "agent_sdk.federate.output.expires_at",
        "agent_sdk.federate.output.issuer",
        "agent_sdk.federate.output.token",
        "agent_sdk.integration.create.canceled",
        "agent_sdk.integration.error.oauth_failed",
        "agent_sdk.integration.output.none_found",
        "agent_sdk.integration.status.active",
        "agent_sdk.harness_support.error.shutdown_error_args_required",
        "agent_sdk.harness_support.output.artifact_reported",
        "agent_sdk.harness_support.output.notification_sent",
        "agent_sdk.harness_support.output.shutdown_reported",
        "agent_sdk.harness_support.output.task_finished",
        "agent_sdk.memory_store.agent_table.access",
        "agent_sdk.memory_store.agent_table.instructions",
        "agent_sdk.memory_store.agent_table.name",
        "agent_sdk.memory_store.agent_table.uid",
        "agent_sdk.memory_store.memory_table.content",
        "agent_sdk.memory_store.memory_table.created",
        "agent_sdk.memory_store.memory_table.source",
        "agent_sdk.memory_store.memory_table.uid",
        "agent_sdk.memory_store.memory_table.updated",
        "agent_sdk.memory_store.memory_table.version",
        "agent_sdk.memory_store.output.created_memory",
        "agent_sdk.memory_store.output.deleted_memory",
        "agent_sdk.memory_store.output.no_agents",
        "agent_sdk.memory_store.output.no_memories",
        "agent_sdk.memory_store.output.no_stores",
        "agent_sdk.memory_store.output.no_versions",
        "agent_sdk.memory_store.output.updated_memory",
        "agent_sdk.memory_store.output.updated_store",
        "agent_sdk.memory_store.output_table.memory_id",
        "agent_sdk.memory_store.output_table.version_id",
        "agent_sdk.memory_store.table.created",
        "agent_sdk.memory_store.table.description",
        "agent_sdk.memory_store.table.owner_type",
        "agent_sdk.memory_store.table.owner_uid",
        "agent_sdk.memory_store.table.uid",
        "agent_sdk.memory_store.table.updated",
        "agent_sdk.memory_store.version_table.content",
        "agent_sdk.memory_store.version_table.created",
        "agent_sdk.memory_store.version_table.reason",
        "agent_sdk.memory_store.version_table.uid",
        "agent_sdk.memory_store.version_table.version",
        "agent_sdk.model.table.model_id",
        "agent_sdk.provider.error.scope_required",
        "agent_sdk.provider.output.authenticate_url",
        "agent_sdk.provider.status.not_connected",
        "agent_sdk.provider.table.allowed_for",
        "agent_sdk.provider.table.name",
        "agent_sdk.provider.table.slug",
        "agent_sdk.provider.table.status",
        "agent_sdk.schedule.detail.agent_name",
        "agent_sdk.schedule.detail.cron_schedule",
        "agent_sdk.schedule.detail.environment_id",
        "agent_sdk.schedule.detail.host",
        "agent_sdk.schedule.detail.last_error",
        "agent_sdk.schedule.detail.last_ran",
        "agent_sdk.schedule.detail.model_id",
        "agent_sdk.schedule.detail.name",
        "agent_sdk.schedule.detail.next_run",
        "agent_sdk.schedule.detail.paused",
        "agent_sdk.schedule.detail.prompt",
        "agent_sdk.schedule.detail.skill",
        "agent_sdk.schedule.error.not_found",
        "agent_sdk.schedule.field.agent_name",
        "agent_sdk.schedule.field.cron_schedule",
        "agent_sdk.schedule.field.environment_id",
        "agent_sdk.schedule.field.host",
        "agent_sdk.schedule.field.last_error",
        "agent_sdk.schedule.field.last_ran",
        "agent_sdk.schedule.field.model_id",
        "agent_sdk.schedule.field.name",
        "agent_sdk.schedule.field.next_run",
        "agent_sdk.schedule.field.paused",
        "agent_sdk.schedule.field.prompt",
        "agent_sdk.schedule.field.skill",
        "agent_sdk.schedule.output.deleted",
        "agent_sdk.schedule.output.no_environment",
        "agent_sdk.schedule.output.paused",
        "agent_sdk.schedule.output.scheduled_agent",
        "agent_sdk.schedule.output.unpaused",
        "agent_sdk.schedule.output.updated",
        "agent_sdk.schedule.progress.deleting_agent",
        "agent_sdk.schedule.progress.pausing_agent",
        "agent_sdk.schedule.progress.resuming_agent",
        "agent_sdk.schedule.progress.scheduling_agent",
        "agent_sdk.schedule.progress.updating_agent",
        "agent_sdk.schedule.table.id",
        "agent_sdk.schedule.table.last_ran",
        "agent_sdk.schedule.table.name",
        "agent_sdk.schedule.table.next_run",
        "agent_sdk.schedule.table.paused",
        "agent_sdk.schedule.table.schedule",
        "agent_sdk.schedule.table.scope",
        "agent_sdk.secret.confirm.delete",
        "agent_sdk.secret.confirm.delete_cancelled",
        "agent_sdk.secret.confirm.delete_help",
        "agent_sdk.secret.error.bedrock_access_key_non_interactive_required",
        "agent_sdk.secret.error.bedrock_access_key_update_value",
        "agent_sdk.secret.error.bedrock_api_key_update_value",
        "agent_sdk.secret.error.bedrock_non_interactive_required",
        "agent_sdk.secret.error.delete_non_interactive_requires_force",
        "agent_sdk.secret.error.name_required",
        "agent_sdk.secret.error.not_found",
        "agent_sdk.secret.error.read_value_file_failed",
        "agent_sdk.secret.output.created",
        "agent_sdk.secret.output.deleted",
        "agent_sdk.secret.output.updated",
        "agent_sdk.secret.prompt.aws_access_key_id",
        "agent_sdk.secret.prompt.aws_region",
        "agent_sdk.secret.prompt.aws_secret_access_key",
        "agent_sdk.secret.prompt.aws_session_token_optional",
        "agent_sdk.secret.prompt.bedrock_api_key",
        "agent_sdk.secret.prompt.openai_base_url",
        "agent_sdk.secret.prompt.openai_base_url_help",
        "agent_sdk.secret.prompt.secret_value",
        "agent_sdk.secret.scope.personal",
        "agent_sdk.secret.scope.team",
        "agent_sdk.secret.table.created",
        "agent_sdk.secret.table.name",
        "agent_sdk.secret.table.scope",
        "agent_sdk.secret.table.type",
        "agent_sdk.secret.table.updated",
        "agent_sdk.secret.type.anthropic_api_key",
        "agent_sdk.secret.type.anthropic_bedrock_access_key",
        "agent_sdk.secret.type.anthropic_bedrock_api_key",
        "agent_sdk.secret.type.dotenvx",
        "agent_sdk.secret.type.openai_api_key",
        "agent_sdk.secret.type.raw_value",
        "agent_sdk.skill.error.ambiguous",
        "agent_sdk.skill.error.clone_failed",
        "agent_sdk.skill.error.not_found",
        "agent_sdk.skill.error.org_mismatch",
        "agent_sdk.skill.error.parse_failed",
        "agent_sdk.skill.error.repo_not_found",
    ];

    assert_bundled_keys_exist(&required_keys);

    let path = workspace_root().join("app/src/ai/agent_sdk/driver/output.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let text_module = content
        .split("pub mod json {")
        .next()
        .expect("driver output should contain text module before json module");
    let snippets = [
        "Command was not allowed to run",
        "Failed to write to command",
        "Updated TODO list:",
        "Searching web",
        "Created PR:",
        "Screenshot captured",
        "New conversation started",
        "Open in Oz",
        "Sharing session at",
        "Created plan",
        "For more information, check Warp logs",
        "Saved prompt (",
    ];
    let mut violations = snippets
        .into_iter()
        .filter(|snippet| text_module.contains(snippet))
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let cases = [
        (
            "app/src/ai/agent_sdk/driver.rs",
            &[
                "Repository indexing is still pending",
                "Repository indexing failed",
                "Repository not found",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/ambient.rs",
            &[
                "Unsupported feature",
                "unexpected argument '--skill' found",
                "Either --prompt, --skill, or --conversation must be provided",
                "Failed to parse saved prompt ID",
                "is not a saved prompt",
                "Saved prompt with ID",
                "Too many attachments.",
                "Attachment upload is not enabled",
                "Agent will run without an environment.",
                "Spawned ambient agent with run ID:",
                "View run:",
                "Concurrent cloud agent limit reached.",
                "To increase your concurrent agent limit",
                "Agent state:",
                "Run failed with no error message",
                "View agent session:",
                "Agent session with run ID",
                "No runs found.",
                "Agent Run:",
                "Agent Runs (",
                "Executed as:",
                "Config:\\n",
                "Created:",
                "Session:",
                "Streaming commands require `--output-format ndjson`",
                "is set but is not valid Unicode",
                "Failed to open agent event stream",
                "Message watch reconnect failed",
                "Skipping malformed agent event payload",
                "Skipping new_message event without ref_id",
                "Failed to hydrate message",
                "Message watch disconnected",
                "Message watch stream closed",
                "Sent {count} message(s).",
                "Message IDs:",
                "Message ID:",
                "From:",
                "Subject:",
                "Sent At:",
                "Delivered At:",
                "Read At:",
                "Body:",
                "Marked message delivered:",
                "MESSAGE ID",
                "DELIVERED AT",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/harness_support.rs",
            &[
                "This feature is not enabled",
                "Artifact reported:",
                "Notification sent.",
                "Task finished.",
                "Shutdown reported.",
                "--error-category and --error-message must be provided together",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/mod.rs",
            &[
                "Warning: Free cloud credits apply to personal runs only",
                "invalid value 'environment'",
                "invalid value 'api-key'",
                "unexpected argument '--environment' found",
                "unexpected argument '--conversation' found",
                "unexpected argument '--harness' found",
                "The opencode harness is only supported",
                "--claude-auth-secret is only valid with --harness claude.",
                "Skill '{skill}' not found",
                "Repository '{repo}' not found",
                "Failed to parse skill file",
                "Failed to clone repository",
                "Unable to determine working directory",
                "Unable to resolve {}",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/common.rs",
            &[
                "Unknown model id",
                "User is not on a team",
                "User should be logged in",
                "Timed out refreshing team metadata",
                "Timed out waiting for Warp Drive to sync",
                "conversation {conversation_id} not found or not accessible",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/memory_store.rs",
            &[
                "Updated store {}.",
                "No agents attached to this store.",
                "No versions found.",
                "Deleted memory {}.",
                "No memory stores found.",
                "No memories found.",
                "Updated memory {}.",
                "Created memory {}.",
                "Owner Type",
                "Memory ID",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/model.rs",
            &["MODEL ID", "Timed out refreshing workspace metadata"][..],
        ),
        (
            "app/src/ai/agent_sdk/federate.rs",
            &[
                "This feature is not enabled",
                "--subject-template requires at least one value",
                "Token:",
                "Expires at:",
                "Issuer:",
                "Error writing GCP token",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/artifact.rs",
            &[
                "Failed to get artifact",
                "Artifact downloaded",
                "Artifact uploaded",
                "Artifact UID:",
                "Artifact type:",
                "Created at:",
                "Download URL:",
                "Expires at:",
                "Content type:",
                "Filepath:",
                "Filename:",
                "Description:",
                "MIME type:",
                "Size bytes:",
                "Artifact UID\\t",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/artifact_upload.rs",
            &[
                "Failed to open artifact file",
                "Failed to stat artifact file",
                "Failed to read artifact file",
                "Conversation not found",
                "Multiple conversations found",
                "is not backed by a cloud agent task",
                "is set but is not valid Unicode",
                "conversation resolution should be provided",
                "Failed to resolve artifact upload association",
                "Artifact file size exceeds supported range",
                "Failed to create file artifact upload target",
                "Failed to confirm file artifact upload",
            ][..],
        ),
        (
            "app/src/ai/blocklist/action_model/execute/upload_artifact.rs",
            &[
                "Artifact upload failed:",
                "Current conversation has not been synced to the server yet",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/agent_config.rs",
            &[
                "No agents found.",
                "Fetching agent skills from your Warp environments",
                "Authorization required for private repository access.",
                "Opening browser for GitHub authorization",
                "After authorizing, please re-run this command.",
                "Cannot access private repo",
                "User not connected to GitHub",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/environment.rs",
            &[
                "Custom Docker image",
                "No Warp dev images available.",
                "Select a base image:",
                "Enter custom Docker image name:",
                "Environment created successfully",
                "Environment deleted successfully",
                "Environment updated successfully",
                "Failed to update environment",
                "Failed to delete environment",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/integration.rs",
            &[
                "Integration creation canceled.",
                "Creating integration without an environment.",
                "OAuth authorization failed.",
                "OAuth authorization expired.",
                "Unexpected non-terminal OAuth status returned",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/integration_output.rs",
            &["No integrations found.", "Integration:", "Integrations:"][..],
        ),
        (
            "app/src/ai/agent_sdk/admin.rs",
            &[
                "You are already logged in",
                "Logged in successfully",
                "To log in, open this URL in your browser",
                "To log in, visit ",
                "Could not determine user ID. Are you logged in?",
                "User ID:",
                "Service account ID:",
                "Display Name:",
                "Email:",
                "Team ID:",
                "Team Name:",
                "`whoami` does not support `--output-format ndjson`",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/api_key.rs",
            &[
                "API key '{key_identifier}' not found",
                "Multiple API keys match '{key_identifier}'",
                "Expiration cancelled",
                "Raw API key:",
                "This secret key is shown only once",
                "expiration behavior is required",
                "expiration duration is too large",
                "failed to create API key",
                "failed to expire API key",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/driver/snapshot.rs",
            &["Failed to read file", "no upload target returned by server"][..],
        ),
        (
            "app/src/ai/agent_sdk/driver/output.rs",
            &[
                "Command was not allowed to run due to presence on denylist",
                "Failed to write to command.",
                "Starting recording",
                "Stopping recording",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/provider.rs",
            &[
                "Provider '{}' must be setup",
                "To authenticate {slug}, open this URL in your browser",
                "User is not on a team",
                "ALLOWED FOR",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/schedule.rs",
            &[
                "Scheduling agent",
                "Scheduled agent:",
                "Pausing agent",
                "Schedule paused",
                "Resuming agent",
                "Schedule unpaused",
                "Updating agent",
                "Schedule updated",
                "Schedule not found",
                "Deleting agent",
                "Schedule deleted",
                "Cron schedule",
                "Last ran",
                "Next run",
                "Environment ID",
                "Model ID",
                "Agent name",
                "Unsynced",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/secret.rs",
            &[
                "Secret name is required",
                "Secret '{}' created",
                "Deletion cancelled",
                "This action cannot be undone",
                "Secret '{name}' deleted",
                "Secret '{}' updated",
                "Secret '{}' not found",
                "Failed to read secret value from",
                "Bedrock secrets require --bedrock-api-key",
                "Bedrock access key secrets require --access-key-id",
            ][..],
        ),
    ];
    violations.extend(selected_snippet_violations(&cases));

    assert!(
        violations.is_empty(),
        "agent SDK text output must use catalog copy: {violations:#?}"
    );
}

#[test]
fn warp_cli_help_copy_uses_catalog_keys() {
    let required_keys = [
        "cli.error.try_help",
        "cli.error.invalid_jq_filter",
        "cli.error.mcp_config_read_failed",
        "cli.error.mcp_spec_utf8",
        "cli.error.share_recipient_utf8",
        "cli.error.share_subject_invalid",
        "cli.error.environment_description_too_long",
        "cli.error.skill_identifier_empty",
        "cli.error.skill_organization_empty",
        "cli.error.skill_qualifier_empty",
        "cli.error.skill_repository_empty",
        "cli.error.skill_specifier_empty",
        "cli.error.unrecognized_subcommand",
        "cli.help.after_help",
        "cli.help.command.warpctrl.after_help",
        "cli.help.command.oz.agent.about",
        "cli.help.command.oz.agent.create.about",
        "cli.help.command.oz.agent.delete.about",
        "cli.help.command.oz.agent.get.about",
        "cli.help.command.oz.agent.list.about",
        "cli.help.command.oz.agent.profile.about",
        "cli.help.command.oz.agent.run.about",
        "cli.help.command.oz.agent.run-cloud.about",
        "cli.help.command.oz.agent.skills.about",
        "cli.help.command.oz.agent.update.about",
        "cli.help.command.oz.api-key.about",
        "cli.help.command.oz.artifact.about",
        "cli.help.command.oz.about",
        "cli.help.command.oz.arg.debug.help",
        "cli.help.command.oz.arg.output_format.help",
        "cli.help.command.oz.environment.about",
        "cli.help.command.oz.federate.about",
        "cli.help.command.oz.integration.about",
        "cli.help.command.oz.login.about",
        "cli.help.command.oz.logout.about",
        "cli.help.command.oz.mcp.about",
        "cli.help.command.oz.model.about",
        "cli.help.command.oz.provider.about",
        "cli.help.command.oz.run.about",
        "cli.help.command.oz.schedule.about",
        "cli.help.command.oz.schedule.create.about",
        "cli.help.command.oz.schedule.delete.about",
        "cli.help.command.oz.schedule.get.about",
        "cli.help.command.oz.schedule.list.about",
        "cli.help.command.oz.schedule.pause.about",
        "cli.help.command.oz.schedule.unpause.about",
        "cli.help.command.oz.schedule.update.about",
        "cli.help.command.oz.secret.about",
        "cli.help.command.oz.whoami.about",
        "cli.help.heading.options",
        "cli.help.heading.subcommands",
        "cli.help.template",
        "cli.help.value.mcp_spec.json",
        "cli.help.value.mcp_spec.path",
        "cli.help.value.share.public_edit",
        "cli.help.value.share.public_view",
        "cli.help.value.share.team_edit",
        "cli.help.value.share.team_view",
        "cli.help.value.share.user_edit",
        "cli.help.value.share.user_view",
    ];

    assert_bundled_keys_exist(&required_keys);

    let cases = [
        (
            "crates/warp_cli/src/json_filter.rs",
            &["invalid jq filter"][..],
        ),
        (
            "crates/warp_cli/src/environment.rs",
            &["Description must be at most"][..],
        ),
        (
            "crates/warp_cli/src/mcp.rs",
            &[
                "Invalid UTF-8 in MCP spec",
                "Failed to read MCP config file",
                "Path to a JSON file containing MCP config",
                "Inline JSON MCP server configuration",
            ][..],
        ),
        (
            "crates/warp_cli/src/share.rs",
            &[
                "Invalid share recipient",
                "Share with your team",
                "Share with anyone who has the link",
                "Share with <user@email.com>",
                "Cannot share with",
            ][..],
        ),
        (
            "crates/warp_cli/src/skill.rs",
            &[
                "Skill specifier cannot be empty",
                "Qualifier cannot be empty in 'repo:skill_identifier' format",
                "Skill identifier cannot be empty",
                "Organization cannot be empty",
                "Repository name cannot be empty",
            ][..],
        ),
        (
            "crates/warp_cli/src/agent.rs",
            &[
                "--claude-auth-secret is only valid with --harness claude.",
                "--codex-auth-secret is only valid with --harness codex.",
            ][..],
        ),
    ];

    let violations = selected_snippet_violations(&cases);

    assert!(
        violations.is_empty(),
        "warp CLI help and parser copy must use catalog keys: {violations:#?}"
    );
}

#[test]
fn selected_accessibility_and_fallback_surfaces_do_not_use_direct_english_literals() {
    let cases = [
        (
            "app/src/workspaces/team.rs",
            &[
                "Your team cannot be deleted with an active subscription.",
                "Your team cannot be deleted with unused add-on credits.",
                "Your team cannot be deleted with other team members.",
            ][..],
        ),
        (
            "app/src/ai/blocklist/suggested_agent_mode_workflow_modal.rs",
            &["const SUGGESTED_PROMPT_MODAL_HEADER", "Some(\"Prompt\""][..],
        ),
        (
            "app/src/ai_assistant/panel.rs",
            &[
                "Write a script to connect to an AWS EC2 instance.",
                "How do I undo the most recent commits in git?",
                "How do I find all files containing specific text?",
            ][..],
        ),
        (
            "app/src/ai_assistant/transcript.rs",
            &[
                "How do I fix this?",
                "Show examples.",
                "What should I do next?",
            ][..],
        ),
        (
            "app/src/terminal/local_tty/terminal_manager.rs",
            &[
                "const ACL_UPDATE_FAILURE_RESPONSE",
                "Failed to update permissions for shared session",
            ][..],
        ),
        (
            "app/src/terminal/shared_session/viewer/terminal_manager.rs",
            &[
                "\"Something went wrong. Please try again.\".to_owned()",
                "\"Failed to update permissions for shared session\".to_owned()",
            ][..],
        ),
        (
            "app/src/search/command_palette/files/search_item.rs",
            &[
                "Directory: {}",
                "File: {}",
                "Press Enter to navigate to this directory",
                "Press Enter to open this file",
            ][..],
        ),
        (
            "app/src/env_vars/view/env_var_collection.rs",
            &["\"Untitled\""][..],
        ),
        (
            "app/src/workspace/view/vertical_tabs.rs",
            &[
                "\"New Group\".to_string()",
                "\"1 tab\".to_string()",
                "format!(\"{member_count} tabs\")",
            ][..],
        ),
        (
            "app/src/workspace/view/global_search/view.rs",
            &[
                "\"Searching…\".to_string()",
                "\"No results found. Review your gitignore files.\".to_string()",
                "format!(\"1 result in {files} {file_word}\")",
                "format!(\"{n} results in {files} {file_word}\")",
            ][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/agent_view_block.rs",
            &["Couldn't navigate to conversation."][..],
        ),
        (
            "app/src/ai/blocklist/block/view_impl/output.rs",
            &[
                "\"Recording computer-use session\".to_string()",
                "primary: \"Recording started\".to_string()",
                "primary: \"Recording failed to start\".to_string()",
                "primary: \"Recording cancelled\".to_string()",
                "primary: \"Starting recording\".to_string()",
                "format!(\"Partial recording",
                "primary: \"Recording saved\".to_string()",
                "primary: \"Recording could not be saved\".to_string()",
                "primary: \"Saving recording\".to_string()",
                "RecordingSpanStatus::Active => \"Recording active\"",
                "RecordingSpanStatus::Captured => \"Captured in recording\"",
                "\"Open recording\",",
                "\"View screenshot\",",
            ][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/orchestration_conversation_links.rs",
            &[
                "\"Parent conversation\".to_string()",
                "\"Back to parent conversation\".to_string()",
            ][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/orchestration_pill_bar.rs",
            &[
                "\"Focus pane\"",
                "\"Open in new pane\"",
                "\"Open in new tab\"",
                "\"View in Oz\"",
                "\"Stop agent\"",
                "\"Delete agent\"",
                "\"Kill agent\"",
                "unwrap_or(\"Agent\")",
                "\"Orchestrator\".to_string()",
                "name.push_str(\"Untitled\")",
            ][..],
        ),
        (
            "app/src/drive/export.rs",
            &["\"Untitled\".to_string()", "name.push_str(\"Untitled\")"][..],
        ),
        (
            "app/src/cloud_object/mod.rs",
            &[
                "\"Personal\".to_string()",
                "\"Team\".to_string()",
                "\"Shared with me\".to_string()",
            ][..],
        ),
        (
            "app/src/ai/blocklist/block/view_impl/orchestration.rs",
            &["unwrap_or(\"Agent\")"][..],
        ),
        (
            "app/src/ai/blocklist/block/view_impl.rs",
            &[
                "Manage AI Autonomy permissions",
                "String::from(\"Untitled\")",
                "String::from(\"Warp Docs\")",
                "String::from(\"Memory\")",
            ][..],
        ),
        (
            "app/src/ai/blocklist/code_block.rs",
            &[
                "\"Add as Context\"",
                "\"Copy\"",
                "\"Open in Warp\"",
                "\"Run in terminal\"",
            ][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/host_picker.rs",
            &[
                "const CUSTOM_HOST_LABEL:",
                "const DEFAULT_BADGE:",
                "const EDITOR_PLACEHOLDER:",
                "\"Custom host",
                "\"Default\"",
                "concat!(\"Custom \",",
                "concat!(\"Def\",",
                "concat!(\"Conn\",",
                "concat!(\"Disconn\",",
            ][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/orchestration_controls.rs",
            &[
                "OpenCode is not supported on Cloud",
                "Disabled by your administrator",
                "Install Claude Code to use this local harness",
                "We recommend selecting an environment for cloud agents",
                "We recommend creating an environment for cloud agents",
            ][..],
        ),
        (
            "app/src/ai/blocklist/usage/rollup.rs",
            &["\"Orchestrator\".to_string()", "\"Agent\".to_string()"][..],
        ),
        (
            "app/src/ai/blocklist/usage/conversation_usage_view.rs",
            &[
                "\"USAGE SUMMARY\"",
                "\"Credits spent (last response)\"",
                "\"Credits spent (total)\"",
                "\"Credits spent\"",
                "\"Tool calls\"",
                "\"Models\"",
                "format!(\"Models ({})\"",
                "which model is used for full terminal use",
                "\"Context window used\"",
                "\"TOOL CALL SUMMARY\"",
                "\"Files changed\"",
                "\"Diffs applied\"",
                "\"Commands executed\"",
                "\"LAST RESPONSE TIME\"",
                "\"Time to first token\"",
                "\"{:.1} seconds\"",
                "\"Total agent response time\"",
                "\"Total time (including tool calls)\"",
                "\"Hide details\"",
                "\"View details\"",
                "format!(\"Show {hidden_count} more\")",
            ][..],
        ),
        (
            "app/src/ai/conversation_details_panel.rs",
            &["title: \"Cloud agent run\""][..],
        ),
        (
            "app/src/ai/blocklist/view_util.rs",
            &[
                "format!(\"{whole} credit\")",
                "format!(\"{whole} credits\")",
            ][..],
        ),
        (
            "app/src/ai/blocklist/suggestion_chip_view.rs",
            &["format!(\"Suggested prompt:"][..],
        ),
        (
            "app/src/ai/blocklist/prompt/plan_and_todo_list.rs",
            &[
                "\"Agent is unaware of recent plan edits\".to_string()",
                "\"View plan\".to_string()",
                "\"View todo list\".to_string()",
            ][..],
        ),
        (
            "app/src/ai/blocklist/suggested_rule_modal.rs",
            &["\"Untitled\".to_string()"][..],
        ),
        (
            "app/src/ai/artifacts/buttons.rs",
            &[
                "make_screenshot_button(\"Screenshots\"",
                "\"Open plan\"",
                "\"Copy branch name\"",
                "\"Open pull request\"",
                "\"View screenshots\"",
                "\"Download file\"",
            ][..],
        ),
        (
            "app/src/workspace/bonus_grant_notification_model.rs",
            &["\"account\"", "\"team\"", "Reload Credits have been added"][..],
        ),
        (
            "app/src/resource_center/section_views/changelog_section.rs",
            &[
                "render_basic_changelog_header(&title",
                "render_special_changelog_header(&title",
            ][..],
        ),
        (
            "app/src/external_secrets/mod.rs",
            &[
                "CLI is not installed",
                "View {} CLI installation documentation",
                "Integrate 1Password app with CLI",
                "didn't return secrets",
                "Platform not supported",
            ][..],
        ),
        (
            "app/src/drive/items/env_var_collection.rs",
            &["\"Untitled\".to_string()"][..],
        ),
        (
            "app/src/drive/items/notebook.rs",
            &["\"Untitled\".to_string()"][..],
        ),
        (
            "app/src/drive/items/item.rs",
            &["\"Untitled\".to_string()"][..],
        ),
        (
            "app/src/search/command_search/env_var_collections/env_var_collection_search_item.rs",
            &["\"Untitled\".to_owned()"][..],
        ),
        (
            "app/src/search/command_palette/warp_drive/env_var_collection_search_item.rs",
            &["\"Untitled\".to_owned()", "\"Environment Variables: {}\""][..],
        ),
        (
            "app/src/search/command_palette/warp_drive/notebook_search_item.rs",
            &["\"Untitled\".to_string()"][..],
        ),
        ("app/src/terminal/view.rs", &["\"Untitled\".to_owned()"][..]),
        (
            "app/src/search/ai_context_menu/notebooks/search_item.rs",
            &["\"Untitled\".to_string()", "format!(\"Notebook: {}"][..],
        ),
        (
            "app/src/search/ai_context_menu/files/search_item.rs",
            &["format!(\"Directory: {}", "format!(\"File: {}"][..],
        ),
        (
            "app/src/search/command_search/notebooks/notebook_search_item.rs",
            &["format!(\"Notebook: {}"][..],
        ),
        (
            "app/src/search/notebook_embedding/notebooks/notebook_search_item.rs",
            &["format!(\"Notebook: {}"][..],
        ),
        (
            "app/src/notebooks/notebook.rs",
            &["title.push_str(\"Untitled\")", "format!(\"{} notebook\""][..],
        ),
        (
            "app/src/notebooks/file/mod.rs",
            &["\"Untitled\".to_string()", "format!(\"{} notebook\""][..],
        ),
        (
            "app/src/notebooks/editor/block_insertion_menu.rs",
            &["\"Untitled\".to_string()"][..],
        ),
        (
            "app/src/settings_view/platform_page.rs",
            &[
                "\"Search API keys\"",
                "\"Save your key\"",
                "\"No API keys match your search\"",
            ][..],
        ),
        (
            "app/src/settings_view/platform/create_api_key_modal.rs",
            &[
                "\"Please select an agent.\"",
                "\"Unable to create a team API key because there is no current team.\"",
                "\"Failed to create API key. Please try again.\"",
            ][..],
        ),
        (
            "app/src/settings_view/main_page.rs",
            &[
                "\"Not yet loaded\"",
                "\"Refreshing",
                "format!(\"Loaded (refreshes in",
                "format!(\"Failed: {message}\")",
                "Using injected token (WARP_IAP_TOKEN)",
                "\"Staging IAP credentials\"",
                "\"Refresh\".into()",
            ][..],
        ),
        (
            "app/src/settings_view/warp_drive_page.rs",
            &["SettingActionPairDescriptions::new(\"Enable Warp Drive\""][..],
        ),
        (
            "app/src/settings_view/warpify_page.rs",
            &["\"SSH Warpification\""][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs",
            &["context remaining", "No plugin manager available"][..],
        ),
        (
            "app/src/terminal/cli_agent_sessions/plugin_manager/mod.rs",
            &[
                "Auto-install not supported for this agent",
                "Auto-update not supported for this agent",
                "format!(\"'{}' failed",
                "format!(\"failed to run",
            ][..],
        ),
        (
            "app/src/terminal/cli_agent_sessions/plugin_manager/claude.rs",
            &[
                "Plugin update did not take effect",
                "Platform plugin installation did not take effect",
                "Platform plugin update did not take effect",
            ][..],
        ),
        (
            "app/src/terminal/cli_agent_sessions/plugin_manager/gemini.rs",
            &["Plugin update did not take effect"][..],
        ),
        (
            "app/src/tab_configs/session_config.rs",
            &["New worktree branch name"][..],
        ),
        (
            "app/src/input_suggestions.rs",
            &[
                "format_approx_duration_from_now",
                "human_readable_approx_duration",
            ][..],
        ),
        (
            "app/src/ai_assistant/mod.rs",
            &["ASK_AI_ASSISTANT_TEXT", "\"Ask Warp AI\""][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/requested_command.rs",
            &["Manage command execution setting"][..],
        ),
        (
            "app/src/ai/blocklist/inline_action/requested_action.rs",
            &[
                "const REQUESTED_ACTION_CANCEL_LABEL: &str = \"Cancel\"",
                "const REQUESTED_ACTION_RUN_LABEL: &str = \"Run\"",
            ][..],
        ),
        (
            "app/src/env_vars/env_var_collection_block.rs",
            &[
                "const ENV_VAR_COLLECTION_CANCEL_LABEL: &str = \"Cancel\"",
                "const ENV_VAR_COLLECTION_ACCEPT_LABEL: &str = \"Run\"",
                "OK if I run this command and read the output?",
            ][..],
        ),
        (
            "app/src/cloud_object/mod.rs",
            &[
                "format!(\"Edited",
                "format!(\"{name} edited",
                "format!(\"Last edited by",
                "\"1 day until permanent deletion\".to_string()",
                "format!(\"{days_left} days until permanent deletion\")",
            ][..],
        ),
        (
            "app/src/terminal/view/init_project/mod.rs",
            &[
                "{} installed and enabled successfully.",
                "Failed to install {}: {e}",
                "Installing {} in background...",
            ][..],
        ),
        (
            "app/src/ai/persisted_workspace.rs",
            &[
                "{} installed and enabled successfully.",
                "Failed to install {}: {}",
                "Failed to start LSP server",
            ][..],
        ),
        (
            "app/src/settings_view/appearance_page.rs",
            &[
                "\"agent font matching terminal font\"",
                "\"notebook font size matching terminal font size\"",
                "\"custom padding in alt-screen\"",
            ][..],
        ),
        (
            "app/src/settings_view/ai_page.rs",
            &[
                "\"e.g. ~/code-repos/repo\"",
                "\"e.g. ls .*\"",
                "\"e.g. rm .*\"",
                "\"command (supports regex)\"",
                "set_placeholder_text(\"aws login\"",
            ][..],
        ),
        (
            "app/src/terminal/input/models/data_source.rs",
            &[
                "AUTO_BEDROCK_TOOLTIP",
                "Inference may use Bedrock",
                "Inference via Bedrock",
                "Inference via API key",
            ][..],
        ),
        (
            "app/src/session_management.rs",
            &[
                "Last run command",
                "Last AI interaction",
                "Currently running",
            ][..],
        ),
        (
            "app/src/terminal/shared_session/mod.rs",
            &["Sharing link copied"][..],
        ),
        (
            "app/src/terminal/view/init_environment/mode_selector.rs",
            &[
                "\"Quick setup\"",
                "\"Use the agent\"",
                "Select the GitHub repositories you'd like to work with",
                "Choose a locally set up project",
            ][..],
        ),
        (
            "app/src/terminal/view/ambient_agent/auth_secret_selector.rs",
            &[
                "format!(\"API key '{name}' deleted.\")",
                "format!(\"Failed to delete API key '{name}': {error}\")",
                "format!(\"Delete API key {}\"",
            ][..],
        ),
        (
            "app/src/ai/agent_sdk/ambient.rs",
            &[
                "\"Artifacts:\".to_string()",
                "format!(\"  PR:",
                "format!(\"    Branch:",
                "format!(\"    Link:",
                "unwrap_or(\"Untitled Plan\")",
                "unwrap_or(\"No description\")",
                "format!(\"  Screenshot:",
                "format!(\"  File:",
                "format!(\"    Path:",
                "format!(\"    Description:",
            ][..],
        ),
        (
            "app/src/ai/blocklist/agent_view/shortcuts/mod.rs",
            &[
                "\"input shell command\"",
                "\"for slash commands\"",
                "\"for file paths and attaching other context\"",
                "\"open code review\"",
                "\"toggle conversation list\"",
                "\"search and continue conversations\"",
                "\"start a new conversation\"",
                "\"toggle auto-accept\"",
                "\"pause agent\"",
                "\"go back to terminal\"",
            ][..],
        ),
        (
            "app/src/terminal/warpify/render.rs",
            &["Never Warpify this host"][..],
        ),
        (
            "app/src/terminal/input/rewind/search_item.rs",
            &[
                "query_text: \"Current\"",
                "\"Current state (no rewind)\"",
                "format!(\"Rewind to: {} (+{} -{})\"",
                "format!(\"Rewind to: {} (no code changes)\"",
            ][..],
        ),
        (
            "app/src/terminal/view/ssh_file_upload.rs",
            &[
                "\"Uploading\"",
                "\"Uploaded\"",
                "\"Failed to upload\"",
                "\" to \"",
            ][..],
        ),
        (
            "app/src/workflows/workflow_view/env_var_selector.rs",
            &["\"None\""][..],
        ),
        (
            "app/src/settings_view/handoff_environment_creation_modal.rs",
            &[
                "\"Not logged in\".to_string()",
                "\"Create environment\".to_string()",
            ][..],
        ),
        (
            "app/src/pane_group/pane/code_diff_pane.rs",
            &["set_title(\"Requested Edit\""][..],
        ),
        (
            "app/src/workflows/info_box.rs",
            &[
                "\"Edit prompt\"",
                "\"Edit workflow\"",
                "\"Save as workflow\".to_string()",
            ][..],
        ),
        (
            "app/src/terminal/input/slash_commands/cloud_mode_v2_view.rs",
            &[
                "Self::Commands => \"Commands\"",
                "Self::Skills => \"Skills\"",
                "Self::Prompts => \"Prompts\"",
                "format!(\"Show {hidden_count} more\")",
            ][..],
        ),
        (
            "app/src/search/search_results_menu/view.rs",
            &["return Some(\"Prompts\")"][..],
        ),
        (
            "app/src/workspace/home.rs",
            &[
                "Welcome to Warp on Web",
                "Use Warp on Web to:",
                "Join Shared Sessions",
                "Manage your Warp Settings",
            ][..],
        ),
        ("app/src/view_components/find.rs", &["\"Scanning...\""][..]),
        (
            "app/src/terminal/view/ambient_agent/block/harness_session_header.rs",
            &["\"Agent\".to_owned()", "format!(\"Running {}...\""][..],
        ),
    ];

    let violations = selected_snippet_violations(&cases);

    assert!(
        violations.is_empty(),
        "selected accessibility and fallback surfaces must use catalog copy: {violations:#?}"
    );
}

#[test]
fn terminal_input_search_items_have_localized_accessibility_labels() {
    let cases: [(&str, &[&str]); 7] = [
        (
            "app/src/terminal/input/skills/data_source.rs",
            &["terminal.input.skills.a11y.label"],
        ),
        (
            "app/src/terminal/input/prompts/data_source.rs",
            &["terminal.input.prompts.a11y.label"],
        ),
        (
            "app/src/terminal/input/plans/search_item.rs",
            &["terminal.input.plans.a11y.label"],
        ),
        (
            "app/src/terminal/input/repos/search_item.rs",
            &["terminal.input.repos.a11y.indexed_repository"],
        ),
        (
            "app/src/terminal/input/conversations/search_item.rs",
            &["terminal.input.conversations.a11y.label"],
        ),
        (
            "app/src/terminal/input/user_query/search_item.rs",
            &["terminal.input.user_query.a11y.label"],
        ),
        (
            "app/src/terminal/input/inline_history/search_item.rs",
            &[
                "terminal.inline_history.a11y.conversation",
                "terminal.inline_history.a11y.command",
                "terminal.inline_history.a11y.ai_prompt",
            ],
        ),
    ];

    let mut violations = Vec::new();
    for (relative_path, keys) in cases {
        let path = workspace_root().join(relative_path);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        if !content.contains("fn accessibility_label_for_app(&self, app: &AppContext) -> String") {
            violations.push(format!(
                "{relative_path}: missing accessibility_label_for_app"
            ));
        }
        for key in keys {
            if !content.contains(key) {
                violations.push(format!("{relative_path}: missing key {key}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "terminal input search item accessibility labels must use localized app copy: {violations:#?}"
    );
}

#[test]
fn create_project_suggestion_prompts_are_catalog_backed() {
    let required_keys = [
        "coding_entrypoints.create_project.suggestion.minesweeper.prompt",
        "coding_entrypoints.create_project.suggestion.node_quotes.prompt",
        "coding_entrypoints.create_project.suggestion.csv_to_json.prompt",
        "coding_entrypoints.create_project.suggestion.resume_page.prompt",
        "coding_entrypoints.create_project.suggestion.game_of_life.prompt",
    ];
    assert_bundled_keys_exist(&required_keys);

    let cases = [(
        "app/src/coding_entrypoints/create_project_view.rs",
        &[
            "Build a Minesweeper clone in React",
            "Code a Node.js server that returns random quotes from a JSON file",
            "Write a CSV to JSON converter CLI",
            "Create a starter template",
            "Make a Conway's Game of Life simulation",
        ][..],
    )];

    let violations = selected_snippet_violations(&cases);

    assert!(
        violations.is_empty(),
        "suggestion prompts must use catalog copy: {violations:#?}"
    );
}

fn bundled_en_us_map() -> CatalogMap {
    serde_json::from_str(BUNDLED_EN_US).unwrap()
}

fn bundled_zh_cn_map() -> CatalogMap {
    serde_json::from_str(BUNDLED_ZH_CN).unwrap()
}

fn assert_bundled_keys_exist<S: AsRef<str>>(keys: &[S]) {
    let en_us = bundled_en_us_map();
    let zh_cn = bundled_zh_cn_map();

    let missing = keys
        .iter()
        .flat_map(|key| {
            let key = key.as_ref();
            [
                en_us.get(key).is_none().then(|| format!("en-US:{key}")),
                zh_cn.get(key).is_none().then(|| format!("zh-CN:{key}")),
            ]
            .into_iter()
            .flatten()
        })
        .collect::<Vec<_>>();

    assert!(missing.is_empty(), "missing bundled keys: {missing:#?}");
}

fn assert_language_option_labels(catalog: &Catalog, expected: [(AppLanguage, &str); 3]) {
    for (language, expected_label) in expected {
        assert_eq!(
            catalog.get(language.translation_key()),
            Some(expected_label)
        );
    }
}

fn empty_translation_keys(catalog: &CatalogMap) -> Vec<&str> {
    catalog
        .iter()
        .filter_map(|(key, value)| (value.as_str() == Some("")).then_some(key.as_str()))
        .collect()
}

fn static_slash_command_names_from_source() -> Vec<String> {
    let path =
        workspace_root().join("app/src/search/slash_command_menu/static_commands/commands.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            trimmed
                .strip_prefix("name:")
                .map(str::trim_start)
                .and_then(|rest| string_literals(rest).into_iter().next())
        })
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn static_slash_command_hint_keys_from_source() -> Vec<String> {
    let path =
        workspace_root().join("app/src/search/slash_command_menu/static_commands/commands.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    content
        .lines()
        .filter(|line| line.contains(".with_hint_text_key("))
        .flat_map(string_literals)
        .filter(|literal| literal.starts_with("terminal.slash.command."))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn slash_command_localization_key(command_name: &str, suffix: &str) -> String {
    let key_name = command_name.trim_start_matches('/').replace('-', "_");
    format!("terminal.slash.command.{key_name}.{suffix}")
}

fn plugin_instruction_keys_from_source() -> Vec<String> {
    let dir = workspace_root().join("app/src/terminal/cli_agent_sessions/plugin_manager");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
    let mut keys = BTreeSet::new();

    for entry in entries {
        let entry = entry.expect("failed to read plugin manager directory entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        keys.extend(
            content
                .lines()
                .flat_map(string_literals)
                .filter(|literal| literal.starts_with("terminal.plugin_instructions."))
                .map(str::to_owned),
        );
    }

    keys.into_iter().collect()
}

fn plugin_instruction_key_violations_from_source() -> Vec<String> {
    let dir = workspace_root().join("app/src/terminal/cli_agent_sessions/plugin_manager");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
    let mut violations = Vec::new();

    for entry in entries {
        let entry = entry.expect("failed to read plugin manager directory entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("_tests.rs"))
        {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let relative_path = path
            .strip_prefix(workspace_root())
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        collect_plugin_instruction_key_violations(&relative_path, &content, &mut violations);
    }

    violations
}

fn collect_plugin_instruction_key_violations(
    relative_path: &str,
    content: &str,
    violations: &mut Vec<String>,
) {
    let mut cursor = 0;

    while let Some(found_at) = content[cursor..].find("PluginInstructions {") {
        let block_start = cursor + found_at;
        let Some(open_brace_offset) = content[block_start..].find('{') else {
            break;
        };
        let open_brace = block_start + open_brace_offset;
        let Some(close_brace) = matching_brace_end(content, open_brace) else {
            break;
        };
        let block = &content[block_start..=close_brace];

        for (key_field, fallback_field) in [("title_key", "title"), ("subtitle_key", "subtitle")] {
            let fallback = field_string_literal(block, fallback_field).unwrap_or("");
            let key = field_string_literal(block, key_field).unwrap_or("");
            if looks_like_english_ui_text(fallback) && key.is_empty() {
                violations.push(format!(
                    "{}:{}: {fallback_field} fallback {fallback:?} has no catalog key",
                    relative_path,
                    line_number_for_offset(content, block_start)
                ));
            }
        }

        let note_keys = field_string_array(block, "post_install_note_keys");
        let notes = field_string_array(block, "post_install_notes");
        for (note_index, note) in notes.iter().enumerate() {
            if looks_like_english_ui_text(note)
                && note_keys
                    .get(note_index)
                    .is_none_or(|key| key.as_str().is_empty())
            {
                violations.push(format!(
                    "{}:{}: post_install_notes[{note_index}] fallback {note:?} has no catalog key",
                    relative_path,
                    line_number_for_offset(content, block_start)
                ));
            }
        }

        for step_block in struct_blocks(block, "PluginInstructionStep") {
            let description = field_string_literal(step_block, "description").unwrap_or("");
            let key = field_string_literal(step_block, "description_key").unwrap_or("");
            if looks_like_english_ui_text(description) && key.is_empty() {
                violations.push(format!(
                    "{}:{}: step description fallback {description:?} has no catalog key",
                    relative_path,
                    line_number_for_offset(content, block_start)
                ));
            }
        }

        cursor = close_brace + 1;
    }
}

fn struct_blocks<'a>(content: &'a str, struct_name: &str) -> Vec<&'a str> {
    let mut blocks = Vec::new();
    let mut cursor = 0;
    let pattern = format!("{struct_name} {{");

    while let Some(found_at) = content[cursor..].find(&pattern) {
        let block_start = cursor + found_at;
        let Some(open_brace_offset) = content[block_start..].find('{') else {
            break;
        };
        let open_brace = block_start + open_brace_offset;
        let Some(close_brace) = matching_brace_end(content, open_brace) else {
            break;
        };
        blocks.push(&content[block_start..=close_brace]);
        cursor = close_brace + 1;
    }

    blocks
}

fn field_string_literal<'a>(content: &'a str, field: &str) -> Option<&'a str> {
    let field_start = field_start(content, field)?;
    first_string_literal_with_offset(&content[field_start..]).map(|(literal, _)| literal)
}

fn field_string_array(content: &str, field: &str) -> Vec<String> {
    let Some(field_start) = field_start(content, field) else {
        return vec![];
    };
    let Some(array_start_offset) = content[field_start..].find('[') else {
        return vec![];
    };
    let array_start = field_start + array_start_offset;
    let Some(array_end) = matching_bracket_end(content, array_start) else {
        return vec![];
    };

    string_literals(&content[array_start..=array_end])
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn field_start(content: &str, field: &str) -> Option<usize> {
    let pattern = format!("{field}:");
    let mut cursor = 0;
    while let Some(found_at) = content[cursor..].find(&pattern) {
        let start = cursor + found_at;
        let has_field_boundary = content[..start]
            .chars()
            .next_back()
            .is_none_or(|character| {
                character.is_whitespace() || matches!(character, '{' | ',' | '\n')
            });
        if has_field_boundary {
            return Some(start);
        }
        cursor = start + pattern.len();
    }
    None
}

fn matching_brace_end(content: &str, open_brace: usize) -> Option<usize> {
    matching_delimiter_end(content, open_brace, b'{', b'}')
}

fn matching_bracket_end(content: &str, open_bracket: usize) -> Option<usize> {
    matching_delimiter_end(content, open_bracket, b'[', b']')
}

fn matching_paren_end(content: &str, open_paren: usize) -> Option<usize> {
    matching_delimiter_end(content, open_paren, b'(', b')')
}

fn matching_delimiter_end(
    content: &str,
    open_index: usize,
    open_delimiter: u8,
    close_delimiter: u8,
) -> Option<usize> {
    let bytes = content.as_bytes();
    if bytes.get(open_index) != Some(&open_delimiter) {
        return None;
    }

    let mut depth = 0usize;
    let mut in_string = false;
    let mut index = open_index;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' && !is_escaped_quote(bytes, index) {
            in_string = !in_string;
        } else if !in_string {
            if byte == open_delimiter {
                depth += 1;
            } else if byte == close_delimiter {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
        }
        index += 1;
    }

    None
}

fn top_level_arguments(input: &str) -> Vec<&str> {
    let bytes = input.as_bytes();
    let mut arguments = Vec::new();
    let mut argument_start = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut in_string = false;
    let mut index = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' && !is_escaped_quote(bytes, index) {
            in_string = !in_string;
        } else if !in_string {
            match byte {
                b'{' => brace_depth += 1,
                b'}' => brace_depth = brace_depth.saturating_sub(1),
                b'[' => bracket_depth += 1,
                b']' => bracket_depth = bracket_depth.saturating_sub(1),
                b'(' => paren_depth += 1,
                b')' => paren_depth = paren_depth.saturating_sub(1),
                b',' if brace_depth == 0 && bracket_depth == 0 && paren_depth == 0 => {
                    arguments.push(input[argument_start..index].trim());
                    argument_start = index + 1;
                }
                _ => {}
            }
        }
        index += 1;
    }

    if argument_start < input.len() {
        arguments.push(input[argument_start..].trim());
    }

    arguments
}

fn static_prompt_suggestion_keys_from_source() -> Vec<String> {
    let path = workspace_root()
        .join("app/src/ai/blocklist/passive_suggestions/static_prompt_suggestions.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    content
        .lines()
        .flat_map(string_literals)
        .filter(|literal| literal.starts_with("terminal.passive_suggestion.static."))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn zero_state_prompt_suggestion_keys_from_source() -> Vec<String> {
    let path = workspace_root().join("app/src/terminal/view/inline_banner/prompt_suggestions.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    content
        .lines()
        .flat_map(string_literals)
        .filter(|literal| literal.starts_with("terminal.prompt_suggestion.zero_state."))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn collect_app_localization_key_literals(dir: &Path, keys: &mut BTreeSet<String>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));

    for entry in entries {
        let entry = entry.expect("failed to read app source directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_app_localization_key_literals(&path, keys);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.ends_with("_tests.rs") || name == "localization_tests.rs" || name == "test.rs"
            })
        {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        collect_localization_key_literals_from_source(&content, keys);
    }
}

fn collect_localization_key_literals_from_source(content: &str, keys: &mut BTreeSet<String>) {
    let lines = content.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || !line_may_reference_localization_key(line) {
            continue;
        }

        let source = if line_may_start_multiline_localization_call(line) {
            lines[index..lines.len().min(index + 12)].join("\n")
        } else {
            line.to_string()
        };
        keys.extend(
            string_literals(&source)
                .into_iter()
                .filter(|literal| looks_like_catalog_key(literal))
                .map(str::to_owned),
        );
    }
}

fn line_may_reference_localization_key(line: &str) -> bool {
    line_may_start_multiline_localization_call(line)
        || line.contains("translation_key(")
        || line.contains("display_label_key(")
        || line.contains("description_key(")
        || line.contains("with_hint_text_key(")
        || line.contains("title_key(")
        || line.contains("subtitle_key(")
        || line.contains("setup_status_text_key(")
        || line.contains("accessibility_label_key(")
        || line.contains("placeholder_key")
        || line.contains("_key:")
        || line.contains("_key =")
        || line.contains("_key =>")
}

fn line_may_start_multiline_localization_call(line: &str) -> bool {
    line.contains("text_for_app")
        || line.contains("text_for_locale")
        || line.contains("localization::text(")
        || line.contains("localization::text_with_args(")
        || line.contains("language_option_label(")
        || line.contains("ai_settings_text(")
        || line.contains("workspace_text(")
        || line.contains("workspace_text_with_args(")
        || line.contains("billing_text(")
        || line.contains("code_review_text(")
        || line.contains("rule_text(")
        || line.contains("code_text(")
        || line.contains("input_binding_description(")
        || line.contains("binding_description(")
        || line.contains("localized_binding_description(")
}

fn looks_like_catalog_key(literal: &str) -> bool {
    if !literal.contains('.') || literal.contains(' ') || literal.contains('/') {
        return false;
    }
    if literal.ends_with(".rs")
        || literal.ends_with(".toml")
        || literal.ends_with(".json")
        || literal.contains(".amazonaws.")
    {
        return false;
    }
    let mut parts = literal.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    is_catalog_key_part(first) && parts.all(is_catalog_key_part)
}

fn is_catalog_key_part(part: &str) -> bool {
    !part.is_empty()
        && part
            .chars()
            .all(|ch| ch == '_' || ch == '-' || ch.is_ascii_lowercase() || ch.is_ascii_digit())
}

fn selected_snippet_violations(cases: &[(&str, &[&str])]) -> Vec<String> {
    let mut violations = Vec::new();

    for (relative_path, snippets) in cases {
        let path = workspace_root().join(relative_path);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        for (line_index, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("/*")
            {
                continue;
            }
            for snippet in *snippets {
                if line.contains(snippet) {
                    violations.push(format!("{relative_path}:{}: {snippet}", line_index + 1));
                }
            }
        }
    }

    violations
}

fn placeholders(value: &str) -> BTreeSet<&str> {
    let mut placeholders = BTreeSet::new();
    let mut rest = value;

    while let Some(start) = rest.find('{') {
        rest = &rest[start + 1..];
        if rest.starts_with('{') {
            rest = &rest[1..];
            continue;
        }

        let Some(end) = rest.find('}') else {
            break;
        };
        let name = &rest[..end];
        if is_placeholder_name(name) {
            placeholders.insert(name);
        }
        rest = &rest[end + 1..];
    }

    placeholders
}

fn is_placeholder_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("localization crate should live under crates/localization")
        .to_path_buf()
}

fn collect_direct_ui_literal_violations(dir: &Path, violations: &mut Vec<String>) {
    collect_direct_ui_literal_violations_with_patterns(dir, UI_LITERAL_PATTERNS, violations);
}

fn collect_skill_entrypoint_description_violations(dir: &Path, violations: &mut Vec<String>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));

    for entry in entries {
        let entry = entry.expect("failed to read bundled skill directory entry");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_path = path.join("SKILL.md");
        if skill_path.exists() {
            let content = fs::read_to_string(&skill_path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", skill_path.display()));
            if !content.contains("\ndescription_zh_CN:") {
                violations.push(
                    skill_path
                        .strip_prefix(workspace_root())
                        .unwrap_or(skill_path.as_path())
                        .display()
                        .to_string(),
                );
            }
        } else {
            collect_skill_entrypoint_description_violations(&path, violations);
        }
    }
}

fn collect_skill_description_semantic_violations(dir: &Path, violations: &mut Vec<String>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));

    for entry in entries {
        let entry = entry.expect("failed to read bundled skill directory entry");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_path = path.join("SKILL.md");
        if !skill_path.exists() {
            collect_skill_description_semantic_violations(&path, violations);
            continue;
        }

        let content = fs::read_to_string(&skill_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", skill_path.display()));
        let Some(front_matter) = skill_front_matter(&content) else {
            violations.push(format!(
                "{}: missing YAML front matter",
                skill_path.display()
            ));
            continue;
        };
        let yaml: YamlValue = serde_yaml::from_str(&front_matter)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", skill_path.display()));
        let Some(mapping) = yaml.as_mapping() else {
            violations.push(format!(
                "{}: front matter is not a mapping",
                skill_path.display()
            ));
            continue;
        };
        let value = |key: &str| {
            mapping
                .get(&YamlValue::String(key.to_owned()))
                .and_then(YamlValue::as_str)
        };
        let Some(description) = value("description") else {
            violations.push(format!("{}: missing description", skill_path.display()));
            continue;
        };
        let Some(localized) = value("description_zh_CN") else {
            violations.push(format!(
                "{}: missing description_zh_CN",
                skill_path.display()
            ));
            continue;
        };

        if !localized
            .chars()
            .any(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch))
        {
            violations.push(format!(
                "{}: description_zh_CN contains no Simplified Chinese text",
                skill_path.display()
            ));
        }

        for anchor in inline_code_spans(description) {
            if !localized.contains(anchor) {
                violations.push(format!(
                    "{}: description_zh_CN dropped technical anchor `{anchor}`",
                    skill_path.display()
                ));
            }
        }

        let skill_name = value("name").unwrap_or_default();
        let required_phrases: &[&str] = match skill_name {
            "claude-api" => &[
                "prompt caching",
                "Claude 4.5",
                "Claude 4.6",
                "Claude 4.7",
                "cache hit rate",
                "provider-neutral",
            ],
            "figma-use" => &["绝不能", "难以调试"],
            "verify-ui-change-in-cloud" => &["仅在非沙盒的本地环境"],
            _ => &[],
        };
        for phrase in required_phrases {
            if !localized.contains(phrase) {
                violations.push(format!(
                    "{}: description_zh_CN dropped required trigger phrase `{phrase}`",
                    skill_path.display()
                ));
            }
        }
    }
}

fn skill_front_matter(content: &str) -> Option<String> {
    let content = content.replace("\r\n", "\n");
    content.strip_prefix("---\n").and_then(|content| {
        content
            .split_once("\n---\n")
            .map(|(yaml, _)| yaml.to_owned())
    })
}

fn inline_code_spans(text: &str) -> BTreeSet<&str> {
    text.split('`')
        .enumerate()
        .filter_map(|(index, span)| (index % 2 == 1 && !span.is_empty()).then_some(span))
        .collect()
}

fn collect_direct_ui_literal_violations_with_patterns(
    dir: &Path,
    patterns: &[&str],
    violations: &mut Vec<String>,
) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));

    for entry in entries {
        let entry = entry.expect("failed to read app source directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_direct_ui_literal_violations_with_patterns(&path, patterns, violations);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.ends_with("_tests.rs") || name == "localization_tests.rs" || name == "test.rs"
            })
        {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let relative_path = path
            .strip_prefix(workspace_root())
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        for pattern in patterns {
            collect_direct_first_argument_literal_violations(
                &relative_path,
                &content,
                pattern,
                violations,
                None,
            );
        }
    }
}

fn collect_direct_first_argument_literal_violations(
    relative_path: &str,
    content: &str,
    pattern: &str,
    violations: &mut Vec<String>,
    allowed_catalog_map_source: Option<&str>,
) {
    let mut cursor = 0;
    while let Some(found_at) = content[cursor..].find(pattern) {
        let invocation_start = cursor + found_at;
        let arg_start = invocation_start + pattern.len();
        let line_start = content[..invocation_start]
            .rfind('\n')
            .map_or(0, |offset| offset + 1);
        if content[line_start..invocation_start]
            .trim_start()
            .starts_with("//")
        {
            cursor = arg_start;
            continue;
        }
        if let Some(literal) = first_argument_string_literal(&content[arg_start..])
            && !ALLOWED_DIRECT_UI_LITERALS.contains(&literal)
            && looks_like_english_ui_text(literal)
        {
            if catalog_map_contains_literal(allowed_catalog_map_source, literal) {
                cursor = arg_start;
                continue;
            }
            violations.push(format!(
                "{}:{}: {literal:?}",
                relative_path,
                line_number_for_offset(content, invocation_start)
            ));
        }
        cursor = arg_start;
    }
}

fn collect_direct_first_argument_literal_violations_in_dir(
    dir: &Path,
    pattern: &str,
    violations: &mut Vec<String>,
    allowed_catalog_map_source: Option<&str>,
) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));

    for entry in entries {
        let entry = entry.expect("failed to read source directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_direct_first_argument_literal_violations_in_dir(
                &path,
                pattern,
                violations,
                allowed_catalog_map_source,
            );
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.ends_with("_tests.rs") || name == "localization_tests.rs" || name == "test.rs"
            })
        {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let relative_path = path
            .strip_prefix(workspace_root())
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        collect_direct_first_argument_literal_violations(
            &relative_path,
            &content,
            pattern,
            violations,
            allowed_catalog_map_source,
        );
    }
}

fn collect_binding_description_literal_violations_in_dir(
    dir: &Path,
    pattern: &str,
    description_argument_index: usize,
    violations: &mut Vec<String>,
    allowed_catalog_map_source: Option<&str>,
) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));

    for entry in entries {
        let entry = entry.expect("failed to read source directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_binding_description_literal_violations_in_dir(
                &path,
                pattern,
                description_argument_index,
                violations,
                allowed_catalog_map_source,
            );
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.ends_with("_tests.rs") || name == "localization_tests.rs" || name == "test.rs"
            })
        {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let relative_path = path
            .strip_prefix(workspace_root())
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        collect_binding_description_literal_violations(
            &relative_path,
            &content,
            pattern,
            description_argument_index,
            violations,
            allowed_catalog_map_source,
        );
    }
}

fn collect_binding_description_literal_violations(
    relative_path: &str,
    content: &str,
    pattern: &str,
    description_argument_index: usize,
    violations: &mut Vec<String>,
    allowed_catalog_map_source: Option<&str>,
) {
    let mut cursor = 0;
    while let Some(found_at) = content[cursor..].find(pattern) {
        let invocation_start = cursor + found_at;
        let open_paren = invocation_start + pattern.len() - 1;
        let Some(close_paren) = matching_paren_end(content, open_paren) else {
            break;
        };
        let arguments = top_level_arguments(&content[open_paren + 1..close_paren]);
        if let Some(argument) = arguments.get(description_argument_index) {
            let argument = argument.trim_start();
            if let Some(literal) = argument_string_literal(argument)
                && !ALLOWED_DIRECT_UI_LITERALS.contains(&literal)
                && looks_like_english_ui_text(literal)
            {
                if catalog_map_contains_literal(allowed_catalog_map_source, literal) {
                    cursor = close_paren + 1;
                    continue;
                }
                violations.push(format!(
                    "{}:{}: {literal:?}",
                    relative_path,
                    line_number_for_offset(content, invocation_start)
                ));
            }
        }
        cursor = close_paren + 1;
    }
}

fn binding_description_catalog_map_source() -> &'static str {
    include_str!("../../../app/src/util/bindings.rs")
}

fn catalog_map_contains_literal(map_source: Option<&str>, literal: &str) -> bool {
    let Some(map_source) = map_source else {
        return false;
    };
    if english_catalog_contains_text(literal) {
        return true;
    }
    let titlecase_literal = titlecase_for_binding_description(literal);
    if english_catalog_contains_text(&titlecase_literal) {
        return true;
    }
    map_source.contains(&format!("\"{titlecase_literal}\" =>"))
}

fn english_catalog_contains_text(text: &str) -> bool {
    english_catalog_values().iter().any(|value| value == text)
}

fn english_catalog_values() -> Vec<String> {
    let catalog: std::collections::HashMap<String, String> =
        serde_json::from_str(BUNDLED_EN_US).expect("en-US catalog must be valid JSON");
    catalog.into_values().collect()
}

fn titlecase_for_binding_description(literal: &str) -> String {
    literal
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn collect_direct_literal_after_patterns(
    relative_path: &str,
    content: &str,
    patterns: &[&str],
    violations: &mut Vec<String>,
) {
    const SCAN_WINDOW: usize = 512;

    for pattern in patterns {
        let mut cursor = 0;
        while let Some(found_at) = content[cursor..].find(pattern) {
            let invocation_start = cursor + found_at;
            let scan_start = invocation_start + pattern.len();
            let mut scan_end = (scan_start + SCAN_WINDOW).min(content.len());
            while scan_end < content.len() && !content.is_char_boundary(scan_end) {
                scan_end += 1;
            }
            if let Some((literal, offset)) =
                first_string_literal_with_offset(&content[scan_start..scan_end])
                && !ALLOWED_DIRECT_UI_LITERALS.contains(&literal)
                && looks_like_english_ui_text(literal)
            {
                violations.push(format!(
                    "{}:{}: {literal:?}",
                    relative_path,
                    line_number_for_offset(content, scan_start + offset)
                ));
            }
            cursor = scan_start;
        }
    }
}

fn collect_direct_literal_after_patterns_in_dir(
    dir: &Path,
    patterns: &[&str],
    violations: &mut Vec<String>,
) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));

    for entry in entries {
        let entry = entry.expect("failed to read source directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_direct_literal_after_patterns_in_dir(&path, patterns, violations);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs")
            || path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("_tests.rs") || name == "test.rs")
        {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let relative_path = path
            .strip_prefix(workspace_root())
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        collect_direct_literal_after_patterns(&relative_path, &content, patterns, violations);
    }
}

fn first_string_literal_with_offset(input: &str) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    let mut cursor = 0;

    while cursor < bytes.len() {
        let Some(start_offset) = input[cursor..].find('"') else {
            break;
        };
        let start = cursor + start_offset;
        let mut end = start + 1;
        while end < bytes.len() {
            if bytes[end] == b'"' && !is_escaped_quote(bytes, end) {
                return Some((&input[start + 1..end], start));
            }
            end += 1;
        }
        cursor = end;
    }

    None
}

fn string_literals(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut literals = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(start_offset) = line[cursor..].find('"') else {
            break;
        };
        let start = cursor + start_offset;
        let mut end = start + 1;
        while end < bytes.len() {
            if bytes[end] == b'"' && !is_escaped_quote(bytes, end) {
                literals.push(&line[start + 1..end]);
                cursor = end + 1;
                break;
            }
            end += 1;
        }
        if end >= bytes.len() {
            break;
        }
    }
    literals
}

fn first_argument_string_literal(input: &str) -> Option<&str> {
    let trimmed = input.trim_start();
    let trimmed = trimmed.strip_prefix('&').unwrap_or(trimmed).trim_start();
    if let Some(format_args) = trimmed.strip_prefix("format!(") {
        return argument_string_literal(format_args);
    }
    argument_string_literal(trimmed)
}

fn argument_string_literal(input: &str) -> Option<&str> {
    let trimmed = input.trim_start();
    let trimmed = trimmed.strip_prefix('&').unwrap_or(trimmed).trim_start();
    if !trimmed.starts_with('"') {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let mut i = 1;
    while i < bytes.len() {
        if bytes[i] == b'"' && !is_escaped_quote(bytes, i) {
            return Some(&trimmed[1..i]);
        }
        i += 1;
    }
    None
}

fn line_number_for_offset(content: &str, offset: usize) -> usize {
    content[..offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn is_escaped_quote(bytes: &[u8], quote_index: usize) -> bool {
    let mut slash_count = 0;
    let mut i = quote_index;
    while i > 0 && bytes[i - 1] == b'\\' {
        slash_count += 1;
        i -= 1;
    }
    slash_count % 2 == 1
}

fn looks_like_english_ui_text(literal: &str) -> bool {
    literal.len() >= 3
        && literal.chars().any(|ch| ch.is_ascii_alphabetic())
        && literal
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        && literal.chars().any(|ch| ch.is_ascii_lowercase())
}
