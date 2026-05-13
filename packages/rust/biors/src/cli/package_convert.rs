use super::PackageConvertArgs;
use crate::errors::CliError;
use crate::input::read_package_manifest;
use crate::output::print_success;
use biors_core::{
    hash::sha256_digest,
    package::{
        convert_package_manifest, CitationMetadata, DocumentArtifact, LicenseMetadata,
        ModelCardMetadata, PackageDirectoryLayout, PackageManifest, PackageManifestConversionError,
        PackageManifestConversionInput, PackageMetadata, PipelineStep, SchemaVersion,
    },
};
use std::fs;

pub(crate) fn run_package_convert(args: PackageConvertArgs) -> Result<(), CliError> {
    let (manifest, _) = read_package_manifest(args.path.clone())?;
    let to = args.to.into();
    let conversion_input = if needs_v1_conversion_input(manifest.schema_version, to) {
        Some(build_conversion_input(&args, &manifest)?)
    } else {
        None
    };
    let mut output =
        convert_package_manifest(&manifest, to, conversion_input).map_err(conversion_error)?;
    let manifest_bytes = converted_manifest_bytes(&output.manifest)?;
    output.report.manifest_sha256 = Some(sha256_digest(&manifest_bytes));
    if let Some(path) = &args.output {
        fs::write(path, &manifest_bytes).map_err(CliError::Write)?;
        output.report.output_path = Some(path.display().to_string());
    }
    print_success(None, output)
}

fn needs_v1_conversion_input(from: SchemaVersion, to: SchemaVersion) -> bool {
    matches!(
        (from, to),
        (SchemaVersion::BiorsPackageV0, SchemaVersion::BiorsPackageV1)
    )
}

fn build_conversion_input(
    args: &PackageConvertArgs,
    manifest: &PackageManifest,
) -> Result<PackageManifestConversionInput, CliError> {
    let metadata = build_metadata(args)?;
    let package_layout = build_layout(args, manifest, &metadata)?;
    Ok(PackageManifestConversionInput {
        package_layout,
        metadata,
    })
}

fn build_metadata(args: &PackageConvertArgs) -> Result<PackageMetadata, CliError> {
    Ok(PackageMetadata {
        license: LicenseMetadata {
            expression: required_option(args.license.as_deref(), "--license")?,
            file: optional_document(args.license_file.as_deref()),
        },
        citation: CitationMetadata {
            preferred_citation: required_option(args.citation.as_deref(), "--citation")?,
            doi: args.doi.clone(),
            file: optional_document(args.citation_file.as_deref()),
        },
        model_card: ModelCardMetadata {
            path: required_option(args.model_card.as_deref(), "--model-card")?,
            checksum: None,
            summary: required_option(args.model_card_summary.as_deref(), "--model-card-summary")?,
            intended_use: required_list(&args.intended_use, "--intended-use")?,
            limitations: required_list(&args.limitations, "--limitation")?,
        },
    })
}

fn build_layout(
    args: &PackageConvertArgs,
    manifest: &PackageManifest,
    metadata: &PackageMetadata,
) -> Result<PackageDirectoryLayout, CliError> {
    let fixture_paths: Vec<&str> = manifest
        .fixtures
        .iter()
        .flat_map(|fixture| [fixture.input.as_str(), fixture.expected_output.as_str()])
        .collect();
    let pipeline_paths = pipeline_config_paths(&manifest.preprocessing, &manifest.postprocessing);
    let docs_paths = document_paths(metadata);

    Ok(PackageDirectoryLayout {
        manifest: manifest_file_name(args),
        models: infer_required_dir(
            args.models_dir.as_deref(),
            &[manifest.model.path.as_str()],
            "--models-dir",
        )?,
        tokenizers: infer_optional_dir(
            args.tokenizers_dir.as_deref(),
            manifest.tokenizer.as_ref().map(|asset| asset.path.as_str()),
            "--tokenizers-dir",
        )?,
        vocabs: infer_optional_dir(
            args.vocabs_dir.as_deref(),
            manifest.vocab.as_ref().map(|asset| asset.path.as_str()),
            "--vocabs-dir",
        )?,
        pipelines: infer_optional_many_dir(
            args.pipelines_dir.as_deref(),
            &pipeline_paths,
            "--pipelines-dir",
        )?,
        fixtures: infer_required_dir(
            args.fixtures_dir.as_deref(),
            &fixture_paths,
            "--fixtures-dir",
        )?,
        observed: clean_optional_dir(args.observed_dir.as_deref(), "--observed-dir")?,
        docs: infer_required_dir(args.docs_dir.as_deref(), &docs_paths, "--docs-dir")?,
    })
}

fn pipeline_config_paths<'a>(
    preprocessing: &'a [PipelineStep],
    postprocessing: &'a [PipelineStep],
) -> Vec<&'a str> {
    preprocessing
        .iter()
        .chain(postprocessing.iter())
        .filter_map(|step| step.config.as_ref().map(|config| config.path.as_str()))
        .collect()
}

fn document_paths(metadata: &PackageMetadata) -> Vec<&str> {
    let mut paths = vec![metadata.model_card.path.as_str()];
    if let Some(file) = &metadata.license.file {
        paths.push(file.path.as_str());
    }
    if let Some(file) = &metadata.citation.file {
        paths.push(file.path.as_str());
    }
    paths
}

fn infer_required_dir(
    override_dir: Option<&str>,
    paths: &[&str],
    option_name: &str,
) -> Result<String, CliError> {
    let dir = match override_dir {
        Some(dir) => clean_dir(dir, option_name)?,
        None => common_parent_dir(paths).ok_or_else(|| {
            conversion_error_message(
                "package.conversion_layout_conflict",
                format!("could not infer {option_name}; pass {option_name} explicitly"),
            )
        })?,
    };
    ensure_paths_under_dir(&dir, paths, option_name)?;
    Ok(dir)
}

fn infer_optional_dir(
    override_dir: Option<&str>,
    path: Option<&str>,
    option_name: &str,
) -> Result<Option<String>, CliError> {
    match (override_dir, path) {
        (Some(dir), Some(path)) => infer_required_dir(Some(dir), &[path], option_name).map(Some),
        (Some(dir), None) => clean_optional_dir(Some(dir), option_name),
        (None, Some(path)) => infer_required_dir(None, &[path], option_name).map(Some),
        (None, None) => Ok(None),
    }
}

fn infer_optional_many_dir(
    override_dir: Option<&str>,
    paths: &[&str],
    option_name: &str,
) -> Result<Option<String>, CliError> {
    if paths.is_empty() {
        return clean_optional_dir(override_dir, option_name);
    }
    infer_required_dir(override_dir, paths, option_name).map(Some)
}

fn clean_optional_dir(dir: Option<&str>, option_name: &str) -> Result<Option<String>, CliError> {
    dir.map(|value| clean_dir(value, option_name)).transpose()
}

fn clean_dir(value: &str, option_name: &str) -> Result<String, CliError> {
    let value = value.trim().trim_end_matches('/').to_string();
    if value.is_empty() || value.starts_with('/') || value.split('/').any(|part| part == "..") {
        return Err(conversion_error_message(
            "package.conversion_layout_conflict",
            format!("{option_name} must be a non-empty package-relative path"),
        ));
    }
    Ok(value)
}

fn common_parent_dir(paths: &[&str]) -> Option<String> {
    let mut parents = paths.iter().filter_map(|path| parent_dir(path));
    let first = parents.next()?;
    parents.all(|parent| parent == first).then_some(first)
}

fn parent_dir(path: &str) -> Option<String> {
    let (parent, _) = path.rsplit_once('/')?;
    let parent = parent.trim().trim_end_matches('/');
    (!parent.is_empty()).then(|| parent.to_string())
}

fn ensure_paths_under_dir(dir: &str, paths: &[&str], option_name: &str) -> Result<(), CliError> {
    if paths.iter().all(|path| path_is_under_dir(path, dir)) {
        return Ok(());
    }
    Err(conversion_error_message(
        "package.conversion_layout_conflict",
        format!("all paths for {option_name} must live under '{dir}'"),
    ))
}

fn path_is_under_dir(path: &str, dir: &str) -> bool {
    path == dir || path.starts_with(&format!("{dir}/"))
}

fn manifest_file_name(args: &PackageConvertArgs) -> String {
    args.output
        .as_ref()
        .or(Some(&args.path))
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .filter(|name| *name != "-")
        .unwrap_or("manifest.json")
        .to_string()
}

fn required_option(value: Option<&str>, option_name: &str) -> Result<String, CliError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(conversion_error_message(
            "package.conversion_missing_metadata",
            format!("{option_name} is required when converting to biors.package.v1"),
        ));
    };
    Ok(value.to_string())
}

fn required_list(values: &[String], option_name: &str) -> Result<Vec<String>, CliError> {
    let cleaned: Vec<String> = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect();
    if cleaned.is_empty() {
        return Err(conversion_error_message(
            "package.conversion_missing_metadata",
            format!("{option_name} is required when converting to biors.package.v1"),
        ));
    }
    Ok(cleaned)
}

fn optional_document(path: Option<&str>) -> Option<DocumentArtifact> {
    path.map(str::trim)
        .filter(|path| !path.is_empty())
        .map(|path| DocumentArtifact {
            path: path.to_string(),
            checksum: None,
        })
}

fn converted_manifest_bytes(manifest: &PackageManifest) -> Result<Vec<u8>, CliError> {
    let mut bytes = serde_json::to_vec_pretty(manifest).map_err(CliError::Serialization)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn conversion_error(error: PackageManifestConversionError) -> CliError {
    match error {
        PackageManifestConversionError::MissingConversionInput { from, to } => {
            conversion_error_message(
                "package.conversion_missing_metadata",
                format!("conversion from '{from}' to '{to}' requires v1 metadata and layout input"),
            )
        }
        PackageManifestConversionError::Unsupported { from, to } => conversion_error_message(
            "package.conversion_unsupported",
            format!("no package manifest conversion from '{from}' to '{to}'"),
        ),
    }
}

fn conversion_error_message(code: &'static str, message: String) -> CliError {
    CliError::Validation {
        code,
        message,
        location: Some("manifest".to_string()),
    }
}
