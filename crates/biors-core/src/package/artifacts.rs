use super::{
    artifact_checks::{validate_artifact_reference, validated_artifact_bytes},
    artifact_content::{
        validate_referenced_pipeline_config, validate_tokenizer_config, validate_vocab_config,
        ReferencedConfigValidator,
    },
    validate_declared_layout, validate_package_manifest, PackageManifest, PackageValidationReport,
};
use std::path::Path;

pub fn validate_package_manifest_artifacts(
    manifest: &PackageManifest,
    base_dir: &Path,
) -> PackageValidationReport {
    validate_package_manifest_artifacts_with_manifest_path_and_pipeline_config_validator(
        manifest, base_dir, None, None,
    )
}

pub fn validate_package_manifest_artifacts_with_pipeline_config_validator(
    manifest: &PackageManifest,
    base_dir: &Path,
    pipeline_config_validator: Option<&ReferencedConfigValidator<'_>>,
) -> PackageValidationReport {
    validate_package_manifest_artifacts_with_manifest_path_and_pipeline_config_validator(
        manifest,
        base_dir,
        None,
        pipeline_config_validator,
    )
}

pub fn validate_package_manifest_artifacts_with_manifest_path_and_pipeline_config_validator(
    manifest: &PackageManifest,
    base_dir: &Path,
    manifest_path: Option<&Path>,
    pipeline_config_validator: Option<&ReferencedConfigValidator<'_>>,
) -> PackageValidationReport {
    let mut report = validate_package_manifest(manifest);

    validate_artifact(
        &mut report,
        "model",
        &manifest.model.path,
        manifest.model.checksum.as_deref(),
        base_dir,
    );

    if let Some(tokenizer) = &manifest.tokenizer {
        if let Some(bytes) = validated_artifact_bytes(
            &mut report,
            "tokenizer",
            &tokenizer.path,
            tokenizer.checksum.as_deref(),
            base_dir,
        ) {
            validate_tokenizer_config(&mut report, tokenizer, &bytes);
        }
    }

    if let Some(vocab) = &manifest.vocab {
        if let Some(bytes) = validated_artifact_bytes(
            &mut report,
            "vocab",
            &vocab.path,
            vocab.checksum.as_deref(),
            base_dir,
        ) {
            validate_vocab_config(&mut report, vocab, &bytes);
        }
    }

    validate_pipeline_configs(
        &mut report,
        "preprocessing",
        &manifest.preprocessing,
        base_dir,
        pipeline_config_validator,
    );
    validate_pipeline_configs(
        &mut report,
        "postprocessing",
        &manifest.postprocessing,
        base_dir,
        pipeline_config_validator,
    );

    if let Some(metadata) = &manifest.metadata {
        if let Some(file) = &metadata.license.file {
            validate_artifact(
                &mut report,
                "metadata.license.file",
                &file.path,
                file.checksum.as_deref(),
                base_dir,
            );
        }
        if let Some(file) = &metadata.citation.file {
            validate_artifact(
                &mut report,
                "metadata.citation.file",
                &file.path,
                file.checksum.as_deref(),
                base_dir,
            );
        }
        validate_artifact(
            &mut report,
            "metadata.model_card",
            &metadata.model_card.path,
            metadata.model_card.checksum.as_deref(),
            base_dir,
        );
    }

    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        validate_artifact(
            &mut report,
            &format!("fixtures[{index}].input"),
            &fixture.input,
            fixture.input_hash.as_deref(),
            base_dir,
        );
        validate_artifact(
            &mut report,
            &format!("fixtures[{index}].expected_output"),
            &fixture.expected_output,
            fixture.expected_output_hash.as_deref(),
            base_dir,
        );
    }

    let manifest_relative_path =
        manifest_path.and_then(|path| manifest_path_relative_to_base(base_dir, path));
    validate_declared_layout(&mut report, manifest, manifest_relative_path.as_deref());

    report.finish()
}

fn manifest_path_relative_to_base(base_dir: &Path, manifest_path: &Path) -> Option<String> {
    manifest_path
        .strip_prefix(base_dir)
        .ok()
        .or_else(|| manifest_path.file_name().map(Path::new))
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn validate_pipeline_configs(
    report: &mut PackageValidationReport,
    section: &str,
    steps: &[super::PipelineStep],
    base_dir: &Path,
    pipeline_config_validator: Option<&ReferencedConfigValidator<'_>>,
) {
    for (index, step) in steps.iter().enumerate() {
        if let Some(config) = &step.config {
            let field = format!("{section}[{index}].config");
            let artifact_valid = validate_artifact(
                report,
                &field,
                &config.path,
                config.checksum.as_deref(),
                base_dir,
            );
            if artifact_valid {
                validate_referenced_pipeline_config(
                    report,
                    &field,
                    &config.path,
                    base_dir,
                    pipeline_config_validator,
                );
            }
        }
    }
}

fn validate_artifact(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    checksum: Option<&str>,
    base_dir: &Path,
) -> bool {
    validate_artifact_reference(report, field, path, checksum, base_dir)
}
