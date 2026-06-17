use super::{
    open_package_file, read_package_file, validate_package_relative_path, PackageArtifactError,
    PackageValidationIssueCode, PackageValidationReport,
};
use crate::hash::{is_sha256_checksum, sha256_bytes_digest, Sha256ByteHasher};
use std::io::{BufReader, Read};
use std::path::Path;

pub(super) fn validated_artifact_bytes(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    checksum: Option<&str>,
    base_dir: &Path,
) -> Option<Vec<u8>> {
    if path.trim().is_empty() {
        return None;
    }

    let checksum_format_valid = validate_checksum_format(report, field, checksum);

    if let Err(error) = validate_package_relative_path(path) {
        push_relative_path_error(report, field, error);
        return None;
    }

    if !checksum_format_valid {
        open_artifact_reference(report, field, path, base_dir);
        return None;
    }

    match read_package_file(base_dir, path) {
        Ok(bytes) => validate_checksum_value(report, field, checksum, &bytes).then_some(bytes),
        Err(error) => {
            push_artifact_error(report, field, path, error);
            None
        }
    }
}

pub(super) fn validate_artifact_reference(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    checksum: Option<&str>,
    base_dir: &Path,
) -> bool {
    if path.trim().is_empty() {
        return false;
    }

    let checksum_format_valid = validate_checksum_format(report, field, checksum);

    if let Err(error) = validate_package_relative_path(path) {
        push_relative_path_error(report, field, error);
        return false;
    }

    if !checksum_format_valid {
        open_artifact_reference(report, field, path, base_dir);
        return false;
    }

    match checksum {
        Some(checksum) => validate_checksum_stream(report, field, path, checksum, base_dir),
        None => open_artifact_reference(report, field, path, base_dir),
    }
}

fn validate_checksum_format(
    report: &mut PackageValidationReport,
    field: &str,
    checksum: Option<&str>,
) -> bool {
    let Some(checksum) = checksum else {
        return true;
    };
    if !is_sha256_checksum(checksum) {
        report.push_issue(
            PackageValidationIssueCode::InvalidChecksumFormat,
            &format!("{field}.checksum"),
            &format!("{field}.checksum must use sha256:<64 hex>"),
        );
        return false;
    }
    true
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
    validate_checksum_match(report, field, checksum, &actual)
}

fn validate_checksum_stream(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    checksum: &str,
    base_dir: &Path,
) -> bool {
    match sha256_package_file_digest(base_dir, path) {
        Ok(actual) => validate_checksum_match(report, field, checksum, &actual),
        Err(error) => {
            push_artifact_error(report, field, path, error);
            false
        }
    }
}

fn open_artifact_reference(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    base_dir: &Path,
) -> bool {
    match open_package_file(base_dir, path) {
        Ok(_) => true,
        Err(error) => {
            push_artifact_error(report, field, path, error);
            false
        }
    }
}

fn sha256_package_file_digest(base_dir: &Path, path: &str) -> Result<String, PackageArtifactError> {
    let (file, resolved) = open_package_file(base_dir, path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256ByteHasher::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let count =
            reader
                .read(&mut buffer)
                .map_err(|error| PackageArtifactError::AssetReadFailed {
                    path: path.to_string(),
                    resolved: resolved.display().to_string(),
                    reason: error.to_string(),
                })?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hasher.finalize())
}

fn validate_checksum_match(
    report: &mut PackageValidationReport,
    field: &str,
    checksum: &str,
    actual: &str,
) -> bool {
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

fn push_relative_path_error(
    report: &mut PackageValidationReport,
    field: &str,
    error: PackageArtifactError,
) {
    report.push_issue(
        PackageValidationIssueCode::InvalidAssetPath,
        field,
        &error.to_string(),
    );
}

fn push_artifact_error(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    error: PackageArtifactError,
) {
    match error {
        PackageArtifactError::AbsolutePath { .. } | PackageArtifactError::EmptyPath => report
            .push_issue(
                PackageValidationIssueCode::InvalidAssetPath,
                field,
                &format!("{field}: {error}"),
            ),
        PackageArtifactError::PathEscape { .. } => report.push_issue(
            PackageValidationIssueCode::InvalidAssetPath,
            field,
            &format!("{field}: asset path '{path}' must stay inside the package root"),
        ),
        error => report.push_issue(
            PackageValidationIssueCode::AssetReadFailed,
            field,
            &format!("{field}: {error}"),
        ),
    }
}
