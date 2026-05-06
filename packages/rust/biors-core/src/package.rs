//! Portable package manifest inspection, validation, artifact checks, and runtime planning.

mod artifacts;
mod checksum;
mod manifest;
mod reports;
mod runtime;
mod summary;
mod types;
mod validation;

pub use artifacts::{
    read_package_file, resolve_package_asset_path, validate_package_manifest_artifacts,
    PackageArtifactError,
};
pub use checksum::{is_sha256_checksum, sha256_digest};
pub use manifest::{
    DataShape, ModelArtifact, PackageFixture, PackageManifest, PipelineStep, RuntimeTarget,
    TokenAsset,
};
pub use reports::{
    PackageLayoutSummary, PackageManifestSummary, PackageValidationIssue,
    PackageValidationIssueCode, PackageValidationReport, RuntimeBridgeReport,
};
pub use runtime::plan_runtime_bridge;
pub use summary::inspect_package_manifest;
pub use types::{DataType, ModelFormat, RuntimeBackend, RuntimeTargetPlatform, SchemaVersion};
pub use validation::validate_package_manifest;
