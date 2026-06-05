use serde_json::Value;
use std::fs;
use std::process::Command;

mod cli_pipeline_support;
mod common;

use cli_pipeline_support::{repo_root, run_biors};
use common::TempDir;

#[test]
fn pipeline_writes_lockfile_with_package_provenance() {
    let temp = TempDir::new("biors-pipeline-lock");
    let lockfile = temp.path().join("pipeline.lock");
    let repo = repo_root();
    let config = repo.join("examples/protein-package/pipelines/protein.toml");
    let package = repo.join("examples/protein-package/manifest.json");
    let config_arg = config.to_string_lossy();
    let package_arg = package.to_string_lossy();
    let lock_arg = lockfile.to_string_lossy();

    let value = run_biors(
        &[
            "pipeline",
            "--config",
            &config_arg,
            "--package",
            &package_arg,
            "--write-lock",
            &lock_arg,
        ],
        &[],
    );

    assert_eq!(value["data"]["pipeline"], "config_pipeline.v0");
    assert!(lockfile.exists(), "pipeline.lock was not written");

    let lock: Value = serde_json::from_str(&fs::read_to_string(&lockfile).expect("read lockfile"))
        .expect("valid lockfile JSON");

    assert_eq!(lock["schema_version"], "biors.pipeline.lock.v0");
    assert_eq!(
        lock["pipeline_config"]["schema_version"],
        "biors.pipeline.v0"
    );
    assert_eq!(lock["package"]["name"], "protein-seed");
    assert_eq!(
        lock["package"]["model_sha256"],
        "sha256:2c1da72b15fab35bd6f1bb62f5037b936e26e6413a220fa9afe5a64bce0df68d"
    );
    assert_eq!(lock["package"]["runtime_backend"], "onnx-webgpu");
    assert_eq!(lock["package"]["backend_version"], "onnx-webgpu.v0");
    assert!(lock["hashes"]["vocabulary_sha256"]
        .as_str()
        .expect("vocab hash")
        .starts_with("sha256:"));
    assert!(lock["execution"]["input_hash"]
        .as_str()
        .expect("input hash")
        .starts_with("fnv1a64:"));
    assert!(lock.get("python_baseline").is_none());
}

#[test]
fn pipeline_lock_rejects_package_with_unrelated_config() {
    let temp = TempDir::new("biors-pipeline-lock-unrelated");
    let lockfile = temp.path().join("pipeline.lock");
    let repo = repo_root();
    let config = repo.join("examples/pipeline/protein.toml");
    let package = repo.join("examples/protein-package/manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("pipeline")
        .arg("--config")
        .arg(&config)
        .arg("--package")
        .arg(&package)
        .arg("--write-lock")
        .arg(&lockfile)
        .output()
        .expect("run biors pipeline lock generation");

    assert_eq!(output.status.code(), Some(2));
    assert!(!lockfile.exists(), "pipeline.lock should not be written");
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(
        value["error"]["code"],
        "pipeline.lock_config_not_in_package"
    );
    assert_eq!(value["error"]["location"], "pipeline.config");
}

#[test]
fn pipeline_lock_rejects_same_basename_config_outside_package() {
    let temp = TempDir::new("biors-pipeline-lock-same-basename");
    let lockfile = temp.path().join("pipeline.lock");
    let other_dir = temp.path().join("other");
    fs::create_dir_all(&other_dir).expect("create other config dir");
    let config = other_dir.join("protein.toml");
    fs::copy(
        repo_root().join("examples/protein-package/pipelines/protein.toml"),
        &config,
    )
    .expect("copy config");
    let package = repo_root().join("examples/protein-package/manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("pipeline")
        .arg("--config")
        .arg(&config)
        .arg("--package")
        .arg(&package)
        .arg("--write-lock")
        .arg(&lockfile)
        .output()
        .expect("run biors pipeline lock generation");

    assert_eq!(output.status.code(), Some(2));
    assert!(!lockfile.exists(), "pipeline.lock should not be written");
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(
        value["error"]["code"],
        "pipeline.lock_config_not_in_package"
    );
}

#[test]
fn checked_in_pipeline_lock_matches_current_generator() {
    let temp = TempDir::new("biors-pipeline-lock-current");
    let generated_lock = temp.path().join("pipeline.lock");
    let repo = repo_root();
    let lock_arg = generated_lock.to_string_lossy();

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .current_dir(&repo)
        .args([
            "pipeline",
            "--config",
            "examples/protein-package/pipelines/protein.toml",
            "--package",
            "examples/protein-package/manifest.json",
            "--write-lock",
            lock_arg.as_ref(),
        ])
        .output()
        .expect("run biors pipeline lock generation");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let checked_in: Value = serde_json::from_str(
        &fs::read_to_string(repo.join("examples/pipeline/pipeline.lock"))
            .expect("read checked-in lockfile"),
    )
    .expect("checked-in lockfile JSON");
    let generated: Value =
        serde_json::from_str(&fs::read_to_string(generated_lock).expect("read generated lockfile"))
            .expect("generated lockfile JSON");

    assert_eq!(generated, checked_in);
}
