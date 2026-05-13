use biors_core::package::PipelineConfigVersion;
use biors_core::package::SchemaVersion;
use biors_core::versioning::{
    manifest_schema_compatibility, manifest_schema_migration_plan, package_manifest_policy,
    pipeline_config_policy, Compatibility, SchemaLifecycleStatus,
};

#[test]
fn package_manifest_policy_defines_current_supported_and_deprecation_rules() {
    let policy = package_manifest_policy();

    assert_eq!(policy.contract, "package_manifest");
    assert_eq!(policy.current, "biors.package.v1");
    assert!(policy
        .supported_versions
        .iter()
        .any(|version| version.version == "biors.package.v0"
            && version.status == SchemaLifecycleStatus::Supported));
    assert!(policy
        .supported_versions
        .iter()
        .any(|version| version.version == "biors.package.v1"
            && version.status == SchemaLifecycleStatus::Current));
    assert!(policy.deprecation.notice_minor_releases >= 2);
    assert!(policy.breaking_changes.require_new_schema_version);
    assert!(policy
        .backward_compatibility_rules
        .iter()
        .any(|rule| rule.contains("read all supported manifest versions")));
}

#[test]
fn manifest_compatibility_identifies_supported_migration_path() {
    assert_eq!(
        manifest_schema_compatibility(SchemaVersion::BiorsPackageV0, SchemaVersion::BiorsPackageV1),
        Compatibility::MigrationRequired
    );
    assert_eq!(
        manifest_schema_compatibility(SchemaVersion::BiorsPackageV1, SchemaVersion::BiorsPackageV1),
        Compatibility::BackwardCompatible
    );

    let plan = manifest_schema_migration_plan(
        SchemaVersion::BiorsPackageV0,
        SchemaVersion::BiorsPackageV1,
    )
    .expect("v0 to v1 plan");

    assert_eq!(plan.from, "biors.package.v0");
    assert_eq!(plan.to, "biors.package.v1");
    assert!(plan
        .required_steps
        .iter()
        .any(|step| step.contains("package_layout")));
    assert!(plan
        .required_steps
        .iter()
        .any(|step| step.contains("metadata")));
}

#[test]
fn pipeline_config_policy_is_versioned_before_config_mvp() {
    let policy = pipeline_config_policy();

    assert_eq!(policy.contract, "pipeline_config");
    assert_eq!(policy.current, "biors.pipeline.v0");
    assert_eq!(
        PipelineConfigVersion::BiorsPipelineV0.to_string(),
        "biors.pipeline.v0"
    );
    assert!(policy.migration.strategy.contains("schema-tagged parse"));
}
