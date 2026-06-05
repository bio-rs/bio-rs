use super::{PackageDirectoryLayout, PackageManifest, PackageMetadata, SchemaVersion};
use crate::{
    error::Diagnostic,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifestConversionInput {
    pub package_layout: PackageDirectoryLayout,
    pub metadata: PackageMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Converted manifest plus provenance for schema-aware package conversion.
pub struct PackageManifestConversionOutput {
    pub report: PackageManifestConversionReport,
    pub manifest: PackageManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Conversion report for a package manifest schema transition.
pub struct PackageManifestConversionReport {
    pub package: String,
    pub from: String,
    pub to: String,
    pub compatibility: Compatibility,
    pub converted: bool,
    pub automatic: bool,
    pub metadata_supplied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageManifestConversionError {
    MissingConversionInput { from: String, to: String },
    Unsupported { from: String, to: String },
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

pub fn convert_package_manifest(
    manifest: &PackageManifest,
    to: SchemaVersion,
    input: Option<PackageManifestConversionInput>,
) -> Result<PackageManifestConversionOutput, PackageManifestConversionError> {
    let Some(plan) = manifest_schema_migration_plan(manifest.schema_version, to) else {
        return Err(PackageManifestConversionError::Unsupported {
            from: manifest.schema_version.to_string(),
            to: to.to_string(),
        });
    };

    let (converted_manifest, converted, metadata_supplied) = match plan.compatibility {
        Compatibility::BackwardCompatible => (manifest.clone(), false, manifest.metadata.is_some()),
        Compatibility::MigrationRequired => {
            let Some(input) = input else {
                return Err(PackageManifestConversionError::MissingConversionInput {
                    from: manifest.schema_version.to_string(),
                    to: to.to_string(),
                });
            };
            let mut converted = manifest.clone();
            converted.schema_version = to;
            converted.package_layout = Some(input.package_layout);
            converted.metadata = Some(input.metadata);
            (converted, true, true)
        }
        Compatibility::Unsupported => {
            return Err(PackageManifestConversionError::Unsupported {
                from: manifest.schema_version.to_string(),
                to: to.to_string(),
            });
        }
    };

    Ok(PackageManifestConversionOutput {
        report: PackageManifestConversionReport {
            package: manifest.name.clone(),
            from: plan.from,
            to: plan.to,
            compatibility: plan.compatibility,
            converted,
            automatic: plan.automatic,
            metadata_supplied,
            output_path: None,
            manifest_sha256: None,
        },
        manifest: converted_manifest,
    })
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

impl std::fmt::Display for PackageManifestConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingConversionInput { from, to } => write!(
                f,
                "conversion from '{from}' to '{to}' requires v1 metadata and layout input"
            ),
            Self::Unsupported { from, to } => {
                write!(f, "no package manifest conversion from '{from}' to '{to}'")
            }
        }
    }
}

impl std::error::Error for PackageManifestConversionError {}

impl Diagnostic for PackageManifestConversionError {
    fn code(&self) -> &'static str {
        match self {
            Self::MissingConversionInput { .. } => "package.conversion_missing_metadata",
            Self::Unsupported { .. } => "package.conversion_unsupported",
        }
    }

    fn message(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests;
