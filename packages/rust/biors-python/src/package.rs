use crate::errors::{py_json_error, py_serialization_error};
use biors_core::package as core_package;
use pyo3::prelude::*;

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
pub(crate) fn plan_runtime_bridge(manifest_json: &str) -> PyResult<String> {
    let manifest = parse_manifest(manifest_json)?;
    let report = core_package::plan_runtime_bridge(&manifest);
    serde_json::to_string(&report).map_err(py_serialization_error)
}

fn parse_manifest(manifest_json: &str) -> PyResult<core_package::PackageManifest> {
    serde_json::from_str(manifest_json).map_err(py_json_error)
}
