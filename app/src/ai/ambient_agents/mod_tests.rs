use super::*;

#[test]
fn localized_task_status_message_localizes_canonical_specific_errors() {
    let cases = [
        ("Agent encountered an error", "Agent 遇到错误"),
        ("Cancelled by user", "用户已取消"),
        (
            "The agent got stuck waiting for user confirmation on the action: delete file",
            "Agent 卡在等待用户确认操作：delete file",
        ),
        (
            "The agent got stuck waiting for user confirmation on the action: Read files",
            "Agent 卡在等待用户确认操作：读取文件",
        ),
        (
            "The agent got stuck waiting for user confirmation on the action: Run command: pwd",
            "Agent 卡在等待用户确认操作：运行命令：pwd",
        ),
        (
            "The agent got stuck waiting for user confirmation on the action: Read mcp resource",
            "Agent 卡在等待用户确认操作：读取 MCP 资源",
        ),
        (
            "The agent got stuck waiting for user confirmation on the action: Write to long running shell command",
            "Agent 卡在等待用户确认操作：写入长时间运行的 shell 命令",
        ),
        (
            "The agent got stuck waiting for user confirmation on the action: Waiting for your answer",
            "Agent 卡在等待用户确认操作：等待你的回答",
        ),
        (
            "Your team has run out of credits. Purchase more credits to continue.",
            "你的团队额度已用完。请购买更多额度后继续。",
        ),
        (
            "Warp is temporarily overloaded. Please try again shortly.",
            "Warp 暂时过载。请稍后重试。",
        ),
        (
            "An internal error occurred during the conversation. Please try again.",
            "对话期间发生内部错误。请重试。",
        ),
        (
            "Context window exceeded: too big",
            "上下文窗口超出限制：too big",
        ),
        (
            "Invalid API key for OpenAI. Update your API key in settings.",
            "OpenAI 的 API key 无效。请在设置中更新 API key。",
        ),
        (
            "AWS Bedrock credentials expired or invalid for claude-v2.",
            "claude-v2 的 AWS Bedrock 凭证已过期或无效。",
        ),
    ];

    for (canonical_message, localized_message) in cases {
        assert_eq!(
            localized_task_status_message_for_locale(LocaleId::ZhCn, canonical_message),
            localized_message
        );
    }
}

#[test]
fn localized_task_status_message_keeps_unknown_errors() {
    assert_eq!(
        localized_task_status_message_for_locale(LocaleId::ZhCn, "provider exploded"),
        "provider exploded"
    );
}

#[test]
fn localized_task_status_message_localizes_canonical_driver_errors() {
    let canonical_inner = localization::text_for_locale_with_args(
        LocaleId::EnUs,
        "agent_sdk.driver.error_classification.bootstrap_error.pty_spawn_failed_with_reason",
        &[("reason", "Argument list too long")],
    );
    let canonical = localization::text_for_locale_with_args(
        LocaleId::EnUs,
        "agent_sdk.driver.error_classification.bootstrap_failed",
        &[("error", &canonical_inner)],
    );

    let localized = localized_task_status_message_for_locale(LocaleId::ZhCn, &canonical);

    assert!(localized.contains("终端会话启动失败"));
    assert!(localized.contains("shell 启动失败"));
    assert!(localized.contains("Argument list too long"));
}

#[test]
fn localized_task_status_message_localizes_driver_errors_with_repeated_arguments() {
    let canonical = localization::text_for_locale_with_args(
        LocaleId::EnUs,
        "agent_sdk.driver.error_classification.conversation_harness_mismatch",
        &[
            ("conversation_id", "conversation-123"),
            ("expected", "claude"),
            ("got", "codex"),
        ],
    );

    let localized = localized_task_status_message_for_locale(LocaleId::ZhCn, &canonical);

    assert!(localized.contains("conversation-123"));
    assert!(localized.contains("claude"));
    assert!(localized.contains("codex"));
    assert!(!localized.contains("was produced by"));
}
