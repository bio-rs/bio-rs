use super::command_probe::{command_version, optional_command_check, required_tool_check};
use super::report::{DoctorReport, PlatformReport, ToolchainReport};
use super::repository_file::repo_file_check;
use super::wasm_target::wasm_target_check;

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
            "testdata/sequences/launch-demo.fasta",
            "launch demo FASTA dataset is available",
        ),
        repo_file_check(
            "package",
            "package.fixture",
            "testdata/protein-package/manifest.json",
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
            "benchmark artifact check is available",
        ),
        repo_file_check(
            "benchmark",
            "benchmark.workflow",
            ".github/workflows/benchmarks.yml",
            "benchmark workflow is available",
        ),
        repo_file_check(
            "benchmark",
            "benchmark.cli_artifact",
            "benchmarks/cli_surfaces.json",
            "CLI benchmark artifact is available",
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
