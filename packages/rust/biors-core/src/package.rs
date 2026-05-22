//! Portable package manifest inspection, validation, artifact checks, and runtime planning.

mod artifacts;
mod checksum;
mod layout;
mod manifest;
mod paths;
mod reports;
mod runtime;
mod summary;
mod tooling;
mod types;
mod validation;

pub use artifacts::validate_package_manifest_artifacts;
pub use checksum::{is_sha256_checksum, sha256_digest};
pub use manifest::{
    CitationMetadata, DataShape, DocumentArtifact, LicenseMetadata, ModelArtifact,
    ModelCardMetadata, PackageDirectoryLayout, PackageFixture, PackageManifest, PackageMetadata,
    PipelineConfigArtifact, PipelineStep, RuntimeTarget, TokenAsset,
};
pub use reports::{
    PackageDirectoryLayoutSummary, PackageLayoutSummary, PackageManifestSummary,
    PackageMetadataSummary, PackageValidationIssue, PackageValidationIssueCode,
    PackageValidationReport, RuntimeBridgeReport,
};
pub use runtime::plan_runtime_bridge;
pub use summary::inspect_package_manifest;
pub use tooling::{
    compare_package_manifest_schemas, convert_package_manifest, diff_package_manifests,
    plan_package_schema_migration, PackageManifestConversionError, PackageManifestConversionInput,
    PackageManifestConversionOutput, PackageManifestDiffReport, PackageSchemaCompatibilityReport,
    PackageSchemaMigrationReport,
};
pub use types::{
    DataType, ModelFormat, PipelineConfigVersion, RuntimeBackend, RuntimeTargetPlatform,
    SchemaVersion,
};
pub use validation::validate_package_manifest;

pub(crate) use layout::validate_declared_layout;
pub(crate) use paths::validate_package_relative_path;
pub use paths::{read_package_file, resolve_package_asset_path, PackageArtifactError};
