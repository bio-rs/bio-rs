use super::{
    DataShape, PackageDirectoryLayout, PackageManifest, PackageMetadata,
    PackageValidationIssueCode, PackageValidationReport, SchemaVersion,
};

/// Validate package manifest fields that do not require filesystem access.
pub fn validate_package_manifest(manifest: &PackageManifest) -> PackageValidationReport {
    let mut report = PackageValidationReport::default();

    push_required_issue(&mut report, "name", &manifest.name);
    validate_v1_contract(&mut report, manifest);
    push_required_issue(&mut report, "model.path", &manifest.model.path);
    validate_fixture_list(&mut report, manifest);
    validate_optional_shape(
        &mut report,
        "expected_input",
        manifest.expected_input.as_ref(),
    );
    validate_optional_shape(
        &mut report,
        "expected_output",
        manifest.expected_output.as_ref(),
    );

    report.finish()
}

fn validate_v1_contract(report: &mut PackageValidationReport, manifest: &PackageManifest) {
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

fn validate_fixture_list(report: &mut PackageValidationReport, manifest: &PackageManifest) {
    if manifest.fixtures.is_empty() {
        report.push_issue(
            PackageValidationIssueCode::MissingFixture,
            "fixtures",
            "fixtures must include at least one fixture",
        );
        return;
    }

    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        push_required_issue(report, &format!("fixtures[{index}].name"), &fixture.name);
        push_required_issue(report, &format!("fixtures[{index}].input"), &fixture.input);
        push_required_issue(
            report,
            &format!("fixtures[{index}].expected_output"),
            &fixture.expected_output,
        );
    }
}

fn validate_pipeline_step_configs(
    report: &mut PackageValidationReport,
    section: &str,
    steps: &[super::PipelineStep],
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

fn validate_optional_shape(
    report: &mut PackageValidationReport,
    field: &str,
    shape: Option<&DataShape>,
) {
    if let Some(shape) = shape {
        validate_shape(report, field, shape);
    }
}

fn push_required_issue(report: &mut PackageValidationReport, field: &str, value: &str) {
    if value.trim().is_empty() {
        report.push_issue(
            PackageValidationIssueCode::RequiredField,
            field,
            &format!("{field} is required"),
        );
    }
}

fn push_required_option_issue(
    report: &mut PackageValidationReport,
    field: &str,
    value: Option<&String>,
) {
    match value {
        Some(value) => push_required_issue(report, field, value),
        None => report.push_issue(
            PackageValidationIssueCode::RequiredField,
            field,
            &format!("{field} is required"),
        ),
    }
}

fn push_required_list_issue(report: &mut PackageValidationReport, field: &str, value: &[String]) {
    if value.is_empty() || value.iter().any(|entry| entry.trim().is_empty()) {
        report.push_issue(
            PackageValidationIssueCode::RequiredField,
            field,
            &format!("{field} must include non-empty values"),
        );
    }
}

fn validate_shape(report: &mut PackageValidationReport, field: &str, shape: &DataShape) {
    if shape.shape.is_empty() {
        report.push_issue(
            PackageValidationIssueCode::InvalidShape,
            &format!("{field}.shape"),
            &format!("{field}.shape must include at least one dimension"),
        );
    }
}
