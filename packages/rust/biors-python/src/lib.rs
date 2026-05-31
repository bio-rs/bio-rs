mod conversion;
mod model_input;
mod package;
mod sequence;
mod tokenizer;
mod types;
mod workflow;

use conversion::{PyModelInputRecord, PyResidueIssue, PyTokenizedProtein};
use pyo3::prelude::*;
use types::{
    PyModelInput, PyProteinSequence, PySequenceValidationReport, PySequenceWorkflowOutput,
};

#[pymodule]
fn biors(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyResidueIssue>()?;
    m.add_class::<PyProteinSequence>()?;
    m.add_class::<PySequenceValidationReport>()?;
    m.add_class::<PyTokenizedProtein>()?;
    m.add_class::<PyModelInput>()?;
    m.add_class::<PyModelInputRecord>()?;
    m.add_class::<PySequenceWorkflowOutput>()?;
    m.add_function(wrap_pyfunction!(sequence::parse_fasta_records, m)?)?;
    m.add_function(wrap_pyfunction!(sequence::validate_fasta_input, m)?)?;
    m.add_function(wrap_pyfunction!(tokenizer::tokenize_fasta_records, m)?)?;
    m.add_function(wrap_pyfunction!(tokenizer::tokenize_protein, m)?)?;
    m.add_function(wrap_pyfunction!(
        model_input::build_model_inputs_checked,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(workflow::prepare_workflow, m)?)?;
    m.add_function(wrap_pyfunction!(workflow::prepare_workflow_from_fasta, m)?)?;
    m.add_function(wrap_pyfunction!(package::inspect_package_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(package::validate_package_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(package::plan_runtime_bridge, m)?)?;
    Ok(())
}
