use super::PackageCommand;
use crate::cli::{run_package_convert, run_package_convert_project, run_package_init};
use crate::errors::{classify_validation_code, classify_verification_code, CliError};
use crate::input::{read_fixture_observations, read_package_manifest};
use crate::output::print_success;
use biors_core::package::{
    compare_package_manifest_schemas, diff_package_manifests, inspect_package_manifest,
    plan_package_schema_migration, plan_runtime_bridge, validate_package_manifest_artifacts,
};
use biors_core::verification::verify_package_outputs_with_observation_base;
use std::{fs, path::PathBuf};

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
    let validation = validate_package_manifest_artifacts(&manifest, &manifest_base_dir);
    if !validation.valid || !report.ready {
        return Err(CliError::Validation {
            code: "package.bridge_not_ready",
            message: format!(
                "{:?}",
                validation
                    .issues
                    .iter()
                    .chain(report.blocking_issues.iter())
                    .collect::<Vec<_>>()
            ),
            location: Some("manifest".to_string()),
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
    let report = validate_package_manifest_artifacts(&manifest, &manifest_base_dir);
    if !report.valid {
        return Err(CliError::Validation {
            code: classify_validation_code(&report),
            message: format!("{:?}", report.issues),
            location: Some("manifest".to_string()),
        });
    }
    print_success(None, report)
}

fn run_package_verify(manifest: PathBuf, observations: PathBuf) -> Result<(), CliError> {
    let (manifest, manifest_base_dir) = read_package_manifest(manifest)?;
    let (observations, observations_base_dir) = read_fixture_observations(observations)?;
    let report = verify_package_outputs_with_observation_base(
        &manifest,
        &observations,
        &manifest_base_dir,
        &observations_base_dir,
    );
    if report.failed > 0 {
        return Err(CliError::Validation {
            code: classify_verification_code(&report),
            message: format!(
                "{:?}",
                report
                    .results
                    .iter()
                    .filter_map(|result| result.issue.as_ref())
                    .collect::<Vec<_>>()
            ),
            location: Some("fixtures".to_string()),
        });
    }
    print_success(None, report)
}

fn read_manifest_bytes(path: &PathBuf) -> Result<Vec<u8>, CliError> {
    fs::read(path).map_err(|source| CliError::Read {
        path: path.clone(),
        source,
    })
}
