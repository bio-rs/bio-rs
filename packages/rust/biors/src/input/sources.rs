use crate::errors::CliError;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedInputDataset {
    pub provided_inputs: usize,
    pub files: Vec<ResolvedInputFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedInputFile {
    pub path: PathBuf,
    pub bytes: u64,
}

pub(crate) fn resolve_fasta_input_dataset(
    inputs: &[PathBuf],
) -> Result<ResolvedInputDataset, CliError> {
    resolve_fasta_input_dataset_with_glob_code(inputs, "dataset.invalid_glob")
}

pub(crate) fn resolve_fasta_input_dataset_with_glob_code(
    inputs: &[PathBuf],
    invalid_glob_code: &'static str,
) -> Result<ResolvedInputDataset, CliError> {
    let mut files = BTreeSet::new();
    for input in inputs {
        if contains_glob_pattern(input) {
            files.extend(expand_glob(input, invalid_glob_code)?);
        } else if input.is_dir() {
            files.extend(expand_directory(input)?);
        } else {
            files.insert(input.clone());
        }
    }

    let files = files
        .into_iter()
        .map(resolved_file)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ResolvedInputDataset {
        provided_inputs: inputs.len(),
        files,
    })
}

fn resolved_file(path: PathBuf) -> Result<ResolvedInputFile, CliError> {
    let metadata = fs::metadata(&path).map_err(|source| CliError::Read {
        path: path.clone(),
        source,
    })?;
    Ok(ResolvedInputFile {
        path,
        bytes: metadata.len(),
    })
}

fn expand_directory(path: &Path) -> Result<Vec<PathBuf>, CliError> {
    let mut files = Vec::new();
    collect_fasta_files(path, &mut files)?;
    files.sort_by_key(|path| path.display().to_string());
    Ok(files)
}

fn collect_fasta_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), CliError> {
    for entry in fs::read_dir(path).map_err(|source| CliError::Read {
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
            collect_fasta_files(&entry_path, files)?;
        } else if file_type.is_file() && is_fasta_path(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn expand_glob(pattern: &Path, invalid_glob_code: &'static str) -> Result<Vec<PathBuf>, CliError> {
    let parent = pattern
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_pattern = pattern
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CliError::Validation {
            code: invalid_glob_code,
            message: format!(
                "glob '{}' does not contain a UTF-8 file pattern",
                pattern.display()
            ),
            location: Some(pattern.display().to_string()),
        })?;

    let mut files = Vec::new();
    for entry in fs::read_dir(parent).map_err(|source| CliError::Read {
        path: parent.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| CliError::Read {
            path: parent.to_path_buf(),
            source,
        })?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let file_name = match file_name.to_str() {
            Some(file_name) => file_name,
            None => continue,
        };
        let file_type = entry.file_type().map_err(|source| CliError::Read {
            path: entry_path.clone(),
            source,
        })?;
        if file_type.is_file() && wildcard_matches(file_pattern, file_name) {
            files.push(entry_path);
        }
    }
    files.sort_by_key(|path| path.display().to_string());
    Ok(files)
}

fn contains_glob_pattern(path: &Path) -> bool {
    let value = path.as_os_str().to_string_lossy();
    value.contains('*') || value.contains('?')
}

fn is_fasta_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("fa" | "fasta" | "faa" | "fna" | "ffn")
    )
}

fn wildcard_matches(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    let mut prev = vec![false; text.len() + 1];
    let mut curr = vec![false; text.len() + 1];

    prev[0] = true;

    for pattern_index in 1..=pattern.len() {
        curr[0] = prev[0] && pattern[pattern_index - 1] == '*';

        for text_index in 1..=text.len() {
            curr[text_index] = match pattern[pattern_index - 1] {
                '*' => prev[text_index] || curr[text_index - 1],
                '?' => prev[text_index - 1],
                literal => literal == text[text_index - 1] && prev[text_index - 1],
            };
        }

        std::mem::swap(&mut prev, &mut curr);
    }

    prev[text.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_matcher_supports_star_and_question_mark() {
        assert!(wildcard_matches("*.fasta", "a.fasta"));
        assert!(wildcard_matches("sample?.fa", "sample1.fa"));
        assert!(!wildcard_matches("*.fasta", "notes.txt"));
    }

    #[test]
    fn fasta_extension_filter_accepts_common_fasta_suffixes() {
        assert!(is_fasta_path(Path::new("protein.FAA")));
        assert!(is_fasta_path(Path::new("reads.fna")));
        assert!(!is_fasta_path(Path::new("notes.txt")));
    }
}
