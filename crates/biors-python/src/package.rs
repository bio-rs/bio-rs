use crate::errors::{py_json_error, py_serialization_error};
use biors_core::package as core_package;
use pyo3::prelude::*;
use std::path::Path;

#[pyfunction]
pub(crate) fn inspect_package_manifest(manifest_json: &str) -> PyResult<String> {
    let manifest = parse_manifest(manifest_json)?;
    let summary = core_package::inspect_package_manifest(&manifest);
    serde_json::to_string(&summary).map_err(py_serialization_error)
}

#[pyfunction]
pub(crate) fn validate_package_manifest(manifest_json: &str) -> PyResult<String> {
    let manifest = parse_manifest(manifest_json)?;
    let report = core_package::validate_package_manifest(&manifest);
    serde_json::to_string(&report).map_err(py_serialization_error)
}

#[pyfunction]
pub(crate) fn validate_package_manifest_artifacts(
    manifest_json: &str,
    base_dir: &str,
) -> PyResult<String> {
    let manifest = parse_manifest(manifest_json)?;
    let base_dir = Path::new(base_dir);
    let report = core_package::validate_package_manifest_artifacts_with_pipeline_config_validator(
        &manifest,
        base_dir,
        Some(&|path| core_package::validate_pipeline_config_artifact(base_dir, path)),
    );
    serde_json::to_string(&report).map_err(py_serialization_error)
}

#[pyfunction]
pub(crate) fn validate_package_manifest_file(manifest_path: &str) -> PyResult<String> {
    let path = Path::new(manifest_path);
    let manifest_json = std::fs::read_to_string(path).map_err(|source| {
        PyErr::new::<pyo3::exceptions::PyOSError, _>(format!(
            "failed to read package manifest '{}': {source}",
            path.display()
        ))
    })?;
    let manifest = parse_manifest(&manifest_json)?;
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let report =
        core_package::validate_package_manifest_artifacts_with_manifest_path_and_pipeline_config_validator(
            &manifest,
            base_dir,
            Some(path),
            Some(&|path| core_package::validate_pipeline_config_artifact(base_dir, path)),
        );
    serde_json::to_string(&report).map_err(py_serialization_error)
}

#[pyfunction]
pub(crate) fn plan_runtime_bridge(manifest_json: &str) -> PyResult<String> {
    let manifest = parse_manifest(manifest_json)?;
    let report = core_package::plan_runtime_bridge(&manifest);
    serde_json::to_string(&report).map_err(py_serialization_error)
}

fn parse_manifest(manifest_json: &str) -> PyResult<core_package::PackageManifest> {
    serde_json::from_str(manifest_json).map_err(py_json_error)
}
