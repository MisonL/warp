use std::ffi::OsString;

use serial_test::serial;

use super::*;

fn set_env_var(name: &str, value: &str) -> Option<OsString> {
    let previous = std::env::var_os(name);
    // Safety: this environment-mutating test is serialized within the crate.
    unsafe { std::env::set_var(name, value) };
    previous
}

fn restore_env_var(name: &str, previous: Option<OsString>) {
    match previous {
        // Safety: this environment-mutating test is serialized within the crate.
        Some(value) => unsafe { std::env::set_var(name, value) },
        // Safety: this environment-mutating test is serialized within the crate.
        None => unsafe { std::env::remove_var(name) },
    }
}

#[test]
fn test_parse_simple_skill_name() {
    let spec: SkillSpec = "code-review".parse().unwrap();
    assert_eq!(spec.org, None);
    assert_eq!(spec.repo, None);
    assert_eq!(spec.skill_identifier, "code-review");
    assert!(!spec.is_full_path());
}

#[test]
fn test_parse_repo_qualified() {
    let spec: SkillSpec = "warp-internal:code-review".parse().unwrap();
    assert_eq!(spec.org, None);
    assert_eq!(spec.repo, Some("warp-internal".to_string()));
    assert_eq!(spec.skill_identifier, "code-review");
    assert!(!spec.is_full_path());
}

#[test]
fn test_parse_org_repo_qualified() {
    let spec: SkillSpec = "warpdotdev/warp-internal:code-review".parse().unwrap();
    assert_eq!(spec.org, Some("warpdotdev".to_string()));
    assert_eq!(spec.repo, Some("warp-internal".to_string()));
    assert_eq!(spec.skill_identifier, "code-review");
    assert!(!spec.is_full_path());
}

#[test]
fn test_parse_full_path_with_org_repo() {
    let spec: SkillSpec = "warpdotdev/warp-internal:.claude/skills/deploy/SKILL.md"
        .parse()
        .unwrap();
    assert_eq!(spec.org, Some("warpdotdev".to_string()));
    assert_eq!(spec.repo, Some("warp-internal".to_string()));
    assert_eq!(spec.skill_identifier, ".claude/skills/deploy/SKILL.md");
    assert!(spec.is_full_path());
}

#[test]
fn test_parse_full_path_with_repo() {
    let spec: SkillSpec = "warp-server:.agents/skills/test/SKILL.md".parse().unwrap();
    assert_eq!(spec.org, None);
    assert_eq!(spec.repo, Some("warp-server".to_string()));
    assert_eq!(spec.skill_identifier, ".agents/skills/test/SKILL.md");
    assert!(spec.is_full_path());
}

#[test]
fn test_display_simple_name() {
    let spec = SkillSpec::without_repo("code-review".to_string());
    assert_eq!(spec.to_string(), "code-review");
}

#[test]
fn test_display_repo_qualified() {
    let spec = SkillSpec::with_repo("warp-internal".to_string(), "code-review".to_string());
    assert_eq!(spec.to_string(), "warp-internal:code-review");
}

#[test]
fn test_display_org_repo_qualified() {
    let spec = SkillSpec::with_org_and_repo(
        "warpdotdev".to_string(),
        "warp-internal".to_string(),
        "code-review".to_string(),
    );
    assert_eq!(spec.to_string(), "warpdotdev/warp-internal:code-review");
}

#[test]
fn test_display_full_path() {
    let spec = SkillSpec::with_org_and_repo(
        "warpdotdev".to_string(),
        "warp-internal".to_string(),
        ".claude/skills/deploy/SKILL.md".to_string(),
    );
    assert_eq!(
        spec.to_string(),
        "warpdotdev/warp-internal:.claude/skills/deploy/SKILL.md"
    );
}

#[test]
fn test_is_full_path_with_slash() {
    let spec = SkillSpec::without_repo(".claude/skills/deploy/SKILL.md".to_string());
    assert!(spec.is_full_path());
}

#[test]
fn test_is_not_full_path_single_component_md() {
    // A single component (even with .md extension) is treated as a skill name, not a full path
    let spec = SkillSpec::without_repo("something.md".to_string());
    assert!(!spec.is_full_path());
}

#[test]
fn test_is_not_full_path() {
    let spec = SkillSpec::without_repo("code-review".to_string());
    assert!(!spec.is_full_path());
}

#[test]
fn test_parse_empty_fails() {
    let result: Result<SkillSpec, _> = "".parse();
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_qualifier_fails() {
    let result: Result<SkillSpec, _> = ":code-review".parse();
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_path_fails() {
    let result: Result<SkillSpec, _> = "warp-internal:".parse();
    assert!(result.is_err());
}

#[test]
#[serial]
fn invalid_skill_specs_use_simplified_chinese_errors() {
    let previous_language = set_env_var("LANGUAGE", "zh_CN");
    let cases = [
        ("", "技能说明符不能为空"),
        (":code-review", "限定符不能为空"),
        ("warp-internal:", "技能标识符不能为空"),
        ("/warp-internal:code-review", "组织名不能为空"),
        ("warpdotdev/:code-review", "仓库名不能为空"),
    ];

    for (input, expected) in cases {
        let error = input
            .parse::<SkillSpec>()
            .expect_err("invalid skill spec should be rejected");
        assert!(error.contains(expected), "unexpected error: {error}");
    }

    restore_env_var("LANGUAGE", previous_language);
}

#[test]
fn test_skill_name_simple_name() {
    let spec: SkillSpec = "feedback-triage-bot".parse().unwrap();
    assert_eq!(spec.skill_name(), "feedback-triage-bot");
}

#[test]
fn test_skill_name_repo_qualified_name() {
    let spec: SkillSpec = "warpdotdev/feedback-triage-bot:feedback-triage-bot"
        .parse()
        .unwrap();
    assert_eq!(spec.skill_name(), "feedback-triage-bot");
}

#[test]
fn test_skill_name_repo_qualified_path() {
    let spec: SkillSpec = "warpdotdev/feedback-triage-bot:.agents/skills/slack-triage/SKILL.md"
        .parse()
        .unwrap();
    assert_eq!(spec.skill_name(), "slack-triage");
}
