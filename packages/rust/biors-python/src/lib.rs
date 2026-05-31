use biors_core::{fasta, model_input, package, sequence, tokenizer, workflow};
mod conversion;

use conversion::{
    parse_padding_policy, residue_issues_from_py, tokenized_protein_to_py, PyModelInputRecord,
    PyResidueIssue, PyTokenizedProtein,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::io::Cursor;

#[pyclass(name = "ProteinSequence")]
#[derive(Clone, Debug)]
pub struct PyProteinSequence {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub sequence: String,
}

#[pyclass(name = "SequenceValidationReport")]
#[derive(Clone, Debug)]
pub struct PySequenceValidationReport {
    #[pyo3(get)]
    pub records: usize,
    #[pyo3(get)]
    pub valid_records: usize,
    #[pyo3(get)]
    pub warning_count: usize,
    #[pyo3(get)]
    pub error_count: usize,
}

#[pyclass(name = "ModelInput")]
#[derive(Clone, Debug)]
pub struct PyModelInput {
    #[pyo3(get)]
    pub records: Vec<PyModelInputRecord>,
}

#[pyclass(name = "SequenceWorkflowOutput")]
#[derive(Clone, Debug)]
pub struct PySequenceWorkflowOutput {
    #[pyo3(get)]
    pub model_ready: bool,
    #[pyo3(get)]
    pub input_hash: String,
    #[pyo3(get)]
    pub records: Vec<PyModelInputRecord>,
}

#[pyfunction]
fn parse_fasta_records(fasta_text: &str) -> PyResult<Vec<PyProteinSequence>> {
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
fn validate_fasta_input(fasta_text: &str) -> PyResult<PySequenceValidationReport> {
    let report = fasta::validate_fasta_input(fasta_text)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PySequenceValidationReport {
        records: report.records,
        valid_records: report.valid_records,
        warning_count: report.warning_count,
        error_count: report.error_count,
    })
}

#[pyfunction]
fn tokenize_fasta_records(fasta_text: &str) -> PyResult<Vec<PyTokenizedProtein>> {
    let records = tokenizer::tokenize_fasta_records(fasta_text)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(records.into_iter().map(tokenized_protein_to_py).collect())
}

#[pyfunction]
fn tokenize_protein(sequence: &str) -> PyResult<PyTokenizedProtein> {
    let protein = sequence::ProteinSequence {
        id: "user".to_string(),
        sequence: sequence::normalize_sequence(sequence).into_bytes(),
    };
    let record = tokenizer::tokenize_protein(&protein);
    Ok(tokenized_protein_to_py(record))
}

#[pyfunction]
#[pyo3(signature = (tokenized, max_length, pad_token_id=0, padding="no_padding"))]
fn build_model_inputs_checked(
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
    let proteins: Vec<tokenizer::TokenizedProtein> = tokenized
        .into_iter()
        .map(|t| {
            let tokens = t
                .tokens
                .into_iter()
                .map(|token| {
                    u8::try_from(token).map_err(|_| {
                        PyValueError::new_err(format!(
                            "token id {token} is outside the supported 0..=255 range"
                        ))
                    })
                })
                .collect::<PyResult<Vec<_>>>()?;
            Ok(tokenizer::TokenizedProtein {
                id: t.id,
                length: t.length,
                alphabet: t.alphabet,
                valid: t.valid,
                tokens,
                warnings: residue_issues_from_py(t.warnings)?,
                errors: residue_issues_from_py(t.errors)?,
            })
        })
        .collect::<PyResult<Vec<_>>>()?;
    let model_input = model_input::build_model_inputs_checked(&proteins, policy)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let records = model_input
        .records
        .into_iter()
        .map(|r| PyModelInputRecord {
            id: r.id,
            input_ids: r.input_ids.into_iter().map(usize::from).collect(),
            attention_mask: r.attention_mask.into_iter().map(usize::from).collect(),
            truncated: r.truncated,
        })
        .collect();
    Ok(PyModelInput { records })
}

#[pyfunction]
#[pyo3(signature = (input_hash, records, max_length, pad_token_id=0, padding="no_padding"))]
fn prepare_workflow(
    input_hash: String,
    records: Vec<PyProteinSequence>,
    max_length: usize,
    pad_token_id: u8,
    padding: &str,
) -> PyResult<PySequenceWorkflowOutput> {
    let policy = model_input::ModelInputPolicy {
        max_length,
        pad_token_id,
        padding: parse_padding_policy(padding)?,
    };
    let sequences: Vec<sequence::ProteinSequence> = records
        .into_iter()
        .map(|r| sequence::ProteinSequence {
            id: r.id,
            sequence: r.sequence.into_bytes(),
        })
        .collect();
    let output = workflow::prepare_protein_model_input_workflow(input_hash, &sequences, policy)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let output_input_hash = output.provenance.input_hash.clone();
    let model_input_records = if let Some(mi) = output.model_input {
        mi.records
            .into_iter()
            .map(|r| PyModelInputRecord {
                id: r.id,
                input_ids: r.input_ids.into_iter().map(usize::from).collect(),
                attention_mask: r.attention_mask.into_iter().map(usize::from).collect(),
                truncated: r.truncated,
            })
            .collect()
    } else {
        vec![]
    };
    Ok(PySequenceWorkflowOutput {
        model_ready: output.model_ready,
        input_hash: output_input_hash,
        records: model_input_records,
    })
}

#[pyfunction]
#[pyo3(signature = (fasta_text, max_length, pad_token_id=0, padding="no_padding"))]
fn prepare_workflow_from_fasta(
    fasta_text: &str,
    max_length: usize,
    pad_token_id: u8,
    padding: &str,
) -> PyResult<PySequenceWorkflowOutput> {
    let input = fasta::parse_fasta_records_reader(Cursor::new(fasta_text.as_bytes()))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
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

#[pyfunction]
fn inspect_package_manifest(manifest_json: &str) -> PyResult<String> {
    let manifest: package::PackageManifest = serde_json::from_str(manifest_json)
        .map_err(|e| PyValueError::new_err(format!("invalid JSON: {e}")))?;
    let summary = package::inspect_package_manifest(&manifest);
    serde_json::to_string(&summary).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn validate_package_manifest(manifest_json: &str) -> PyResult<String> {
    let manifest: package::PackageManifest = serde_json::from_str(manifest_json)
        .map_err(|e| PyValueError::new_err(format!("invalid JSON: {e}")))?;
    let report = package::validate_package_manifest(&manifest);
    serde_json::to_string(&report).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn plan_runtime_bridge(manifest_json: &str) -> PyResult<String> {
    let manifest: package::PackageManifest = serde_json::from_str(manifest_json)
        .map_err(|e| PyValueError::new_err(format!("invalid JSON: {e}")))?;
    let report = package::plan_runtime_bridge(&manifest);
    serde_json::to_string(&report).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pymodule]
fn biors(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyResidueIssue>()?;
    m.add_class::<PyProteinSequence>()?;
    m.add_class::<PySequenceValidationReport>()?;
    m.add_class::<PyTokenizedProtein>()?;
    m.add_class::<PyModelInput>()?;
    m.add_class::<PyModelInputRecord>()?;
    m.add_class::<PySequenceWorkflowOutput>()?;
    m.add_function(wrap_pyfunction!(parse_fasta_records, m)?)?;
    m.add_function(wrap_pyfunction!(validate_fasta_input, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize_fasta_records, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize_protein, m)?)?;
    m.add_function(wrap_pyfunction!(build_model_inputs_checked, m)?)?;
    m.add_function(wrap_pyfunction!(prepare_workflow, m)?)?;
    m.add_function(wrap_pyfunction!(prepare_workflow_from_fasta, m)?)?;
    m.add_function(wrap_pyfunction!(inspect_package_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(validate_package_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(plan_runtime_bridge, m)?)?;
    Ok(())
}
