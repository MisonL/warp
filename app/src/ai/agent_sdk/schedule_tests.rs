use chrono::{TimeZone as _, Utc};
use warp_cli::agent::OutputFormat;
use warp_localization::LocaleId;

use super::ScheduleInfo;
use crate::ai::agent_sdk::output::write_list_for_locale;
use crate::ai::ambient_agents::AgentConfigSnapshot;

fn schedule_info() -> ScheduleInfo {
    ScheduleInfo {
        id: "schedule-1".to_string(),
        name: "nightly".to_string(),
        cron_schedule: "0 0 * * *".to_string(),
        paused: true,
        last_ran: Some(Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()),
        next_run: None,
        scope: "Team".to_string(),
        prompt: "Run checks".to_string(),
        last_spawn_error: Some("failed".to_string()),
        agent_config: AgentConfigSnapshot::default(),
    }
}

#[test]
fn schedule_list_text_localizes_status_rows() {
    let localized_paused =
        crate::localization::text_for_locale(LocaleId::ZhCn, "agent_sdk.common.value.yes");
    let localized_error_prefix = crate::localization::text_for_locale(
        LocaleId::ZhCn,
        "agent_sdk.schedule.value.last_ran_with_error",
    )
    .replace("{timestamp}", "");
    let localized_scope =
        crate::localization::text_for_locale(LocaleId::ZhCn, "agent_sdk.secret.scope.team");
    let mut output = Vec::new();

    write_list_for_locale(
        [schedule_info()],
        OutputFormat::Text,
        &mut output,
        LocaleId::ZhCn,
    )
    .unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains(&localized_paused));
    assert!(rendered.contains(&localized_error_prefix));
    assert!(rendered.contains(&localized_scope));
}
