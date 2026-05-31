use crate::types::{PyProteinSequence, PySequenceValidationReport};
use biors_core::fasta;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
pub(crate) fn parse_fasta_records(fasta_text: &str) -> PyResult<Vec<PyProteinSequence>> {
    let records =
        fasta::parse_fasta_records(fasta_text).map_err(|e| PyValueError::new_err(e.to_string()))?;
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
    let report = fasta::validate_fasta_input(fasta_text)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PySequenceValidationReport {
        records: report.records,
        valid_records: report.valid_records,
        warning_count: report.warning_count,
        error_count: report.error_count,
    })
}
