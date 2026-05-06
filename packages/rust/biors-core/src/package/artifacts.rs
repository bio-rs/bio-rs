use super::{
    is_sha256_checksum, sha256_digest, validate_package_manifest, PackageManifest,
    PackageValidationIssueCode, PackageValidationReport,
};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Error type for package artifact path resolution and file reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageArtifactError {
    /// Asset path is empty.
    EmptyPath,
    /// Asset path is absolute.
    AbsolutePath { path: String },
    /// Asset path escapes the package root.
    PathEscape { path: String },
    /// Asset file could not be read.
    AssetReadFailed {
        path: String,
        resolved: String,
        reason: String,
    },
}

impl fmt::Display for PackageArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "asset path is required"),
            Self::AbsolutePath { path } => {
                write!(
                    f,
                    "asset path '{path}' must be relative to the package root"
                )
            }
            Self::PathEscape { path } => {
                write!(f, "asset path '{path}' must stay inside the package root")
            }
            Self::AssetReadFailed {
                path,
                resolved,
                reason,
            } => write!(f, "failed to read asset '{path}' at '{resolved}': {reason}"),
        }
    }
}

impl std::error::Error for PackageArtifactError {}

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

    report.finish()
}

/// Resolve a package-relative path under a package base directory without validation.
pub fn resolve_package_path(base_dir: &Path, relative_path: &str) -> PathBuf {
    base_dir.join(relative_path)
}

/// Validate and resolve a package-relative asset path under a package base directory.
pub fn resolve_package_asset_path(
    base_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, PackageArtifactError> {
    validate_package_relative_path(relative_path)?;
    Ok(resolve_package_path(base_dir, relative_path))
}

/// Read a package-relative asset after validating that the path stays inside the package root.
pub fn read_package_file(
    base_dir: &Path,
    relative_path: &str,
) -> Result<Vec<u8>, PackageArtifactError> {
    let resolved = resolve_package_asset_path(base_dir, relative_path)?;
    fs::read(&resolved).map_err(|error| PackageArtifactError::AssetReadFailed {
        path: relative_path.to_string(),
        resolved: resolved.display().to_string(),
        reason: error.to_string(),
    })
}

/// Validate that an asset path is relative and cannot escape the package root.
pub fn validate_package_relative_path(relative_path: &str) -> Result<(), PackageArtifactError> {
    let path = Path::new(relative_path);
    if relative_path.trim().is_empty() {
        return Err(PackageArtifactError::EmptyPath);
    }

    if path.is_absolute() {
        return Err(PackageArtifactError::AbsolutePath {
            path: relative_path.to_string(),
        });
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(PackageArtifactError::PathEscape {
            path: relative_path.to_string(),
        });
    }

    Ok(())
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
