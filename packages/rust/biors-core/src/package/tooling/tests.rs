use crate::error::Diagnostic;
use crate::package::{
    compare_package_manifest_schemas, convert_package_manifest, diff_package_manifests,
    plan_package_schema_migration, CitationMetadata, LicenseMetadata, ModelArtifact,
    ModelCardMetadata, ModelFormat, PackageDirectoryLayout, PackageManifest,
    PackageManifestConversionError, PackageManifestConversionInput, PackageMetadata,
    RuntimeBackend, RuntimeTarget, RuntimeTargetPlatform, SchemaVersion,
};
use crate::versioning::Compatibility;

fn minimal_manifest(name: &str, schema_version: SchemaVersion) -> PackageManifest {
    PackageManifest {
        schema_version,
        name: name.to_string(),
        package_layout: None,
        metadata: None,
        model: ModelArtifact {
            format: ModelFormat::Onnx,
            path: "model.onnx".to_string(),
            checksum: None,
        },
        tokenizer: None,
        vocab: None,
        preprocessing: vec![],
        postprocessing: vec![],
        runtime: RuntimeTarget {
            backend: RuntimeBackend::OnnxWebgpu,
            target: RuntimeTargetPlatform::BrowserWasmWebgpu,
            version: None,
        },
        expected_input: None,
        expected_output: None,
        fixtures: vec![],
    }
}

fn manifest_v1(name: &str) -> PackageManifest {
    let mut manifest = minimal_manifest(name, SchemaVersion::BiorsPackageV1);
    manifest.package_layout = Some(PackageDirectoryLayout {
        manifest: "manifest.json".to_string(),
        models: "models".to_string(),
        tokenizers: None,
        vocabs: None,
        pipelines: None,
        fixtures: "fixtures".to_string(),
        observed: None,
        docs: "docs".to_string(),
    });
    manifest.metadata = Some(PackageMetadata {
        license: LicenseMetadata {
            expression: "MIT".to_string(),
            file: None,
        },
        citation: CitationMetadata {
            preferred_citation: "Test et al. 2024".to_string(),
            doi: None,
            file: None,
        },
        model_card: ModelCardMetadata {
            path: "docs/model-card.md".to_string(),
            checksum: None,
            summary: "A test model".to_string(),
            intended_use: vec!["testing".to_string()],
            limitations: vec!["none".to_string()],
        },
    });
    manifest
}

fn conversion_input() -> PackageManifestConversionInput {
    PackageManifestConversionInput {
        package_layout: PackageDirectoryLayout {
            manifest: "manifest.json".to_string(),
            models: "models".to_string(),
            tokenizers: None,
            vocabs: None,
            pipelines: None,
            fixtures: "fixtures".to_string(),
            observed: None,
            docs: "docs".to_string(),
        },
        metadata: PackageMetadata {
            license: LicenseMetadata {
                expression: "MIT".to_string(),
                file: None,
            },
            citation: CitationMetadata {
                preferred_citation: "Test et al. 2024".to_string(),
                doi: None,
                file: None,
            },
            model_card: ModelCardMetadata {
                path: "docs/model-card.md".to_string(),
                checksum: None,
                summary: "A test model".to_string(),
                intended_use: vec!["testing".to_string()],
                limitations: vec!["none".to_string()],
            },
        },
    }
}

#[test]
fn test_plan_migration_v0_to_v1() {
    let manifest = minimal_manifest("test-pkg", SchemaVersion::BiorsPackageV0);
    let report = plan_package_schema_migration(&manifest, SchemaVersion::BiorsPackageV1)
        .expect("should return a migration plan");
    assert_eq!(report.package, "test-pkg");
    assert_eq!(report.from, "biors.package.v0");
    assert_eq!(report.to, "biors.package.v1");
    assert_eq!(report.compatibility, Compatibility::MigrationRequired);
    assert!(!report.automatic);
    assert_eq!(report.required_steps.len(), 4);
}

#[test]
fn test_plan_migration_same_version() {
    let manifest = minimal_manifest("test-pkg", SchemaVersion::BiorsPackageV0);
    let report = plan_package_schema_migration(&manifest, SchemaVersion::BiorsPackageV0)
        .expect("should return a migration plan");
    assert_eq!(report.from, "biors.package.v0");
    assert_eq!(report.to, "biors.package.v0");
    assert_eq!(report.compatibility, Compatibility::BackwardCompatible);
    assert!(report.automatic);
    assert!(report.required_steps.is_empty());
}

#[test]
fn test_plan_migration_unsupported() {
    let manifest = minimal_manifest("test-pkg", SchemaVersion::BiorsPackageV1);
    let result = plan_package_schema_migration(&manifest, SchemaVersion::BiorsPackageV0);
    assert!(result.is_none());
}

#[test]
fn test_compare_same_version() {
    let left = minimal_manifest("pkg-a", SchemaVersion::BiorsPackageV0);
    let right = minimal_manifest("pkg-b", SchemaVersion::BiorsPackageV0);
    let report = compare_package_manifest_schemas("left.json", "right.json", &left, &right);
    assert_eq!(report.left_path, "left.json");
    assert_eq!(report.right_path, "right.json");
    assert_eq!(report.left_package, "pkg-a");
    assert_eq!(report.right_package, "pkg-b");
    assert_eq!(report.left_schema_version, "biors.package.v0");
    assert_eq!(report.right_schema_version, "biors.package.v0");
    assert_eq!(report.compatibility, Compatibility::BackwardCompatible);
    assert!(report.schema_compatible);
    assert!(!report.migration_required);
    assert!(!report.same_package_name);
}

#[test]
fn test_compare_migration_required() {
    let left = minimal_manifest("pkg-a", SchemaVersion::BiorsPackageV0);
    let right = minimal_manifest("pkg-b", SchemaVersion::BiorsPackageV1);
    let report = compare_package_manifest_schemas("left.json", "right.json", &left, &right);
    assert_eq!(report.compatibility, Compatibility::MigrationRequired);
    assert!(report.schema_compatible);
    assert!(report.migration_required);
}

#[test]
fn test_compare_same_name() {
    let left = minimal_manifest("same-pkg", SchemaVersion::BiorsPackageV0);
    let right = minimal_manifest("same-pkg", SchemaVersion::BiorsPackageV1);
    let report = compare_package_manifest_schemas("a.json", "b.json", &left, &right);
    assert!(report.same_package_name);
}

#[test]
fn test_convert_backward_compatible() {
    let manifest = minimal_manifest("test-pkg", SchemaVersion::BiorsPackageV0);
    let output = convert_package_manifest(&manifest, SchemaVersion::BiorsPackageV0, None)
        .expect("conversion should succeed");
    assert_eq!(output.manifest.name, "test-pkg");
    assert_eq!(
        output.manifest.schema_version,
        SchemaVersion::BiorsPackageV0
    );
    assert!(!output.report.converted);
    assert!(output.report.automatic);
    assert!(!output.report.metadata_supplied);
}

#[test]
fn test_convert_migration_success() {
    let manifest = minimal_manifest("test-pkg", SchemaVersion::BiorsPackageV0);
    let input = conversion_input();
    let output = convert_package_manifest(&manifest, SchemaVersion::BiorsPackageV1, Some(input))
        .expect("conversion should succeed");
    assert_eq!(
        output.manifest.schema_version,
        SchemaVersion::BiorsPackageV1
    );
    assert!(output.report.converted);
    assert!(!output.report.automatic);
    assert!(output.report.metadata_supplied);
    assert!(output.manifest.package_layout.is_some());
    assert!(output.manifest.metadata.is_some());
}

#[test]
fn test_convert_missing_input() {
    let manifest = minimal_manifest("test-pkg", SchemaVersion::BiorsPackageV0);
    let err = convert_package_manifest(&manifest, SchemaVersion::BiorsPackageV1, None)
        .expect_err("should fail without input");
    assert!(matches!(
        err,
        PackageManifestConversionError::MissingConversionInput { .. }
    ));
    assert_eq!(err.code(), "package.conversion_missing_metadata");
}

#[test]
fn test_convert_unsupported() {
    let manifest = manifest_v1("test-pkg");
    let err = convert_package_manifest(&manifest, SchemaVersion::BiorsPackageV0, None)
        .expect_err("should fail for unsupported conversion");
    assert!(matches!(
        err,
        PackageManifestConversionError::Unsupported { .. }
    ));
    assert_eq!(err.code(), "package.conversion_unsupported");
}

#[test]
fn test_diff_matching() {
    let left = minimal_manifest("pkg-a", SchemaVersion::BiorsPackageV0);
    let right = minimal_manifest("pkg-b", SchemaVersion::BiorsPackageV0);
    let bytes = b"same content";
    let report = diff_package_manifests("a.json", "b.json", &left, &right, bytes, bytes);
    assert_eq!(report.left_path, "a.json");
    assert_eq!(report.right_path, "b.json");
    assert_eq!(report.left_package, "pkg-a");
    assert_eq!(report.right_package, "pkg-b");
    assert_eq!(report.left_schema_version, "biors.package.v0");
    assert_eq!(report.right_schema_version, "biors.package.v0");
    assert_eq!(report.compatibility, Compatibility::BackwardCompatible);
    assert!(!report.same_package_name);
    assert!(report.diff.matches);
    assert!(report.diff.content_diff.is_none());
}

#[test]
fn test_diff_different() {
    let left = minimal_manifest("pkg-a", SchemaVersion::BiorsPackageV0);
    let right = minimal_manifest("pkg-b", SchemaVersion::BiorsPackageV1);
    let left_bytes = b"left content";
    let right_bytes = b"right content";
    let report = diff_package_manifests("a.json", "b.json", &left, &right, left_bytes, right_bytes);
    assert!(!report.diff.matches);
    let diff = report.diff.content_diff.expect("should have content diff");
    assert_eq!(diff.expected_path, "a.json");
    assert_eq!(diff.observed_path, "b.json");
    assert_eq!(diff.expected_len, left_bytes.len());
    assert_eq!(diff.observed_len, right_bytes.len());
    let first = diff.first_difference.expect("should have first difference");
    assert_eq!(first.byte_offset, 0);
    assert_eq!(first.expected_byte, Some(b'l'));
    assert_eq!(first.observed_byte, Some(b'r'));
}
