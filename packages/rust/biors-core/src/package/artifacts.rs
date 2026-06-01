use super::{
    read_package_file, validate_declared_layout, validate_package_manifest,
    validate_package_relative_path, PackageArtifactError, PackageManifest,
    PackageValidationIssueCode, PackageValidationReport,
};
use crate::hash::{is_sha256_checksum, sha256_bytes_digest};
use std::path::Path;

pub type ReferencedConfigValidator<'a> = dyn Fn(&Path) -> Result<(), ReferencedConfigError> + 'a;

pub fn validate_package_manifest_artifacts(
    manifest: &PackageManifest,
    base_dir: &Path,
) -> PackageValidationReport {
    validate_package_manifest_artifacts_with_pipeline_config_validator(manifest, base_dir, None)
}

pub fn validate_package_manifest_artifacts_with_pipeline_config_validator(
    manifest: &PackageManifest,
    base_dir: &Path,
    pipeline_config_validator: Option<&ReferencedConfigValidator<'_>>,
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

    validate_pipeline_configs(
        &mut report,
        "preprocessing",
        &manifest.preprocessing,
        base_dir,
        pipeline_config_validator,
    );
    validate_pipeline_configs(
        &mut report,
        "postprocessing",
        &manifest.postprocessing,
        base_dir,
        pipeline_config_validator,
    );

    if let Some(metadata) = &manifest.metadata {
        if let Some(file) = &metadata.license.file {
            validate_artifact(
                &mut report,
                "metadata.license.file",
                &file.path,
                file.checksum.as_deref(),
                base_dir,
            );
        }
        if let Some(file) = &metadata.citation.file {
            validate_artifact(
                &mut report,
                "metadata.citation.file",
                &file.path,
                file.checksum.as_deref(),
                base_dir,
            );
        }
        validate_artifact(
            &mut report,
            "metadata.model_card",
            &metadata.model_card.path,
            metadata.model_card.checksum.as_deref(),
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

    validate_declared_layout(&mut report, manifest);

    report.finish()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencedConfigError {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

impl ReferencedConfigError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        location: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            location,
        }
    }
}

fn validate_pipeline_configs(
    report: &mut PackageValidationReport,
    section: &str,
    steps: &[super::PipelineStep],
    base_dir: &Path,
    pipeline_config_validator: Option<&ReferencedConfigValidator<'_>>,
) {
    for (index, step) in steps.iter().enumerate() {
        if let Some(config) = &step.config {
            let field = format!("{section}[{index}].config");
            let artifact_valid = validate_artifact(
                report,
                &field,
                &config.path,
                config.checksum.as_deref(),
                base_dir,
            );
            if artifact_valid {
                validate_referenced_config(
                    report,
                    &field,
                    &config.path,
                    base_dir,
                    pipeline_config_validator,
                );
            }
        }
    }
}

fn validate_referenced_config(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    base_dir: &Path,
    pipeline_config_validator: Option<&ReferencedConfigValidator<'_>>,
) {
    let Some(validator) = pipeline_config_validator else {
        return;
    };
    let config_path = base_dir.join(path);
    if let Err(error) = validator(&config_path) {
        let mut message = format!(
            "{field}: pipeline config '{path}' is invalid: {}: {}",
            error.code, error.message
        );
        if let Some(location) = error.location {
            message.push_str(&format!(" at {location}"));
        }
        report.push_issue(
            PackageValidationIssueCode::InvalidPipelineConfig,
            field,
            &message,
        );
    }
}

fn validate_artifact(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    checksum: Option<&str>,
    base_dir: &Path,
) -> bool {
    if path.trim().is_empty() {
        return false;
    }

    validate_checksum_format(report, field, checksum);

    if let Err(error) = validate_package_relative_path(path) {
        report.push_issue(
            PackageValidationIssueCode::InvalidAssetPath,
            field,
            &error.to_string(),
        );
        return false;
    }

    match read_package_file(base_dir, path) {
        Ok(bytes) => validate_checksum_value(report, field, checksum, &bytes),
        Err(PackageArtifactError::PathEscape { .. }) => {
            report.push_issue(
                PackageValidationIssueCode::InvalidAssetPath,
                field,
                &format!("{field}: asset path '{path}' must stay inside the package root"),
            );
            false
        }
        Err(error) => {
            report.push_issue(
                PackageValidationIssueCode::AssetReadFailed,
                field,
                &format!("{field}: {error}"),
            );
            false
        }
    }
}

fn validate_checksum_format(
    report: &mut PackageValidationReport,
    field: &str,
    checksum: Option<&str>,
) {
    let Some(checksum) = checksum else {
        return;
    };
    if !is_sha256_checksum(checksum) {
        report.push_issue(
            PackageValidationIssueCode::InvalidChecksumFormat,
            &format!("{field}.checksum"),
            &format!("{field}.checksum must use sha256:<64 hex>"),
        );
    }
}

fn validate_checksum_value(
    report: &mut PackageValidationReport,
    field: &str,
    checksum: Option<&str>,
    bytes: &[u8],
) -> bool {
    let Some(checksum) = checksum else {
        return true;
    };
    if !is_sha256_checksum(checksum) {
        return false;
    }

    let actual = sha256_bytes_digest(bytes);
    if actual != checksum {
        report.push_issue(
            PackageValidationIssueCode::ChecksumMismatch,
            &format!("{field}.checksum"),
            &format!("{field}.checksum mismatch: expected '{checksum}' but computed '{actual}'"),
        );
        return false;
    }
    true
}
