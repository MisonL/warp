use clap::Parser;
use warp_localization::LocaleId;

use super::{TuiArgs, parse_resume_token, tui_command_for};

#[test]
fn parses_resume_server_token() {
    let token = uuid::Uuid::new_v4().to_string();
    let args = TuiArgs::try_parse_from([
        "warp",
        "--resume",
        token.as_str(),
        "--api-key",
        "test-api-key",
    ])
    .expect("TUI launch arguments should parse together");

    assert_eq!(args.resume.as_deref(), Some(token.as_str()));
    assert_eq!(args.api_key.as_deref(), Some("test-api-key"));
    assert_eq!(
        parse_resume_token(token.clone())
            .expect("UUID token should validate")
            .as_str(),
        token
    );
}

#[test]
fn rejects_malformed_resume_server_token() {
    let error = parse_resume_token("not-a-token".to_owned())
        .expect_err("non-UUID token should be rejected");

    assert!(error.to_string().contains("not-a-token"));
}

#[test]
fn accepts_startup_without_resume() {
    let args = TuiArgs::try_parse_from(["warp"]).expect("empty arguments should parse");

    assert_eq!(args.resume, None);
    assert_eq!(args.api_key, None);
}

#[test]
fn help_uses_simplified_chinese_catalog() {
    let mut command = tui_command_for(LocaleId::ZhCn).disable_colored_help(true);
    let mut output = Vec::new();
    command
        .write_help(&mut output)
        .expect("TUI help should render");
    let help = String::from_utf8(output).expect("TUI help should be valid UTF-8");
    assert!(help.contains("Warp 无界面终端用户界面"));
    assert!(help.contains("用法:"));
    assert!(help.contains("选项"), "rendered help:\n{help}");
    assert!(help.contains("使用服务器令牌恢复 Oz/Warp 对话"));
    assert!(help.contains("用于非交互式身份验证的 API 密钥"));
}
