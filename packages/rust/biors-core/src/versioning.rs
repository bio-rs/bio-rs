//! Public schema versioning policies for package and pipeline contracts.

use crate::package::SchemaVersion;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaContractPolicy {
    pub contract: String,
    pub current: String,
    pub supported_versions: Vec<SupportedSchemaVersion>,
    pub deprecation: DeprecationPolicy,
    pub breaking_changes: BreakingChangePolicy,
    pub backward_compatibility_rules: Vec<String>,
    pub migration: MigrationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportedSchemaVersion {
    pub version: String,
    pub status: SchemaLifecycleStatus,
    pub introduced_in: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated_after: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_after: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaLifecycleStatus {
    Current,
    Supported,
    Deprecated,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeprecationPolicy {
    pub notice_minor_releases: u8,
    pub requires_docs: bool,
    pub requires_validator_warning_before_removal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreakingChangePolicy {
    pub require_new_schema_version: bool,
    pub forbidden_in_patch_release: bool,
    pub requires_migration_notes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPolicy {
    pub strategy: String,
    pub automatic_rewrite_allowed: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaMigrationPlan {
    pub from: String,
    pub to: String,
    pub compatibility: Compatibility,
    pub automatic: bool,
    pub required_steps: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Compatibility {
    BackwardCompatible,
    MigrationRequired,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineConfigVersion {
    #[serde(rename = "biors.pipeline.v0")]
    BiorsPipelineV0,
}

pub fn package_manifest_policy() -> SchemaContractPolicy {
    SchemaContractPolicy {
        contract: "package_manifest".to_string(),
        current: SchemaVersion::BiorsPackageV1.to_string(),
        supported_versions: vec![
            supported(
                "biors.package.v0",
                SchemaLifecycleStatus::Supported,
                "0.6.0",
            ),
            supported("biors.package.v1", SchemaLifecycleStatus::Current, "0.31.0"),
        ],
        deprecation: standard_deprecation_policy(),
        breaking_changes: standard_breaking_change_policy(),
        backward_compatibility_rules: vec![
            "Readers MUST read all supported manifest versions.".to_string(),
            "Writers SHOULD emit the current manifest version.".to_string(),
            "Patch releases MUST NOT remove accepted fields or enum values.".to_string(),
            "Minor releases MAY add optional fields to supported schemas.".to_string(),
        ],
        migration: MigrationPolicy {
            strategy: "schema-tagged parse followed by explicit migration plan".to_string(),
            automatic_rewrite_allowed: false,
            notes: vec![
                "Manifest migration must preserve package-relative artifact paths.".to_string(),
                "Missing research metadata must be supplied by the package author.".to_string(),
            ],
        },
    }
}

pub fn pipeline_config_policy() -> SchemaContractPolicy {
    SchemaContractPolicy {
        contract: "pipeline_config".to_string(),
        current: PipelineConfigVersion::BiorsPipelineV0.to_string(),
        supported_versions: vec![supported(
            "biors.pipeline.v0",
            SchemaLifecycleStatus::Current,
            "0.33.0",
        )],
        deprecation: standard_deprecation_policy(),
        breaking_changes: standard_breaking_change_policy(),
        backward_compatibility_rules: vec![
            "Pipeline config readers MUST reject unknown schema versions.".to_string(),
            "Pipeline config writers SHOULD emit the current config version.".to_string(),
            "Pipeline config migrations MUST be deterministic and inspectable.".to_string(),
        ],
        migration: MigrationPolicy {
            strategy: "schema-tagged parse before normalization and validation".to_string(),
            automatic_rewrite_allowed: true,
            notes: vec![
                "Config migrations may rewrite syntax but must not change biological input semantics."
                    .to_string(),
            ],
        },
    }
}

pub fn manifest_schema_compatibility(from: SchemaVersion, to: SchemaVersion) -> Compatibility {
    match (from, to) {
        (SchemaVersion::BiorsPackageV0, SchemaVersion::BiorsPackageV1) => {
            Compatibility::MigrationRequired
        }
        (left, right) if left == right => Compatibility::BackwardCompatible,
        _ => Compatibility::Unsupported,
    }
}

pub fn manifest_schema_migration_plan(
    from: SchemaVersion,
    to: SchemaVersion,
) -> Option<SchemaMigrationPlan> {
    match (from, to) {
        (SchemaVersion::BiorsPackageV0, SchemaVersion::BiorsPackageV1) => {
            Some(SchemaMigrationPlan {
                from: from.to_string(),
                to: to.to_string(),
                compatibility: Compatibility::MigrationRequired,
                automatic: false,
                required_steps: vec![
                    "Add package_layout with manifest, models, fixtures, docs, and any optional tokenizer/vocab directories.".to_string(),
                    "Add metadata.license with a redistributable license expression and optional checked file.".to_string(),
                    "Add metadata.citation with preferred citation text and optional checked citation file.".to_string(),
                    "Add metadata.model_card with path, summary, intended_use, limitations, and checksum when available.".to_string(),
                ],
            })
        }
        (left, right) if left == right => Some(SchemaMigrationPlan {
            from: from.to_string(),
            to: to.to_string(),
            compatibility: Compatibility::BackwardCompatible,
            automatic: true,
            required_steps: Vec::new(),
        }),
        _ => None,
    }
}

impl fmt::Display for PipelineConfigVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BiorsPipelineV0 => f.write_str("biors.pipeline.v0"),
        }
    }
}

fn supported(
    version: &str,
    status: SchemaLifecycleStatus,
    introduced_in: &str,
) -> SupportedSchemaVersion {
    SupportedSchemaVersion {
        version: version.to_string(),
        status,
        introduced_in: introduced_in.to_string(),
        deprecated_after: None,
        removed_after: None,
    }
}

fn standard_deprecation_policy() -> DeprecationPolicy {
    DeprecationPolicy {
        notice_minor_releases: 2,
        requires_docs: true,
        requires_validator_warning_before_removal: true,
    }
}

fn standard_breaking_change_policy() -> BreakingChangePolicy {
    BreakingChangePolicy {
        require_new_schema_version: true,
        forbidden_in_patch_release: true,
        requires_migration_notes: true,
    }
}
