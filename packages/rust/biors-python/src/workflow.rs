use crate::errors::{py_diagnostic_error, py_model_input_error, py_serialization_error};
use crate::model_input::model_input_record_to_py;
use crate::types::{PyProteinSequence, PySequenceWorkflowOutput};
use biors_core::{fasta, model_input, sequence, workflow};
use pyo3::prelude::*;
use std::io::Cursor;

#[pyfunction]
#[pyo3(signature = (input_hash, records, max_length, pad_token_id=0, padding="no_padding"))]
pub(crate) fn prepare_workflow(
    input_hash: String,
    records: Vec<PyProteinSequence>,
    max_length: usize,
    pad_token_id: u8,
    padding: &str,
) -> PyResult<PySequenceWorkflowOutput> {
    let policy = model_input::ModelInputPolicy {
        max_length,
        pad_token_id,
        padding: crate::conversion::parse_padding_policy(padding)?,
    };
    let sequences = records
        .into_iter()
        .map(|r| sequence::ProteinSequence {
            id: r.id,
            sequence: r.sequence.into_bytes(),
        })
        .collect::<Vec<_>>();
    let output = workflow::prepare_protein_model_input_workflow(input_hash, &sequences, policy)
        .map_err(py_model_input_error)?;
    let output_input_hash = output.provenance.input_hash.clone();
    let report_json = serde_json::to_string(&output).map_err(py_serialization_error)?;
    let records = output
        .model_input
        .map(|model_input| {
            model_input
                .records
                .into_iter()
                .map(model_input_record_to_py)
                .collect()
        })
        .unwrap_or_default();
    Ok(PySequenceWorkflowOutput {
        model_ready: output.model_ready,
        input_hash: output_input_hash,
        records,
        report_json,
    })
}

#[pyfunction]
#[pyo3(signature = (fasta_text, max_length, pad_token_id=0, padding="no_padding"))]
pub(crate) fn prepare_workflow_from_fasta(
    fasta_text: &str,
    max_length: usize,
    pad_token_id: u8,
    padding: &str,
) -> PyResult<PySequenceWorkflowOutput> {
    let input = fasta::parse_fasta_records_reader(Cursor::new(fasta_text.as_bytes()))
        .map_err(py_diagnostic_error)?;
    prepare_workflow(
        input.input_hash,
        input
            .records
            .into_iter()
            .map(|record| PyProteinSequence {
                id: record.id,
                sequence: String::from_utf8(record.sequence).unwrap_or_default(),
            })
            .collect(),
        max_length,
        pad_token_id,
        padding,
    )
}
