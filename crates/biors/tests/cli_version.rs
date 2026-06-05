use std::process::Command;

mod common;

#[test]
fn cli_version_flag_reports_published_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--version")
        .output()
        .expect("run biors --version");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("version output is UTF-8");
    assert_eq!(
        stdout.trim(),
        format!("biors {}", env!("CARGO_PKG_VERSION"))
    );
}
