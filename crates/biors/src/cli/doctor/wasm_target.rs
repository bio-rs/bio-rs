use super::report::{DoctorCheck, DoctorStatus};
use std::process::Command;

pub(super) fn wasm_target_check() -> DoctorCheck {
    let output = Command::new("rustup")
        .arg("target")
        .arg("list")
        .arg("--installed")
        .output();

    let installed_targets = match output {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).unwrap_or_default()
        }
        _ => {
            return DoctorCheck {
                capability: "wasm",
                name: "wasm32.target",
                status: DoctorStatus::Warn,
                message: "rustup target list --installed could not be run".to_string(),
                hint: Some("install rustup or add the wasm target manually"),
            };
        }
    };

    if installed_targets
        .lines()
        .any(|line| line.trim() == "wasm32-unknown-unknown")
    {
        DoctorCheck {
            capability: "wasm",
            name: "wasm32.target",
            status: DoctorStatus::Pass,
            message: "wasm32-unknown-unknown target is installed".to_string(),
            hint: None,
        }
    } else {
        DoctorCheck {
            capability: "wasm",
            name: "wasm32.target",
            status: DoctorStatus::Warn,
            message: "wasm32-unknown-unknown target is not installed".to_string(),
            hint: Some("install with: rustup target add wasm32-unknown-unknown"),
        }
    }
}
