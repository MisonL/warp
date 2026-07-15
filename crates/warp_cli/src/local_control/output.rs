//! Output rendering helpers for `warpctrl`.
use std::io::Write as _;

use local_control::protocol::{ControlError, ErrorCode};
use serde::Serialize;

use crate::agent::OutputFormat;
use crate::localization::text_with_args;

/// JSON/NDJSON error payload emitted by `warpctrl`.
#[derive(Serialize)]
pub(crate) struct ErrorSummary<'a> {
    pub ok: bool,
    pub error: &'a ControlError,
}

pub(super) fn write_control_error(
    error: &ControlError,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match output_format {
        OutputFormat::Json => write_json(&ErrorSummary { ok: false, error }),
        OutputFormat::Ndjson => write_json_line(&ErrorSummary { ok: false, error }),
        OutputFormat::Pretty | OutputFormat::Text => {
            for line in control_error_lines(error) {
                eprintln!("{line}");
            }
            Ok(())
        }
    }
}

fn control_error_lines(error: &ControlError) -> Vec<String> {
    let code = error.code.to_string();
    let mut lines = vec![text_with_args(
        "warpctrl.error.summary",
        &[("code", &code), ("message", &error.message)],
    )];
    if let Some(details) = &error.details {
        lines.push(text_with_args(
            "warpctrl.error.details",
            &[("details", details)],
        ));
    }
    lines
}

#[cfg(test)]
pub(crate) fn control_error_lines_for_test(error: &ControlError) -> Vec<String> {
    control_error_lines(error)
}

pub(super) fn write_json(value: &impl Serialize) -> Result<(), ControlError> {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    serde_json::to_writer_pretty(&mut lock, value).map_err(write_error)?;
    writeln!(&mut lock).map_err(write_error)?;
    Ok(())
}
pub(super) fn write_json_line(value: &impl Serialize) -> Result<(), ControlError> {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    serde_json::to_writer(&mut lock, value).map_err(write_error)?;
    writeln!(&mut lock).map_err(write_error)?;
    Ok(())
}
fn write_error(error: impl std::error::Error) -> ControlError {
    ControlError::with_details(
        ErrorCode::Internal,
        "failed to write local-control output",
        error.to_string(),
    )
}
