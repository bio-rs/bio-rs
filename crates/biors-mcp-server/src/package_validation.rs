use biors_core::package::{PackageManifest, PackageValidationReport};
use rmcp::{model::ErrorData as McpError, schemars};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PackageValidateParams {
    /// Package manifest JSON string to validate with `base_dir`.
    pub manifest_json: Option<String>,
    /// Package base directory used for manifest-relative artifact validation.
    pub base_dir: Option<String>,
    /// Package manifest path. Artifacts are validated relative to its parent directory.
    pub manifest_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PackageValidateFieldsParams {
    /// Package manifest JSON string to validate without filesystem artifact checks.
    pub manifest_json: String,
}

pub fn validate_fields(
    params: PackageValidateFieldsParams,
) -> Result<PackageValidationReport, McpError> {
    let manifest = parse_manifest(&params.manifest_json)?;
    Ok(biors_core::package::validate_package_manifest(&manifest))
}

pub fn validate(params: PackageValidateParams) -> Result<PackageValidationReport, McpError> {
    let (manifest, base_dir, manifest_path) = load_manifest_and_base_dir(&params)?;
    let validator =
        |path: &Path| biors_core::package::validate_pipeline_config_artifact(&base_dir, path);
    Ok(
        biors_core::package::validate_package_manifest_artifacts_with_manifest_path_and_pipeline_config_validator(
            &manifest,
            &base_dir,
            manifest_path.as_deref(),
            Some(&validator),
        ),
    )
}

fn load_manifest_and_base_dir(
    params: &PackageValidateParams,
) -> Result<(PackageManifest, PathBuf, Option<PathBuf>), McpError> {
    match (&params.manifest_path, &params.manifest_json) {
        (Some(_), Some(_)) => Err(McpError::invalid_params(
            "provide either manifest_path or manifest_json, not both",
            None,
        )),
        (Some(manifest_path), None) => {
            let path = Path::new(manifest_path);
            let manifest_json = std::fs::read_to_string(path)
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
            let manifest = parse_manifest(&manifest_json)?;
            let base_dir = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            Ok((manifest, base_dir, Some(path.to_path_buf())))
        }
        (None, Some(manifest_json)) => {
            let Some(base_dir) = &params.base_dir else {
                return Err(McpError::invalid_params(
                    "package_validate with manifest_json requires base_dir; use package_validate_fields for field-only validation",
                    None,
                ));
            };
            Ok((
                parse_manifest(manifest_json)?,
                Path::new(base_dir).to_path_buf(),
                None,
            ))
        }
        (None, None) => Err(McpError::invalid_params(
            "package_validate requires manifest_path or manifest_json with base_dir",
            None,
        )),
    }
}

fn parse_manifest(manifest_json: &str) -> Result<PackageManifest, McpError> {
    serde_json::from_str(manifest_json).map_err(|e| McpError::invalid_params(e.to_string(), None))
}
