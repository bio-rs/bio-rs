use super::{PackageManifest, SchemaVersion};
use crate::{
    verification::{diff_output_bytes, OutputDiffReport},
    versioning::{manifest_schema_compatibility, manifest_schema_migration_plan, Compatibility},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Migration guidance for one package manifest schema transition.
pub struct PackageSchemaMigrationReport {
    pub package: String,
    pub from: String,
    pub to: String,
    pub compatibility: Compatibility,
    pub automatic: bool,
    pub required_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Schema compatibility report for two package manifests.
pub struct PackageSchemaCompatibilityReport {
    pub left_path: String,
    pub right_path: String,
    pub left_package: String,
    pub right_package: String,
    pub left_schema_version: String,
    pub right_schema_version: String,
    pub compatibility: Compatibility,
    pub schema_compatible: bool,
    pub migration_required: bool,
    pub same_package_name: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Canonical manifest diff annotated with package schema compatibility.
pub struct PackageManifestDiffReport {
    pub left_path: String,
    pub right_path: String,
    pub left_package: String,
    pub right_package: String,
    pub left_schema_version: String,
    pub right_schema_version: String,
    pub compatibility: Compatibility,
    pub same_package_name: bool,
    pub diff: OutputDiffReport,
}

pub fn plan_package_schema_migration(
    manifest: &PackageManifest,
    to: SchemaVersion,
) -> Option<PackageSchemaMigrationReport> {
    let plan = manifest_schema_migration_plan(manifest.schema_version, to)?;
    Some(PackageSchemaMigrationReport {
        package: manifest.name.clone(),
        from: plan.from,
        to: plan.to,
        compatibility: plan.compatibility,
        automatic: plan.automatic,
        required_steps: plan.required_steps,
    })
}

pub fn compare_package_manifest_schemas(
    left_path: &str,
    right_path: &str,
    left: &PackageManifest,
    right: &PackageManifest,
) -> PackageSchemaCompatibilityReport {
    let compatibility = manifest_schema_compatibility(left.schema_version, right.schema_version);
    PackageSchemaCompatibilityReport {
        left_path: left_path.to_string(),
        right_path: right_path.to_string(),
        left_package: left.name.clone(),
        right_package: right.name.clone(),
        left_schema_version: left.schema_version.to_string(),
        right_schema_version: right.schema_version.to_string(),
        compatibility,
        schema_compatible: compatibility != Compatibility::Unsupported,
        migration_required: compatibility == Compatibility::MigrationRequired,
        same_package_name: left.name == right.name,
    }
}

pub fn diff_package_manifests(
    left_path: &str,
    right_path: &str,
    left: &PackageManifest,
    right: &PackageManifest,
    left_bytes: &[u8],
    right_bytes: &[u8],
) -> PackageManifestDiffReport {
    let compatibility = manifest_schema_compatibility(left.schema_version, right.schema_version);
    PackageManifestDiffReport {
        left_path: left_path.to_string(),
        right_path: right_path.to_string(),
        left_package: left.name.clone(),
        right_package: right.name.clone(),
        left_schema_version: left.schema_version.to_string(),
        right_schema_version: right.schema_version.to_string(),
        compatibility,
        same_package_name: left.name == right.name,
        diff: diff_output_bytes(left_path, right_path, left_bytes, right_bytes),
    }
}
