use super::{BatchCommand, KindArg};
use crate::errors::CliError;
use crate::input::open_fasta_input;
use crate::output::print_success;
use biors_core::{
    validate_fasta_reader_summary_with_kind_and_hash, KindAwareSequenceValidationSummary,
    SequenceKindCounts,
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
struct BatchValidationOutput {
    inputs: usize,
    summary: BatchValidationSummary,
    files: Vec<BatchFileValidationReport>,
}

#[derive(Debug, Default, Serialize)]
struct BatchValidationSummary {
    files: usize,
    records: usize,
    valid_records: usize,
    warning_count: usize,
    error_count: usize,
    kind_counts: SequenceKindCounts,
}

#[derive(Debug, Serialize)]
struct BatchFileValidationReport {
    path: String,
    input_hash: String,
    records: usize,
    valid_records: usize,
    warning_count: usize,
    error_count: usize,
    kind_counts: SequenceKindCounts,
}

pub(crate) fn run_batch_command(command: BatchCommand) -> Result<(), CliError> {
    match command {
        BatchCommand::Validate { kind, inputs } => run_batch_validate(kind, inputs),
    }
}

fn run_batch_validate(kind: KindArg, inputs: Vec<PathBuf>) -> Result<(), CliError> {
    let files = expand_batch_inputs(&inputs)?;
    if files.is_empty() {
        return Err(CliError::Validation {
            code: "batch.no_inputs",
            message: "batch validate did not resolve any FASTA input files".to_string(),
            location: None,
        });
    }

    let mut output = BatchValidationOutput {
        inputs: files.len(),
        summary: BatchValidationSummary::default(),
        files: Vec::with_capacity(files.len()),
    };

    for path in files {
        let reader = open_fasta_input(&path)?;
        let validated = validate_fasta_reader_summary_with_kind_and_hash(reader, kind.into())
            .map_err(|error| CliError::from_fasta_read(path.clone(), error))?;
        output.summary.add_file(&validated.summary);
        output.files.push(BatchFileValidationReport::from_summary(
            &path,
            validated.input_hash,
            validated.summary,
        ));
    }

    print_success(None, output)
}

fn expand_batch_inputs(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, CliError> {
    let mut files = BTreeSet::new();
    for input in inputs {
        if contains_glob_pattern(input) {
            files.extend(expand_glob(input)?);
        } else if input.is_dir() {
            files.extend(expand_directory(input)?);
        } else {
            files.insert(input.clone());
        }
    }
    Ok(files.into_iter().collect())
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

fn expand_glob(pattern: &Path) -> Result<Vec<PathBuf>, CliError> {
    let parent = pattern
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_pattern = pattern
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CliError::Validation {
            code: "batch.invalid_glob",
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
    let mut matches = vec![vec![false; text.len() + 1]; pattern.len() + 1];
    matches[0][0] = true;

    for pattern_index in 1..=pattern.len() {
        if pattern[pattern_index - 1] == '*' {
            matches[pattern_index][0] = matches[pattern_index - 1][0];
        }
    }

    for pattern_index in 1..=pattern.len() {
        for text_index in 1..=text.len() {
            matches[pattern_index][text_index] = match pattern[pattern_index - 1] {
                '*' => {
                    matches[pattern_index - 1][text_index] || matches[pattern_index][text_index - 1]
                }
                '?' => matches[pattern_index - 1][text_index - 1],
                literal => {
                    literal == text[text_index - 1] && matches[pattern_index - 1][text_index - 1]
                }
            };
        }
    }

    matches[pattern.len()][text.len()]
}

impl BatchValidationSummary {
    fn add_file(&mut self, summary: &KindAwareSequenceValidationSummary) {
        self.files += 1;
        self.records += summary.records;
        self.valid_records += summary.valid_records;
        self.warning_count += summary.warning_count;
        self.error_count += summary.error_count;
        self.kind_counts.protein += summary.kind_counts.protein;
        self.kind_counts.dna += summary.kind_counts.dna;
        self.kind_counts.rna += summary.kind_counts.rna;
    }
}

impl BatchFileValidationReport {
    fn from_summary(
        path: &Path,
        input_hash: String,
        summary: KindAwareSequenceValidationSummary,
    ) -> Self {
        Self {
            path: path.display().to_string(),
            input_hash,
            records: summary.records,
            valid_records: summary.valid_records,
            warning_count: summary.warning_count,
            error_count: summary.error_count,
            kind_counts: summary.kind_counts,
        }
    }
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
