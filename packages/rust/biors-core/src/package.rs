//! Portable package manifest inspection, validation, artifact checks, and runtime planning.

mod artifacts;
mod checksum;
mod runtime;
mod summary;
mod types;
mod validation;

pub use artifacts::{
    read_package_file, resolve_package_asset_path, resolve_package_path,
    validate_package_manifest_artifacts, validate_package_relative_path, PackageArtifactError,
};
pub use checksum::{is_sha256_checksum, sha256_digest};
pub use runtime::plan_runtime_bridge;
pub use summary::inspect_package_manifest;
pub use types::*;
pub use validation::validate_package_manifest;
