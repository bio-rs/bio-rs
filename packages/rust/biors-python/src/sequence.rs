use crate::conversion::validated_sequence_to_py;
use crate::errors::{py_diagnostic_error, py_error};
use crate::types::{PyProteinSequence, PySequenceValidationReport};
use biors_core::fasta;
use biors_core::sequence::{SequenceKind, SequenceKindSelection};
use pyo3::prelude::*;

#[pyfunction]
pub(crate) fn parse_fasta_records(fasta_text: &str) -> PyResult<Vec<PyProteinSequence>> {
    let records = fasta::parse_fasta_records(fasta_text).map_err(py_diagnostic_error)?;
    Ok(records
        .into_iter()
        .map(|r| PyProteinSequence {
            id: r.id,
            sequence: String::from_utf8(r.sequence).unwrap_or_default(),
        })
        .collect())
}

#[pyfunction]
pub(crate) fn validate_fasta_input(fasta_text: &str) -> PyResult<PySequenceValidationReport> {
    let report = fasta::validate_fasta_input(fasta_text).map_err(py_diagnostic_error)?;
    Ok(PySequenceValidationReport {
        records: report.records,
        valid_records: report.valid_records,
        warning_count: report.warning_count,
        error_count: report.error_count,
        sequences: report
            .sequences
            .into_iter()
            .map(validated_sequence_to_py)
            .collect(),
    })
}

#[pyfunction]
pub(crate) fn validate_fasta_input_with_kind(
    fasta_text: &str,
    kind: &str,
) -> PyResult<PySequenceValidationReport> {
    let selection = parse_kind_selection(kind)?;
    let report = biors_core::sequence::validate_fasta_input_with_kind(fasta_text, selection)
        .map_err(py_diagnostic_error)?;
    Ok(PySequenceValidationReport {
        records: report.records,
        valid_records: report.valid_records,
        warning_count: report.warning_count,
        error_count: report.error_count,
        sequences: report
            .sequences
            .into_iter()
            .map(|record| crate::conversion::PyValidatedSequence {
                id: record.id,
                sequence: record.sequence,
                alphabet: record.alphabet,
                valid: record.valid,
                warnings: record
                    .warnings
                    .into_iter()
                    .map(|issue| crate::conversion::PyResidueIssue {
                        residue: issue.symbol.to_string(),
                        position: issue.position,
                    })
                    .collect(),
                errors: record
                    .errors
                    .into_iter()
                    .map(|issue| crate::conversion::PyResidueIssue {
                        residue: issue.symbol.to_string(),
                        position: issue.position,
                    })
                    .collect(),
            })
            .collect(),
    })
}

fn parse_kind_selection(kind: &str) -> PyResult<SequenceKindSelection> {
    match kind {
        "auto" => Ok(SequenceKindSelection::Auto),
        "protein" => Ok(SequenceKindSelection::Explicit(SequenceKind::Protein)),
        "dna" => Ok(SequenceKindSelection::Explicit(SequenceKind::Dna)),
        "rna" => Ok(SequenceKindSelection::Explicit(SequenceKind::Rna)),
        other => Err(py_error(
            "sequence.invalid_kind",
            format!("invalid sequence kind: '{other}'. Expected one of auto, protein, dna, rna"),
            None,
        )),
    }
}
