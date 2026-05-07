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
pub(crate) fn validate_package_relative_path(
    relative_path: &str,
) -> Result<(), PackageArtifactError> {
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

fn resolve_package_path(base_dir: &Path, relative_path: &str) -> PathBuf {
    base_dir.join(relative_path)
}
