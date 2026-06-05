use super::PackageConvertArgs;
use crate::cli::package_convert_layout::{build_conversion_input, needs_v1_conversion_input};
use crate::errors::CliError;
use crate::input::read_package_manifest;
use crate::output::print_success;
use biors_core::{
    hash::sha256_bytes_digest,
    package::{convert_package_manifest, PackageManifest, PackageManifestConversionError},
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
    output.report.manifest_sha256 = Some(sha256_bytes_digest(&manifest_bytes));
    if let Some(path) = &args.output {
        fs::write(path, &manifest_bytes).map_err(CliError::Write)?;
        output.report.output_path = Some(path.display().to_string());
    }
    print_success(None, output)
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
