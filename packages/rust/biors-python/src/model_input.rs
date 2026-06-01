use crate::conversion::{
    parse_padding_policy, residue_issues_from_py, PyModelInputRecord, PyTokenizedProtein,
};
use crate::errors::{py_error, py_model_input_error};
use crate::types::PyModelInput;
use biors_core::{model_input, tokenizer};
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (tokenized, max_length, pad_token_id=0, padding="no_padding"))]
pub(crate) fn build_model_inputs_checked(
    tokenized: Vec<PyTokenizedProtein>,
    max_length: usize,
    pad_token_id: u8,
    padding: &str,
) -> PyResult<PyModelInput> {
    let policy = model_input::ModelInputPolicy {
        max_length,
        pad_token_id,
        padding: parse_padding_policy(padding)?,
    };
    let proteins = tokenized
        .into_iter()
        .map(tokenized_protein_from_py)
        .collect::<PyResult<Vec<_>>>()?;
    let model_input =
        model_input::build_model_inputs_checked(&proteins, policy).map_err(py_model_input_error)?;
    let records = model_input
        .records
        .into_iter()
        .map(model_input_record_to_py)
        .collect();
    Ok(PyModelInput { records })
}

pub(crate) fn model_input_record_to_py(
    record: model_input::ModelInputRecord,
) -> PyModelInputRecord {
    PyModelInputRecord {
        id: record.id,
        input_ids: record.input_ids.into_iter().map(usize::from).collect(),
        attention_mask: record.attention_mask.into_iter().map(usize::from).collect(),
        truncated: record.truncated,
    }
}

fn tokenized_protein_from_py(record: PyTokenizedProtein) -> PyResult<tokenizer::TokenizedProtein> {
    let tokens = record
        .tokens
        .into_iter()
        .map(|token| {
            u8::try_from(token).map_err(|_| {
                py_error(
                    "model_input.invalid_sequence",
                    format!("token id {token} is outside the supported 0..=255 range"),
                    None,
                )
            })
        })
        .collect::<PyResult<Vec<_>>>()?;
    Ok(tokenizer::TokenizedProtein {
        id: record.id,
        length: record.length,
        alphabet: record.alphabet,
        valid: record.valid,
        tokens,
        warnings: residue_issues_from_py(record.warnings)?,
        errors: residue_issues_from_py(record.errors)?,
    })
}
