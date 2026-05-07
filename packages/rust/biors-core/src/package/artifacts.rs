use super::{
    is_sha256_checksum, read_package_file, sha256_digest, validate_declared_layout,
    validate_package_manifest, validate_package_relative_path, PackageManifest,
    PackageValidationIssueCode, PackageValidationReport,
};
use std::path::Path;

/// Validate manifest fields and all package-relative artifact paths and checksums.
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

    validate_pipeline_configs(
        &mut report,
        "preprocessing",
        &manifest.preprocessing,
        base_dir,
    );
    validate_pipeline_configs(
        &mut report,
        "postprocessing",
        &manifest.postprocessing,
        base_dir,
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

fn validate_pipeline_configs(
    report: &mut PackageValidationReport,
    section: &str,
    steps: &[super::PipelineStep],
    base_dir: &Path,
) {
    for (index, step) in steps.iter().enumerate() {
        if let Some(config) = &step.config {
            validate_artifact(
                report,
                &format!("{section}[{index}].config"),
                &config.path,
                config.checksum.as_deref(),
                base_dir,
            );
        }
    }
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

    validate_checksum_format(report, field, checksum);

    if let Err(error) = validate_package_relative_path(path) {
        report.push_issue(
            PackageValidationIssueCode::InvalidAssetPath,
            field,
            &error.to_string(),
        );
        return;
    }

    match read_package_file(base_dir, path) {
        Ok(bytes) => validate_checksum_value(report, field, checksum, &bytes),
        Err(error) => report.push_issue(
            PackageValidationIssueCode::AssetReadFailed,
            field,
            &format!("{field}: {error}"),
        ),
    }
}

fn validate_checksum_format(
    report: &mut PackageValidationReport,
    field: &str,
    checksum: Option<&str>,
) {
    if let Some(checksum) = checksum {
        if !is_sha256_checksum(checksum) {
            report.push_issue(
                PackageValidationIssueCode::InvalidChecksumFormat,
                &format!("{field}.checksum"),
                &format!("{field}.checksum must use sha256:<64 hex>"),
            );
        }
    }
}

fn validate_checksum_value(
    report: &mut PackageValidationReport,
    field: &str,
    checksum: Option<&str>,
    bytes: &[u8],
) {
    let Some(checksum) = checksum else {
        return;
    };
    if !is_sha256_checksum(checksum) {
        return;
    }

    let actual = sha256_digest(bytes);
    if actual != checksum {
        report.push_issue(
            PackageValidationIssueCode::ChecksumMismatch,
            &format!("{field}.checksum"),
            &format!("{field}.checksum mismatch: expected '{checksum}' but computed '{actual}'"),
        );
    }
}
