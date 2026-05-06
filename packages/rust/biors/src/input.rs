use crate::errors::CliError;
use biors_core::{FixtureObservation, PackageManifest};
use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::PathBuf;

pub(crate) fn open_fasta_input(path: &PathBuf) -> Result<Box<dyn BufRead>, CliError> {
    if path.as_os_str() == "-" {
        return Ok(Box::new(BufReader::new(io::stdin())));
    }

    let file = fs::File::open(path).map_err(|source| CliError::Read {
        path: path.clone(),
        source,
    })?;
    Ok(Box::new(BufReader::new(file)))
}

pub(crate) fn read_package_manifest(path: PathBuf) -> Result<(PackageManifest, PathBuf), CliError> {
    let (input, base_dir) = read_input_with_base_dir(path)?;
    Ok((
        serde_json::from_str(&input).map_err(CliError::Json)?,
        base_dir,
    ))
}

pub(crate) fn read_fixture_observations(
    path: PathBuf,
) -> Result<(Vec<FixtureObservation>, PathBuf), CliError> {
    let (input, base_dir) = read_input_with_base_dir(path)?;
    Ok((
        serde_json::from_str(&input).map_err(CliError::Json)?,
        base_dir,
    ))
}

fn read_input(path: PathBuf) -> Result<String, CliError> {
    if path.as_os_str() == "-" {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|source| CliError::Read { path, source })?;
        return Ok(input);
    }

    fs::read_to_string(&path).map_err(|source| CliError::Read { path, source })
}

fn read_input_with_base_dir(path: PathBuf) -> Result<(String, PathBuf), CliError> {
    if path.as_os_str() == "-" {
        return Ok((
            read_input(path)?,
            std::env::current_dir().map_err(CliError::CurrentDir)?,
        ));
    }

    let base_dir = path
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok((read_input(path)?, base_dir))
}
