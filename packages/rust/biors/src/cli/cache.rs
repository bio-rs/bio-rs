use super::CacheCommand;
use crate::errors::CliError;
use crate::output::print_success;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
struct CacheReport {
    action: &'static str,
    root: String,
    exists: bool,
    policy: CachePolicy,
    layout: Vec<CacheLayoutEntry>,
    files: usize,
    total_bytes: u64,
    entries: Vec<CacheEntry>,
    removed_files: usize,
    removed_bytes: u64,
    dry_run: bool,
}

#[derive(Debug, Serialize)]
struct CachePolicy {
    environment_variable: &'static str,
    default_root: &'static str,
    portable_artifact_paths: bool,
    clean_requires_dry_run_or_yes: bool,
}

#[derive(Debug, Serialize)]
struct CacheLayoutEntry {
    name: &'static str,
    purpose: &'static str,
}

#[derive(Debug, Serialize)]
struct CacheEntry {
    path: String,
    bytes: u64,
}

pub(crate) fn run_cache_command(command: CacheCommand) -> Result<(), CliError> {
    match command {
        CacheCommand::Inspect { root } => {
            let root = cache_root(root);
            let entries = collect_cache_entries(&root)?;
            print_success(None, report("inspect", &root, entries, false, 0, 0))
        }
        CacheCommand::Clean { root, dry_run, yes } => {
            if !dry_run && !yes {
                return Err(CliError::Validation {
                    code: "cache.clean_requires_confirmation",
                    message: "cache clean requires --dry-run or --yes".to_string(),
                    location: Some("cache".to_string()),
                });
            }
            let root = cache_root(root);
            validate_clean_root(&root)?;
            let entries = collect_cache_entries(&root)?;
            let removed_files = entries.len();
            let removed_bytes = entries.iter().map(|entry| entry.bytes).sum();
            if yes && !dry_run {
                remove_cache_entries(&entries)?;
                remove_empty_dirs(&root)?;
            }
            print_success(
                None,
                report(
                    "clean",
                    &root,
                    entries,
                    dry_run,
                    removed_files,
                    removed_bytes,
                ),
            )
        }
    }
}

fn cache_root(root: Option<PathBuf>) -> PathBuf {
    root.or_else(|| std::env::var_os("BIORS_ARTIFACT_STORE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(".biors/artifacts"))
}

fn report(
    action: &'static str,
    root: &Path,
    entries: Vec<CacheEntry>,
    dry_run: bool,
    removed_files: usize,
    removed_bytes: u64,
) -> CacheReport {
    let total_bytes = entries.iter().map(|entry| entry.bytes).sum();
    CacheReport {
        action,
        root: root.display().to_string(),
        exists: root.exists(),
        policy: CachePolicy {
            environment_variable: "BIORS_ARTIFACT_STORE",
            default_root: ".biors/artifacts",
            portable_artifact_paths: true,
            clean_requires_dry_run_or_yes: true,
        },
        layout: vec![
            CacheLayoutEntry {
                name: "packages/",
                purpose: "resolved bio-rs package directories or unpacked archives",
            },
            CacheLayoutEntry {
                name: "datasets/",
                purpose: "dataset snapshots keyed by source, version, split, and content hash",
            },
            CacheLayoutEntry {
                name: "locks/",
                purpose: "pipeline.lock and provenance records for reproducible runs",
            },
        ],
        files: entries.len(),
        total_bytes,
        entries,
        removed_files,
        removed_bytes,
        dry_run,
    }
}

fn collect_cache_entries(root: &Path) -> Result<Vec<CacheEntry>, CliError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    collect_files(root, &mut entries)?;
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

fn collect_files(path: &Path, entries: &mut Vec<CacheEntry>) -> Result<(), CliError> {
    for entry in std::fs::read_dir(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| CliError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let entry_path = entry.path();
        let file_type = entry.file_type().map_err(|source| CliError::Read {
            path: entry_path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            collect_files(&entry_path, entries)?;
        } else if file_type.is_file() {
            let bytes = entry
                .metadata()
                .map(|metadata| metadata.len())
                .map_err(|source| CliError::Read {
                    path: entry_path.clone(),
                    source,
                })?;
            entries.push(CacheEntry {
                path: entry_path.display().to_string(),
                bytes,
            });
        }
    }
    Ok(())
}

fn remove_cache_entries(entries: &[CacheEntry]) -> Result<(), CliError> {
    for entry in entries {
        let path = PathBuf::from(&entry.path);
        std::fs::remove_file(&path).map_err(CliError::Write)?;
    }
    Ok(())
}

fn remove_empty_dirs(root: &Path) -> Result<(), CliError> {
    if !root.exists() {
        return Ok(());
    }
    let mut dirs = Vec::new();
    collect_dirs(root, &mut dirs)?;
    dirs.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for dir in dirs {
        let _ = std::fs::remove_dir(&dir);
    }
    Ok(())
}

fn collect_dirs(path: &Path, dirs: &mut Vec<PathBuf>) -> Result<(), CliError> {
    for entry in std::fs::read_dir(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| CliError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let entry_path = entry.path();
        if entry
            .file_type()
            .map_err(|source| CliError::Read {
                path: entry_path.clone(),
                source,
            })?
            .is_dir()
        {
            collect_dirs(&entry_path, dirs)?;
            dirs.push(entry_path);
        }
    }
    Ok(())
}

fn validate_clean_root(root: &Path) -> Result<(), CliError> {
    let components = root.components().count();
    if components < 2 || root == Path::new("/") || root == Path::new(".") {
        return Err(CliError::Validation {
            code: "cache.invalid_root",
            message: "cache root is too broad to clean safely".to_string(),
            location: Some(root.display().to_string()),
        });
    }
    Ok(())
}
