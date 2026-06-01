//! Portable package manifest inspection, validation, artifact checks, and runtime planning.

mod artifact_content;
mod artifacts;
mod layout;
mod manifest;
mod paths;
mod reports;
mod runtime;
mod summary;
mod tooling;
mod types;
mod validation;

pub use artifact_content::{ReferencedConfigError, ReferencedConfigValidator};
pub use artifacts::{
    validate_package_manifest_artifacts,
    validate_package_manifest_artifacts_with_pipeline_config_validator,
};
pub use manifest::{
    CitationMetadata, DataShape, DocumentArtifact, LicenseMetadata, ModelArtifact,
    ModelArtifactMetadata, ModelCardMetadata, PackageDirectoryLayout, PackageFixture,
    PackageManifest, PackageMetadata, PipelineConfigArtifact, PipelineStep, RuntimeTarget,
    TokenAsset,
};
pub use reports::{
    BackendCapabilitiesSummary, BackendCompatibilityCheck, BenchmarkEvidence, BenchmarkMetric,
    ModelArtifactMetadataSummary, PackageDirectoryLayoutSummary, PackageLayoutSummary,
    PackageManifestSummary, PackageMetadataSummary, PackageValidationIssue,
    PackageValidationIssueCode, PackageValidationReport, RegressionBaseline, RuntimeBridgeReport,
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
