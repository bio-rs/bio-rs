use super::report::{DoctorCheck, DoctorStatus};
use std::process::Command;

pub(super) fn command_version(program: &str, arg: &str) -> Option<String> {
    let output = Command::new(program).arg(arg).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn required_tool_check(
    capability: &'static str,
    name: &'static str,
    version: &Option<String>,
    pass_message: &'static str,
) -> DoctorCheck {
    match version {
        Some(version) => DoctorCheck {
            capability,
            name,
            status: DoctorStatus::Pass,
            message: format!("{pass_message}: {version}"),
            hint: None,
        },
        None => DoctorCheck {
            capability,
            name,
            status: DoctorStatus::Fail,
            message: format!("{name} was not found on PATH"),
            hint: None,
        },
    }
}

pub(super) fn optional_command_check(
    capability: &'static str,
    name: &'static str,
    program: &str,
    arg: &str,
    pass_message: &'static str,
    hint: &'static str,
) -> DoctorCheck {
    match command_version(program, arg) {
        Some(version) => DoctorCheck {
            capability,
            name,
            status: DoctorStatus::Pass,
            message: format!("{pass_message}: {version}"),
            hint: None,
        },
        None => DoctorCheck {
            capability,
            name,
            status: DoctorStatus::Warn,
            message: format!("{program} {arg} could not be run"),
            hint: Some(hint),
        },
    }
}
