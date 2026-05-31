use crate::conversion::PyModelInputRecord;
use pyo3::prelude::*;

#[pyclass(name = "ProteinSequence")]
#[derive(Clone, Debug)]
pub struct PyProteinSequence {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub sequence: String,
}

#[pymethods]
impl PyProteinSequence {
    #[new]
    fn new(id: String, sequence: String) -> Self {
        Self { id, sequence }
    }
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
