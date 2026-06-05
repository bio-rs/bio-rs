use super::{BatchCommand, KindArg};
use crate::errors::CliError;
use crate::input::{open_fasta_input, resolve_fasta_input_dataset_with_glob_code};
use crate::output::print_success;
use biors_core::sequence::kind_validation::validate_fasta_reader_summary_with_kind_and_hash;
use biors_core::sequence::{KindAwareSequenceValidationSummary, SequenceKindCounts};
use serde::Serialize;
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
    let dataset = resolve_fasta_input_dataset_with_glob_code(&inputs, "batch.invalid_glob")?;
    if dataset.files.is_empty() {
        return Err(CliError::Validation {
            code: "batch.no_inputs",
            message: "batch validate did not resolve any FASTA input files".to_string(),
            location: None,
        });
    }

    let mut output = BatchValidationOutput {
        inputs: dataset.files.len(),
        summary: BatchValidationSummary::default(),
        files: Vec::with_capacity(dataset.files.len()),
    };

    for file in dataset.files {
        let path = file.path;
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
