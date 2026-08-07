use super::*;

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
fn replacing_current_locale_reports_only_changes() {
    let locale = RwLock::new(LocaleId::EnUs);

    assert!(!replace_current_locale(&locale, LocaleId::EnUs));
    assert!(replace_current_locale(&locale, LocaleId::ZhCn));
    assert_eq!(*locale.read(), LocaleId::ZhCn);
}

#[test]
fn bundled_simplified_chinese_templates_are_available() {
    assert_eq!(
        &*CATALOGS.text(LocaleId::ZhCn, "tui.auth.sign_in"),
        "登录后继续"
    );
    assert_eq!(
        replace_placeholders(
            &CATALOGS.text(LocaleId::ZhCn, "tui.auth.open_browser"),
            &[("uri", "https://example.com")],
        )
        .expect("localized TUI test template arguments should match the catalog"),
        "在浏览器中打开 https://example.com"
    );
    for (key, expected) in [
        ("tui.markdown.empty_table", "[空表格]"),
        ("tui.markdown.table_has_no_rows", "[表格没有行]"),
        (
            "tui.markdown.unsupported_embedded_content",
            "[不支持的嵌入内容]",
        ),
    ] {
        assert_eq!(text_for_locale(LocaleId::ZhCn, key), expected);
    }
}

#[test]
fn bundled_simplified_chinese_covers_new_tui_surfaces() {
    for (key, expected) in [
        ("tui.agent_message.orchestrator", "协调 Agent"),
        ("tui.ask_question.title", "Agent 问题"),
        ("tui.orchestration.binding.next_tab", "选择下一个编排标签页"),
        (
            "tui.orchestration.binding.open_cloud_run",
            "打开云端运行链接",
        ),
        (
            "tui.orchestration.cloud_run.status.in_progress",
            "云端运行进行中",
        ),
        ("tui.orchestration.tab_bar.orchestrator", "协调 Agent"),
        ("tui.orchestration.option.default_model", "默认模型"),
        ("tui.plan.updated", "已更新计划"),
        ("tui.permission_prompt.option.edit_command", "编辑命令"),
        ("tui.permission_prompt.option.other", "其他"),
        ("tui.permission_prompt.option.yes", "是"),
        ("tui.permission_prompt.footer.key.confirm", "Enter"),
        ("tui.permission_prompt.footer.key.edit", "Ctrl+E"),
        ("tui.permission_prompt.footer.key.escape", "Esc"),
        (
            "tui.shell_command.permission.empty_command_error",
            "请输入命令后继续。",
        ),
        (
            "tui.shell_command.permission.title",
            "我可以运行此命令并读取输出吗？",
        ),
        ("tui.failure.compare_plans", "比较套餐"),
        ("tui.failure.use_own_api_keys", "使用你自己的 API 密钥"),
        ("tui.failure.usage_notice", "此响应不会计入你的用量。"),
        ("tui.rich_text.code_unavailable", "[代码块不可用]"),
        ("tui.clipboard.copy.success", "已复制到剪贴板"),
        ("tui.clipboard.copy.sent_to_terminal", "已发送到终端剪贴板"),
        ("tui.clipboard.copy.failed", "无法复制到剪贴板"),
        (
            "tui.session.export.copied_to_clipboard",
            "对话已复制到剪贴板",
        ),
        (
            "tui.session.log_bundle.failed",
            "无法创建日志包（请检查日志）",
        ),
    ] {
        assert_eq!(text_for_locale(LocaleId::ZhCn, key), expected);
    }
    assert_eq!(
        text_with_args_for_locale(
            LocaleId::ZhCn,
            "tui.session.log_bundle.saved",
            &[("path", "/tmp/warp.zip")],
        ),
        "日志包已保存到 /tmp/warp.zip",
    );
    assert_eq!(
        text_with_args_for_locale(
            LocaleId::ZhCn,
            "tui.orchestration.child.remote_skills_unresolved",
            &[("references", "frontend, backend")],
        ),
        "无法解析子 Agent 技能：frontend, backend",
    );
    assert_eq!(
        text_with_args_for_locale(
            LocaleId::ZhCn,
            "tui.orchestration.cloud_run.detail.github_auth_required",
            &[("message", "需要 GitHub 认证")],
        ),
        "需要 GitHub 认证 请完成认证后重新运行编排请求。",
    );
    assert_eq!(
        text_with_args_for_locale(
            LocaleId::ZhCn,
            "tui.tool.awaiting_approval",
            &[("label", "运行 `cargo test`")],
        ),
        "运行 `cargo test`（等待批准）",
    );
}
