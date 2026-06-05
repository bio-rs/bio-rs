use super::super::{
    PackageDirectoryLayout, PackageManifest, PackageMetadata, PackageValidationIssueCode,
    PackageValidationReport, PipelineStep, RuntimeBackend, SchemaVersion, TokenAsset,
};
use super::required::{push_required_issue, push_required_list_issue, push_required_option_issue};

pub(super) fn validate_contract_identifiers(
    report: &mut PackageValidationReport,
    manifest: &PackageManifest,
) {
    if let Some(tokenizer) = &manifest.tokenizer {
        validate_token_asset_identifiers(report, "tokenizer", tokenizer);
    }
    if let Some(vocab) = &manifest.vocab {
        validate_token_asset_identifiers(report, "vocab", vocab);
    }
    validate_pipeline_step_identifiers(report, "preprocessing", &manifest.preprocessing);
    validate_pipeline_step_identifiers(report, "postprocessing", &manifest.postprocessing);
}

pub(super) fn validate_runtime_contract(
    report: &mut PackageValidationReport,
    manifest: &PackageManifest,
) {
    if manifest.runtime.backend == RuntimeBackend::ExternalProcess {
        report.push_issue(
            PackageValidationIssueCode::UnsupportedRuntimeBackend,
            "runtime.backend",
            "runtime.backend 'external-process' is experimental and is not supported by the public package manifest contract",
        );
    }
}

pub(super) fn validate_v1_contract(
    report: &mut PackageValidationReport,
    manifest: &PackageManifest,
) {
    if manifest.schema_version != SchemaVersion::BiorsPackageV1 {
        if let Some(layout) = &manifest.package_layout {
            validate_package_layout_fields(report, layout, manifest);
        }
        if let Some(metadata) = &manifest.metadata {
            validate_metadata_fields(report, metadata);
        }
        return;
    }

    match &manifest.package_layout {
        Some(layout) => validate_package_layout_fields(report, layout, manifest),
        None => report.push_issue(
            PackageValidationIssueCode::RequiredField,
            "package_layout",
            "package_layout is required for biors.package.v1",
        ),
    }

    match &manifest.metadata {
        Some(metadata) => validate_metadata_fields(report, metadata),
        None => report.push_issue(
            PackageValidationIssueCode::RequiredField,
            "metadata",
            "metadata is required for biors.package.v1",
        ),
    }
}

fn validate_token_asset_identifiers(
    report: &mut PackageValidationReport,
    field: &str,
    asset: &TokenAsset,
) {
    push_required_issue(report, &format!("{field}.name"), &asset.name);
    if let Some(contract_version) = &asset.contract_version {
        push_required_issue(
            report,
            &format!("{field}.contract_version"),
            contract_version,
        );
    }
}

fn validate_pipeline_step_identifiers(
    report: &mut PackageValidationReport,
    section: &str,
    steps: &[PipelineStep],
) {
    for (index, step) in steps.iter().enumerate() {
        push_required_issue(report, &format!("{section}[{index}].name"), &step.name);
        push_required_issue(
            report,
            &format!("{section}[{index}].implementation"),
            &step.implementation,
        );
        push_required_issue(
            report,
            &format!("{section}[{index}].contract"),
            &step.contract,
        );
        if let Some(contract_version) = &step.contract_version {
            push_required_issue(
                report,
                &format!("{section}[{index}].contract_version"),
                contract_version,
            );
        }
    }
}

fn validate_package_layout_fields(
    report: &mut PackageValidationReport,
    layout: &PackageDirectoryLayout,
    manifest: &PackageManifest,
) {
    push_required_issue(report, "package_layout.manifest", &layout.manifest);
    push_required_issue(report, "package_layout.models", &layout.models);
    push_required_issue(report, "package_layout.fixtures", &layout.fixtures);
    push_required_issue(report, "package_layout.docs", &layout.docs);

    if manifest.tokenizer.is_some() {
        push_required_option_issue(
            report,
            "package_layout.tokenizers",
            layout.tokenizers.as_ref(),
        );
    }
    if manifest.vocab.is_some() {
        push_required_option_issue(report, "package_layout.vocabs", layout.vocabs.as_ref());
    }
    if pipeline_steps_have_config(manifest) {
        push_required_option_issue(
            report,
            "package_layout.pipelines",
            layout.pipelines.as_ref(),
        );
    }
    validate_pipeline_step_configs(report, "preprocessing", &manifest.preprocessing);
    validate_pipeline_step_configs(report, "postprocessing", &manifest.postprocessing);
}

fn validate_metadata_fields(report: &mut PackageValidationReport, metadata: &PackageMetadata) {
    push_required_issue(
        report,
        "metadata.license.expression",
        &metadata.license.expression,
    );
    push_required_issue(
        report,
        "metadata.citation.preferred_citation",
        &metadata.citation.preferred_citation,
    );
    push_required_issue(
        report,
        "metadata.model_card.path",
        &metadata.model_card.path,
    );
    push_required_issue(
        report,
        "metadata.model_card.summary",
        &metadata.model_card.summary,
    );
    push_required_list_issue(
        report,
        "metadata.model_card.intended_use",
        &metadata.model_card.intended_use,
    );
    push_required_list_issue(
        report,
        "metadata.model_card.limitations",
        &metadata.model_card.limitations,
    );
}

fn validate_pipeline_step_configs(
    report: &mut PackageValidationReport,
    section: &str,
    steps: &[PipelineStep],
) {
    for (index, step) in steps.iter().enumerate() {
        if let Some(config) = &step.config {
            push_required_issue(
                report,
                &format!("{section}[{index}].config.path"),
                &config.path,
            );
        }
    }
}

fn pipeline_steps_have_config(manifest: &PackageManifest) -> bool {
    manifest
        .preprocessing
        .iter()
        .chain(manifest.postprocessing.iter())
        .any(|step| step.config.is_some())
}
