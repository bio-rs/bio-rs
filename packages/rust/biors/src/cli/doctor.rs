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
    pub capability: &'static str,
    pub name: &'static str,
    pub status: DoctorStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<&'static str>,
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
        required_tool_check("core_cli", "rust.toolchain", &rustc, "rustc is available"),
        required_tool_check("core_cli", "cargo.toolchain", &cargo, "cargo is available"),
        wasm_target_check(),
        optional_command_check(
            "wasm",
            "wasm-pack.toolchain",
            "wasm-pack",
            "--version",
            "wasm-pack is available",
            "install with: cargo install wasm-pack --locked",
        ),
        optional_command_check(
            "wasm",
            "node.toolchain",
            "node",
            "--version",
            "Node.js is available for npm/WASM checks",
            "install Node.js before building or testing the WASM npm package",
        ),
        optional_command_check(
            "wasm",
            "npm.toolchain",
            "npm",
            "--version",
            "npm is available for package inspection",
            "install npm before checking the WASM package tarball",
        ),
        optional_command_check(
            "python",
            "python.toolchain",
            "python3",
            "--version",
            "Python is available for binding tests",
            "install Python 3 before building or testing the Python wheel",
        ),
        optional_command_check(
            "python",
            "maturin.toolchain",
            "maturin",
            "--version",
            "maturin is available for Python packaging",
            "install the pinned release maturin before building wheels",
        ),
        repo_file_check(
            "core_cli",
            "demo.dataset",
            "examples/launch-demo.fasta",
            "launch demo FASTA dataset is available",
        ),
        repo_file_check(
            "package",
            "package.fixture",
            "examples/protein-package/manifest.json",
            "package fixture manifest is available",
        ),
        repo_file_check(
            "package",
            "package.license_apache",
            "LICENSE-APACHE",
            "Apache license file is available for package artifacts",
        ),
        repo_file_check(
            "package",
            "package.license_mit",
            "LICENSE-MIT",
            "MIT license file is available for package artifacts",
        ),
        repo_file_check(
            "release",
            "release.workflow",
            ".github/workflows/release.yml",
            "release workflow is available",
        ),
        repo_file_check(
            "release",
            "release.security_audit",
            "scripts/check-security-audit.sh",
            "security audit script is available",
        ),
        optional_command_check(
            "release",
            "cargo-deny.toolchain",
            "cargo-deny",
            "--version",
            "cargo-deny is available for dependency audits",
            "install with: cargo install cargo-deny --locked",
        ),
        repo_file_check(
            "benchmark",
            "benchmark.docs_check",
            "scripts/check-benchmark-docs.sh",
            "benchmark documentation check is available",
        ),
        repo_file_check(
            "benchmark",
            "benchmark.workflow",
            ".github/workflows/benchmarks.yml",
            "benchmark workflow is available",
        ),
        repo_file_check(
            "benchmark",
            "benchmark.fasta_artifact",
            "benchmarks/fasta_vs_biopython.json",
            "FASTA benchmark artifact is available",
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

fn required_tool_check(
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

fn optional_command_check(
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

fn repo_file_check(
    capability: &'static str,
    name: &'static str,
    relative_path: &str,
    pass_message: &'static str,
) -> DoctorCheck {
    match find_repo_file(relative_path) {
        Some(path) => DoctorCheck {
            capability,
            name,
            status: DoctorStatus::Pass,
            message: format!("{pass_message}: {}", path.display()),
            hint: None,
        },
        None => DoctorCheck {
            capability,
            name,
            status: DoctorStatus::Warn,
            message: format!("{relative_path} was not found from the current checkout"),
            hint: Some("run doctor from a bio-rs checkout or verify the release artifact contents"),
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
