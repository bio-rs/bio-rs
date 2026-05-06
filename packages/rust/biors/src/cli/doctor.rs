use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize)]
pub(crate) struct DoctorReport {
    pub cli_version: &'static str,
    pub platform: PlatformReport,
    pub toolchain: ToolchainReport,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PlatformReport {
    pub os: &'static str,
    pub arch: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ToolchainReport {
    pub rustc: Option<String>,
    pub cargo: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DoctorCheck {
    pub name: &'static str,
    pub status: DoctorStatus,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DoctorStatus {
    Pass,
    Warn,
    Fail,
}

pub(crate) fn build_doctor_report() -> DoctorReport {
    let rustc = command_version("rustc", "--version");
    let cargo = command_version("cargo", "--version");
    let checks = vec![
        toolchain_check("rust.toolchain", &rustc, "rustc is available"),
        toolchain_check("cargo.toolchain", &cargo, "cargo is available"),
        wasm_target_check(),
        repo_file_check(
            "demo.dataset",
            "examples/launch-demo.fasta",
            "launch demo FASTA dataset is available",
        ),
        repo_file_check(
            "package.fixture",
            "examples/protein-package/manifest.json",
            "package fixture manifest is available",
        ),
    ];

    DoctorReport {
        cli_version: env!("CARGO_PKG_VERSION"),
        platform: PlatformReport {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        },
        toolchain: ToolchainReport { rustc, cargo },
        checks,
    }
}

fn command_version(program: &str, arg: &str) -> Option<String> {
    let output = Command::new(program).arg(arg).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn toolchain_check(
    name: &'static str,
    version: &Option<String>,
    pass_message: &'static str,
) -> DoctorCheck {
    match version {
        Some(version) => DoctorCheck {
            name,
            status: DoctorStatus::Pass,
            message: format!("{pass_message}: {version}"),
        },
        None => DoctorCheck {
            name,
            status: DoctorStatus::Fail,
            message: format!("{name} was not found on PATH"),
        },
    }
}

fn wasm_target_check() -> DoctorCheck {
    let output = Command::new("rustup")
        .arg("target")
        .arg("list")
        .arg("--installed")
        .output();

    let installed_targets = match output {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).unwrap_or_else(|_| String::new())
        }
        _ => {
            return DoctorCheck {
                name: "wasm32.target",
                status: DoctorStatus::Warn,
                message: "rustup target list --installed could not be run".to_string(),
            };
        }
    };

    if installed_targets
        .lines()
        .any(|line| line.trim() == "wasm32-unknown-unknown")
    {
        DoctorCheck {
            name: "wasm32.target",
            status: DoctorStatus::Pass,
            message: "wasm32-unknown-unknown target is installed".to_string(),
        }
    } else {
        DoctorCheck {
            name: "wasm32.target",
            status: DoctorStatus::Warn,
            message: "wasm32-unknown-unknown target is not installed".to_string(),
        }
    }
}

fn repo_file_check(
    name: &'static str,
    relative_path: &str,
    pass_message: &'static str,
) -> DoctorCheck {
    match find_repo_file(relative_path) {
        Some(path) => DoctorCheck {
            name,
            status: DoctorStatus::Pass,
            message: format!("{pass_message}: {}", path.display()),
        },
        None => DoctorCheck {
            name,
            status: DoctorStatus::Warn,
            message: format!("{relative_path} was not found from the current checkout"),
        },
    }
}

fn find_repo_file(relative_path: &str) -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let direct = current_dir.join(relative_path);
    if direct.exists() {
        return Some(direct);
    }

    let source_checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let source_path = source_checkout.join(relative_path);
    if source_path.exists() {
        return Some(source_path);
    }

    None
}
