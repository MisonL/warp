use warp_localization::LocaleId;

use super::{
    RunnerArch, RunnerArchArg, RunnerOsArg, confirm_delete, localized_runner_arch,
    localized_runner_os, merge_instance_shape, resolve_arch, resolve_updated_name,
};

#[test]
fn confirm_delete_refuses_non_interactive_without_force() {
    // In non-interactive mode, refusal must surface as an error so the caller
    // exits non-zero instead of treating a skipped delete as a success.
    let err = confirm_delete("runner-123", false, LocaleId::EnUs)
        .expect_err("non-interactive refusal is an error");
    let msg = err.to_string();
    assert!(msg.contains("non-interactive"), "got: {msg}");
    assert!(msg.contains("runner-123"), "got: {msg}");
}

#[test]
fn resolve_arch_auto_maps_to_os_default() {
    assert!(matches!(
        resolve_arch(RunnerArchArg::Auto, RunnerOsArg::Linux),
        RunnerArch::X8664
    ));
    assert!(matches!(
        resolve_arch(RunnerArchArg::Auto, RunnerOsArg::Macos),
        RunnerArch::Aarch64
    ));
}

#[test]
fn resolve_arch_explicit_is_preserved_regardless_of_os() {
    assert!(matches!(
        resolve_arch(RunnerArchArg::X8664, RunnerOsArg::Macos),
        RunnerArch::X8664
    ));
    assert!(matches!(
        resolve_arch(RunnerArchArg::Aarch64, RunnerOsArg::Linux),
        RunnerArch::Aarch64
    ));
}

#[test]
fn merge_instance_shape_updates_dimensions_independently() {
    // Neither specified: preserve the existing shape.
    assert_eq!(
        merge_instance_shape(None, None, Some((2, 4)), LocaleId::EnUs).unwrap(),
        Some((2, 4))
    );
    // Only vCPUs: keep existing memory.
    assert_eq!(
        merge_instance_shape(Some(8), None, Some((2, 4)), LocaleId::EnUs).unwrap(),
        Some((8, 4))
    );
    // Only memory: keep existing vCPUs.
    assert_eq!(
        merge_instance_shape(None, Some(16), Some((2, 4)), LocaleId::EnUs).unwrap(),
        Some((2, 16))
    );
    // Both specified: use both.
    assert_eq!(
        merge_instance_shape(Some(8), Some(16), Some((2, 4)), LocaleId::EnUs).unwrap(),
        Some((8, 16))
    );
    // No existing shape and nothing set: no shape.
    assert_eq!(
        merge_instance_shape(None, None, None, LocaleId::EnUs).unwrap(),
        None
    );
}

#[test]
fn merge_instance_shape_errors_on_partial_shape_without_existing() {
    assert!(merge_instance_shape(Some(8), None, None, LocaleId::EnUs).is_err());
    assert!(merge_instance_shape(None, Some(16), None, LocaleId::EnUs).is_err());
}

#[test]
fn resolve_updated_name_renames_only_with_uid() {
    // UID + --name renames the runner.
    assert_eq!(resolve_updated_name(true, Some("new"), "old"), "new");
    // UID without --name keeps the existing name.
    assert_eq!(resolve_updated_name(true, None, "old"), "old");
    // No UID: --name is the selector, so the name is unchanged.
    assert_eq!(resolve_updated_name(false, Some("old"), "old"), "old");
}

#[test]
fn table_runner_values_follow_the_selected_locale() {
    assert_eq!(localized_runner_os(LocaleId::ZhCn, "Linux"), "Linux");
    assert_eq!(localized_runner_arch(LocaleId::ZhCn, "x86-64"), "x86-64");
    assert_eq!(
        localized_runner_os(LocaleId::ZhCn, "custom-os"),
        "custom-os"
    );
}
