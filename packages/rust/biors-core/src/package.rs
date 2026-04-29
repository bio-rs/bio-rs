use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};

mod types;
pub use types::*;

pub fn inspect_package_manifest(manifest: &PackageManifest) -> PackageManifestSummary {
    PackageManifestSummary {
        schema_version: manifest.schema_version,
        name: manifest.name.clone(),
        model_format: manifest.model.format,
        has_model_checksum: manifest.model.checksum.is_some(),
        tokenizer: manifest
            .tokenizer
            .as_ref()
            .map(|tokenizer| tokenizer.name.clone()),
        vocab: manifest.vocab.as_ref().map(|vocab| vocab.name.clone()),
        runtime_backend: manifest.runtime.backend,
        runtime_target: manifest.runtime.target,
        preprocessing_steps: manifest.preprocessing.len(),
        postprocessing_steps: manifest.postprocessing.len(),
        fixtures: manifest.fixtures.len(),
    }
}

pub fn validate_package_manifest(manifest: &PackageManifest) -> PackageValidationReport {
    let mut report = PackageValidationReport::default();

    push_required_issue(&mut report, "name", &manifest.name);
    push_required_issue(&mut report, "model.path", &manifest.model.path);

    if manifest.fixtures.is_empty() {
        report.push_issue(
            PackageValidationIssueCode::MissingFixture,
            "fixtures",
            "fixtures must include at least one fixture",
        );
    }

    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        push_required_issue(
            &mut report,
            &format!("fixtures[{index}].name"),
            &fixture.name,
        );
        push_required_issue(
            &mut report,
            &format!("fixtures[{index}].input"),
            &fixture.input,
        );
        push_required_issue(
            &mut report,
            &format!("fixtures[{index}].expected_output"),
            &fixture.expected_output,
        );
    }

    if let Some(input) = &manifest.expected_input {
        validate_shape(&mut report, "expected_input", input);
    }

    if let Some(output) = &manifest.expected_output {
        validate_shape(&mut report, "expected_output", output);
    }

    report.valid = report.structured_issues.is_empty();
    report
}

pub fn validate_package_manifest_artifacts(
    manifest: &PackageManifest,
    base_dir: &Path,
) -> PackageValidationReport {
    let mut report = validate_package_manifest(manifest);
    validate_artifact(
        &mut report,
        "model",
        &manifest.model.path,
        manifest.model.checksum.as_deref(),
        base_dir,
    );

    if let Some(tokenizer) = &manifest.tokenizer {
        validate_artifact(
            &mut report,
            "tokenizer",
            &tokenizer.path,
            tokenizer.checksum.as_deref(),
            base_dir,
        );
    }

    if let Some(vocab) = &manifest.vocab {
        validate_artifact(
            &mut report,
            "vocab",
            &vocab.path,
            vocab.checksum.as_deref(),
            base_dir,
        );
    }

    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        validate_artifact(
            &mut report,
            &format!("fixtures[{index}].input"),
            &fixture.input,
            fixture.input_hash.as_deref(),
            base_dir,
        );
        validate_artifact(
            &mut report,
            &format!("fixtures[{index}].expected_output"),
            &fixture.expected_output,
            fixture.expected_output_hash.as_deref(),
            base_dir,
        );
    }

    report.valid = report.structured_issues.is_empty();
    report
}

pub fn plan_runtime_bridge(manifest: &PackageManifest) -> RuntimeBridgeReport {
    let blocking_issues = validate_package_manifest(manifest).issues;

    RuntimeBridgeReport {
        ready: blocking_issues.is_empty(),
        backend: manifest.runtime.backend,
        target: manifest.runtime.target,
        execution_provider: "webgpu".to_string(),
        blocking_issues,
    }
}

fn push_required_issue(report: &mut PackageValidationReport, field: &str, value: &str) {
    if value.trim().is_empty() {
        report.push_issue(
            PackageValidationIssueCode::RequiredField,
            field,
            &format!("{field} is required"),
        );
    }
}

fn validate_shape(report: &mut PackageValidationReport, field: &str, shape: &DataShape) {
    if shape.shape.is_empty() {
        report.push_issue(
            PackageValidationIssueCode::InvalidShape,
            &format!("{field}.shape"),
            &format!("{field}.shape must include at least one dimension"),
        );
    }
}

pub fn resolve_package_path(base_dir: &Path, relative_path: &str) -> PathBuf {
    base_dir.join(relative_path)
}

pub fn resolve_package_asset_path(base_dir: &Path, relative_path: &str) -> Result<PathBuf, String> {
    validate_package_relative_path(relative_path)?;
    Ok(resolve_package_path(base_dir, relative_path))
}

pub fn read_package_file(base_dir: &Path, relative_path: &str) -> Result<Vec<u8>, String> {
    let resolved = resolve_package_asset_path(base_dir, relative_path)?;
    fs::read(&resolved).map_err(|error| {
        format!(
            "failed to read asset '{}' at '{}': {error}",
            relative_path,
            resolved.display()
        )
    })
}

pub fn validate_package_relative_path(relative_path: &str) -> Result<(), String> {
    let path = Path::new(relative_path);
    if relative_path.trim().is_empty() {
        return Err("asset path is required".to_string());
    }

    if path.is_absolute() {
        return Err(format!(
            "asset path '{relative_path}' must be relative to the package root"
        ));
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!(
            "asset path '{relative_path}' must stay inside the package root"
        ));
    }

    Ok(())
}

pub fn sha256_digest(bytes: &[u8]) -> String {
    let normalized = canonical_hash_bytes(bytes);
    let digest = Sha256::digest(&normalized);
    format!("sha256:{digest:x}")
}

pub fn is_sha256_checksum(checksum: &str) -> bool {
    let Some(hex) = checksum.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_artifact(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    checksum: Option<&str>,
    base_dir: &Path,
) {
    if path.trim().is_empty() {
        return;
    }

    if let Some(checksum) = checksum {
        if !is_sha256_checksum(checksum) {
            report.push_issue(
                PackageValidationIssueCode::InvalidChecksumFormat,
                &format!("{field}.checksum"),
                &format!("{field}.checksum must use sha256:<64 hex>"),
            );
        }
    }

    if let Err(error) = validate_package_relative_path(path) {
        report.push_issue(PackageValidationIssueCode::InvalidAssetPath, field, &error);
        return;
    }

    match read_package_file(base_dir, path) {
        Ok(bytes) => {
            if let Some(checksum) = checksum {
                if is_sha256_checksum(checksum) {
                    let actual = sha256_digest(&bytes);
                    if actual != checksum {
                        report.push_issue(
                            PackageValidationIssueCode::ChecksumMismatch,
                            &format!("{field}.checksum"),
                            &format!(
                                "{field}.checksum mismatch: expected '{checksum}' but computed '{actual}'"
                            ),
                        );
                    }
                }
            }
        }
        Err(error) => report.push_issue(
            PackageValidationIssueCode::AssetReadFailed,
            field,
            &format!("{field}: {error}"),
        ),
    }
}

impl PackageValidationReport {
    fn push_issue(&mut self, code: PackageValidationIssueCode, field: &str, message: &str) {
        self.issues.push(message.to_string());
        self.structured_issues.push(PackageValidationIssue {
            code,
            field: field.to_string(),
            message: message.to_string(),
        });
    }
}

impl Default for PackageValidationReport {
    fn default() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
            structured_issues: Vec::new(),
        }
    }
}

fn canonical_hash_bytes(bytes: &[u8]) -> Vec<u8> {
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(json) => serde_json::to_vec(&json).unwrap_or_else(|_| bytes.to_vec()),
        Err(_) => bytes.to_vec(),
    }
}
