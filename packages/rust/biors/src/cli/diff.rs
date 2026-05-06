use crate::errors::CliError;
use crate::output::print_success;
use biors_core::verification::diff_output_bytes;
use std::fs;
use std::path::PathBuf;

pub(crate) fn run_diff(expected: PathBuf, observed: PathBuf) -> Result<(), CliError> {
    let expected_bytes = fs::read(&expected).map_err(|source| CliError::Read {
        path: expected.clone(),
        source,
    })?;
    let observed_bytes = fs::read(&observed).map_err(|source| CliError::Read {
        path: observed.clone(),
        source,
    })?;
    let report = diff_output_bytes(
        &expected.display().to_string(),
        &observed.display().to_string(),
        &expected_bytes,
        &observed_bytes,
    );
    print_success(None, report)
}
