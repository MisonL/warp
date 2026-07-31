use std::fs;

use ai::skills::{ParsedSkill, SkillProvider, SkillScope};
use tempfile::TempDir;
use warp_util::host_id::HostId;
use warp_util::local_or_remote_path::LocalOrRemotePath;

use super::*;

fn bundled_skill(content: &str) -> BundledSkill {
    let mut bundled_skill = BundledSkill::default();
    bundled_skill.insert_for_testing(
        "test-skill",
        ParsedSkill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            path: LocalOrRemotePath::Local("/bundled/skills/test-skill/SKILL.md".into()),
            content: content.to_string(),
            line_range: None,
            provider: SkillProvider::Warp,
            scope: SkillScope::Bundled,
        },
        BundledSkillActivation::Always,
    );
    bundled_skill
}

#[test]
fn unavailable_bundled_context_path_renders_as_empty_string() {
    assert_eq!(display_optional_path(None), "");
}

#[test]
fn localized_settings_schema_path_uses_english_until_the_localized_resource_exists() {
    let temp_dir = TempDir::new().unwrap();
    let resources_dir = temp_dir.path();
    let english_path = resources_dir.join("settings_schema.json");
    let chinese_path = resources_dir.join("settings_schema.zh-CN.json");

    assert_eq!(
        settings_schema_path(resources_dir, LocaleId::EnUs),
        english_path
    );
    assert_eq!(
        settings_schema_path(resources_dir, LocaleId::ZhCn),
        english_path
    );

    fs::write(&chinese_path, "{}\n").unwrap();

    assert_eq!(
        settings_schema_path(resources_dir, LocaleId::ZhCn),
        chinese_path
    );
}

#[test]
fn localized_settings_schema_fallback_requires_an_english_schema() {
    let temp_dir = TempDir::new().unwrap();
    let resources_dir = temp_dir.path();
    let english_path = resources_dir.join("settings_schema.json");
    let chinese_path = resources_dir.join("settings_schema.zh-CN.json");

    assert_eq!(
        english_settings_schema_fallback_path(resources_dir, LocaleId::ZhCn),
        None
    );

    fs::write(&english_path, "{}\n").unwrap();
    assert_eq!(
        english_settings_schema_fallback_path(resources_dir, LocaleId::ZhCn),
        Some(english_path)
    );

    fs::write(&chinese_path, "{}\n").unwrap();
    assert_eq!(
        english_settings_schema_fallback_path(resources_dir, LocaleId::ZhCn),
        None
    );
}

#[test]
fn modify_settings_activation_checks_the_same_locale_specific_schema_path() {
    let temp_dir = TempDir::new().unwrap();
    let resources_dir = temp_dir.path();
    let expected_path = resources_dir.join("settings_schema.zh-CN.json");
    fs::write(&expected_path, "{}\n").unwrap();

    assert!(matches!(
        activation_for_bundled_skill("modify-settings", resources_dir, LocaleId::ZhCn),
        BundledSkillActivation::RequiresFile(path) if path == expected_path
    ));
}

fn remote_content<'a>(bundled_skills: &'a BundledSkills, host_id: &HostId) -> Option<&'a str> {
    bundled_skills
        .remote(host_id)?
        .skill("test-skill")
        .map(|skill| skill.content.as_str())
}

#[test]
fn local_and_remote_catalogs_are_isolated() {
    let first_host_id = HostId::new("first-host".to_string());
    let second_host_id = HostId::new("second-host".to_string());
    let mut bundled_skills = BundledSkills::default();
    bundled_skills.set_local(bundled_skill("local"));
    bundled_skills.insert_remote(first_host_id.clone(), bundled_skill("first"));
    bundled_skills.insert_remote(second_host_id.clone(), bundled_skill("second"));

    assert_eq!(
        bundled_skills
            .local_skill("test-skill")
            .map(|skill| skill.content.as_str()),
        Some("local")
    );
    assert_eq!(
        remote_content(&bundled_skills, &first_host_id),
        Some("first")
    );
    assert_eq!(
        remote_content(&bundled_skills, &second_host_id),
        Some("second")
    );

    // A reconnect refresh replaces the host's catalog wholesale.
    bundled_skills.insert_remote(first_host_id.clone(), bundled_skill("first-refreshed"));
    assert_eq!(
        remote_content(&bundled_skills, &first_host_id),
        Some("first-refreshed")
    );

    // Disconnecting one host leaves the local and sibling-host catalogs intact.
    bundled_skills.remove_remote(&first_host_id);
    assert_eq!(
        bundled_skills
            .local_skill("test-skill")
            .map(|skill| skill.content.as_str()),
        Some("local")
    );
    assert_eq!(remote_content(&bundled_skills, &first_host_id), None);
    assert_eq!(
        remote_content(&bundled_skills, &second_host_id),
        Some("second")
    );
}

#[test]
fn localized_bundled_skill_description_accepts_windows_line_endings() {
    let content = "---\r\nname: test-skill\r\ndescription: English description\r\ndescription_zh_CN: 简体中文描述\r\n---\r\n\r\nbody\r\n";

    assert_eq!(
        localized_bundled_skill_description(content, LocaleId::ZhCn),
        Some("简体中文描述".to_string())
    );
}
