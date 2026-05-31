use crate::conversion::{tokenized_protein_to_py, PyTokenizedProtein};
use biors_core::{sequence, tokenizer};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
pub(crate) fn tokenize_fasta_records(fasta_text: &str) -> PyResult<Vec<PyTokenizedProtein>> {
    let records = tokenizer::tokenize_fasta_records(fasta_text)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(records.into_iter().map(tokenized_protein_to_py).collect())
}

#[pyfunction]
pub(crate) fn tokenize_protein(sequence: &str) -> PyResult<PyTokenizedProtein> {
    let protein = sequence::ProteinSequence {
        id: "user".to_string(),
        sequence: sequence::normalize_sequence(sequence).into_bytes(),
    };
    let record = tokenizer::tokenize_protein(&protein);
    Ok(tokenized_protein_to_py(record))
}
