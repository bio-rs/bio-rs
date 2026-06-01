use super::pipeline_config::load_pipeline_config;
use super::PackageCommand;
use crate::cli::{run_package_convert, run_package_convert_project, run_package_init};
use crate::errors::ErrorLocationValue;
use crate::errors::{classify_validation_code, classify_verification_code, CliError};
use crate::input::{read_fixture_observations, read_package_manifest};
use crate::output::print_success;
use biors_core::package::{
    compare_package_manifest_schemas, diff_package_manifests, inspect_package_manifest,
    plan_package_schema_migration, plan_runtime_bridge,
    validate_package_manifest_artifacts_with_pipeline_config_validator, PackageManifest,
    PackageValidationReport, ReferencedConfigError,
};
use biors_core::verification::verify_package_outputs_with_observation_base;
use serde::Serialize;
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn run_package_command(command: PackageCommand) -> Result<(), CliError> {
    match command {
        PackageCommand::Bridge { path } => run_package_bridge(path),
        PackageCommand::Compatibility { left, right } => run_package_compatibility(left, right),
        PackageCommand::Convert(args) => run_package_convert(*args),
        PackageCommand::ConvertProject(args) => run_package_convert_project(*args),
        PackageCommand::Diff { left, right } => run_package_diff(left, right),
        PackageCommand::Init(args) => run_package_init(*args),
        PackageCommand::Inspect { path } => run_package_inspect(path),
        PackageCommand::Migrate { path, to } => run_package_migrate(path, to.into()),
        PackageCommand::Validate { path } => run_package_validate(path),
        PackageCommand::Verify {
            manifest,
            observations,
        } => run_package_verify(manifest, observations),
    }
}

fn run_package_bridge(path: PathBuf) -> Result<(), CliError> {
    let (manifest, manifest_base_dir) = read_package_manifest(path)?;
    let report = plan_runtime_bridge(&manifest);
    let validation = validate_cli_package_manifest_artifacts(&manifest, &manifest_base_dir);
    if !validation.valid || !report.ready {
        let message = join_failure_messages(
            validation
                .issues
                .iter()
                .chain(report.blocking_issues.iter())
                .map(String::as_str),
        );
        return Err(CliError::ValidationDetails {
            code: "package.bridge_not_ready",
            message,
            location: Some("manifest".to_string()),
            details: json!({
                "validation": validation,
                "bridge": report,
            }),
        });
    }
    print_success(None, report)
}

fn run_package_compatibility(left: PathBuf, right: PathBuf) -> Result<(), CliError> {
    let (left_manifest, _) = read_package_manifest(left.clone())?;
    let (right_manifest, _) = read_package_manifest(right.clone())?;
    let report = compare_package_manifest_schemas(
        &left.display().to_string(),
        &right.display().to_string(),
        &left_manifest,
        &right_manifest,
    );
    print_success(None, report)
}

fn run_package_diff(left: PathBuf, right: PathBuf) -> Result<(), CliError> {
    let left_bytes = read_manifest_bytes(&left)?;
    let right_bytes = read_manifest_bytes(&right)?;
    let (left_manifest, _) = read_package_manifest(left.clone())?;
    let (right_manifest, _) = read_package_manifest(right.clone())?;
    let report = diff_package_manifests(
        &left.display().to_string(),
        &right.display().to_string(),
        &left_manifest,
        &right_manifest,
        &left_bytes,
        &right_bytes,
    );
    print_success(None, report)
}

fn run_package_inspect(path: PathBuf) -> Result<(), CliError> {
    let (manifest, _) = read_package_manifest(path)?;
    let summary = inspect_package_manifest(&manifest);
    print_success(None, summary)
}

fn run_package_migrate(
    path: PathBuf,
    to: biors_core::package::SchemaVersion,
) -> Result<(), CliError> {
    let (manifest, _) = read_package_manifest(path)?;
    let Some(report) = plan_package_schema_migration(&manifest, to) else {
        return Err(CliError::Validation {
            code: "package.migration_unsupported",
            message: format!(
                "no package manifest migration plan from '{}' to '{}'",
                manifest.schema_version, to
            ),
            location: Some("manifest".to_string()),
        });
    };
    print_success(None, report)
}

fn run_package_validate(path: PathBuf) -> Result<(), CliError> {
    let (manifest, manifest_base_dir) = read_package_manifest(path)?;
    let report = validate_cli_package_manifest_artifacts(&manifest, &manifest_base_dir);
    if !report.valid {
        let message = join_failure_messages(report.issues.iter().map(String::as_str));
        return Err(CliError::ValidationDetails {
            code: classify_validation_code(&report),
            message,
            location: Some("manifest".to_string()),
            details: report_details(&report),
        });
    }
    print_success(None, report)
}

fn run_package_verify(manifest: PathBuf, observations: PathBuf) -> Result<(), CliError> {
    let (manifest, manifest_base_dir) = read_package_manifest(manifest)?;
    let (observations, observations_base_dir) = read_fixture_observations(observations)?;
    let validation = validate_cli_package_manifest_artifacts(&manifest, &manifest_base_dir);
    if !validation.valid {
        let message = join_failure_messages(validation.issues.iter().map(String::as_str));
        return Err(CliError::ValidationDetails {
            code: classify_validation_code(&validation),
            message,
            location: Some("manifest".to_string()),
            details: json!({
                "validation": validation,
            }),
        });
    }

    let report = verify_package_outputs_with_observation_base(
        &manifest,
        &observations,
        &manifest_base_dir,
        &observations_base_dir,
    );
    if report.failed > 0 {
        let message = join_failure_messages(
            report
                .results
                .iter()
                .filter_map(|result| result.issue.as_deref()),
        );
        return Err(CliError::ValidationDetails {
            code: classify_verification_code(&report),
            message,
            location: Some("fixtures".to_string()),
            details: report_details(&report),
        });
    }
    print_success(None, report)
}

pub(crate) fn validate_cli_package_manifest_artifacts(
    manifest: &PackageManifest,
    manifest_base_dir: &Path,
) -> PackageValidationReport {
    validate_package_manifest_artifacts_with_pipeline_config_validator(
        manifest,
        manifest_base_dir,
        Some(&|path| {
            load_pipeline_config(path).map(|_| ()).map_err(|error| {
                ReferencedConfigError::new(
                    error.code(),
                    error.to_string(),
                    error.location().map(location_label),
                )
            })
        }),
    )
}

fn location_label(location: ErrorLocationValue) -> String {
    match location {
        ErrorLocationValue::Label(label) => label,
        ErrorLocationValue::Core(location) => format!("{location:?}"),
    }
}

fn read_manifest_bytes(path: &PathBuf) -> Result<Vec<u8>, CliError> {
    fs::read(path).map_err(|source| CliError::Read {
        path: path.clone(),
        source,
    })
}

fn join_failure_messages<'a>(messages: impl Iterator<Item = &'a str>) -> String {
    let joined = messages
        .filter(|message| !message.is_empty())
        .collect::<Vec<_>>()
        .join("; ");
    if joined.is_empty() {
        "package command failed".to_string()
    } else {
        joined
    }
}

fn report_details<T: Serialize>(report: &T) -> serde_json::Value {
    serde_json::to_value(report).unwrap_or_else(|error| {
        json!({
            "serialization_error": error.to_string(),
        })
    })
}
