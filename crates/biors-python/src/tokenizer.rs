use crate::conversion::{parse_tokenizer_profile, tokenized_protein_to_py, PyTokenizedProtein};
use crate::errors::py_diagnostic_error;
use biors_core::{fasta, sequence, tokenizer};
use pyo3::prelude::*;
use std::io::Cursor;

#[pyfunction]
#[pyo3(signature = (fasta_text, profile="protein-20"))]
pub(crate) fn tokenize_fasta_records(
    fasta_text: &str,
    profile: &str,
) -> PyResult<Vec<PyTokenizedProtein>> {
    let profile = parse_tokenizer_profile(profile)?;
    let config = tokenizer::protein_tokenizer_config_for_profile(profile);
    let input = fasta::parse_fasta_records_reader(Cursor::new(fasta_text.as_bytes()))
        .map_err(py_diagnostic_error)?;
    Ok(input
        .records
        .iter()
        .map(|record| tokenizer::tokenize_protein_with_config(record, &config))
        .map(tokenized_protein_to_py)
        .collect())
}

#[pyfunction]
#[pyo3(signature = (sequence, id="user", profile="protein-20"))]
pub(crate) fn tokenize_protein(
    sequence: &str,
    id: &str,
    profile: &str,
) -> PyResult<PyTokenizedProtein> {
    let profile = parse_tokenizer_profile(profile)?;
    let config = tokenizer::protein_tokenizer_config_for_profile(profile);
    let protein = sequence::ProteinSequence {
        id: id.to_string(),
        sequence: sequence::normalize_sequence(sequence).into_bytes(),
    };
    let record = tokenizer::tokenize_protein_with_config(&protein, &config);
    Ok(tokenized_protein_to_py(record))
}
