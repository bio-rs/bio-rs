use super::{
    validate_package_relative_path, PackageManifest, PackageValidationIssueCode,
    PackageValidationReport,
};

pub(crate) fn validate_declared_layout(
    report: &mut PackageValidationReport,
    manifest: &PackageManifest,
) {
    let Some(layout) = &manifest.package_layout else {
        return;
    };

    validate_layout_field(report, "package_layout.manifest", &layout.manifest);
    validate_layout_field(report, "package_layout.models", &layout.models);
    validate_layout_field(report, "package_layout.fixtures", &layout.fixtures);
    validate_layout_field(report, "package_layout.docs", &layout.docs);
    if let Some(tokenizers) = &layout.tokenizers {
        validate_layout_field(report, "package_layout.tokenizers", tokenizers);
    }
    if let Some(vocabs) = &layout.vocabs {
        validate_layout_field(report, "package_layout.vocabs", vocabs);
    }
    if let Some(observed) = &layout.observed {
        validate_layout_field(report, "package_layout.observed", observed);
    }

    validate_path_under_dir(report, "model.path", &manifest.model.path, &layout.models);
    if let (Some(tokenizer), Some(tokenizers)) = (&manifest.tokenizer, &layout.tokenizers) {
        validate_path_under_dir(report, "tokenizer.path", &tokenizer.path, tokenizers);
    }
    if let (Some(vocab), Some(vocabs)) = (&manifest.vocab, &layout.vocabs) {
        validate_path_under_dir(report, "vocab.path", &vocab.path, vocabs);
    }

    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        validate_path_under_dir(
            report,
            &format!("fixtures[{index}].input"),
            &fixture.input,
            &layout.fixtures,
        );
        validate_path_under_dir(
            report,
            &format!("fixtures[{index}].expected_output"),
            &fixture.expected_output,
            &layout.fixtures,
        );
    }

    if let Some(metadata) = &manifest.metadata {
        if let Some(file) = &metadata.license.file {
            validate_path_under_dir(
                report,
                "metadata.license.file.path",
                &file.path,
                &layout.docs,
            );
        }
        if let Some(file) = &metadata.citation.file {
            validate_path_under_dir(
                report,
                "metadata.citation.file.path",
                &file.path,
                &layout.docs,
            );
        }
        validate_path_under_dir(
            report,
            "metadata.model_card.path",
            &metadata.model_card.path,
            &layout.docs,
        );
    }
}

fn validate_layout_field(report: &mut PackageValidationReport, field: &str, path: &str) {
    if path.trim().is_empty() {
        return;
    }
    if let Err(error) = validate_package_relative_path(path) {
        report.push_issue(
            PackageValidationIssueCode::InvalidAssetPath,
            field,
            &error.to_string(),
        );
    }
}

fn validate_path_under_dir(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    expected_dir: &str,
) {
    if path.trim().is_empty() || expected_dir.trim().is_empty() {
        return;
    }
    if !is_path_under_declared_dir(path, expected_dir) {
        report.push_issue(
            PackageValidationIssueCode::LayoutMismatch,
            field,
            &format!("{field} must be under declared package directory '{expected_dir}'"),
        );
    }
}

fn is_path_under_declared_dir(path: &str, expected_dir: &str) -> bool {
    let expected_dir = expected_dir.trim().trim_end_matches('/');
    let path = path.trim();
    path == expected_dir || path.starts_with(&format!("{expected_dir}/"))
}
