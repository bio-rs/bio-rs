use biors_core::error::{Diagnostic, ErrorLocation};
use biors_core::model_input::ModelInputBuildError;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

pyo3::create_exception!(biors, BioRsError, PyValueError);

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("BioRsError", m.py().get_type::<BioRsError>())
}

pub(crate) fn py_error(
    code: &'static str,
    message: impl Into<String>,
    location: Option<ErrorLocation>,
) -> PyErr {
    let message = message.into();
    Python::attach(|py| {
        let error = BioRsError::new_err(message.clone());
        let value = error.value(py);
        set_py_error_attrs(py, &value, code, message, location)
            .map_or_else(|error| error, |_| error)
    })
}

pub(crate) fn py_diagnostic_error(error: impl Diagnostic) -> PyErr {
    py_error(error.code(), error.message(), error.location())
}

pub(crate) fn py_json_error(error: serde_json::Error) -> PyErr {
    py_error("json.invalid", format!("invalid JSON: {error}"), None)
}

pub(crate) fn py_serialization_error(error: serde_json::Error) -> PyErr {
    py_error("json.serialization_failed", error.to_string(), None)
}

pub(crate) fn py_model_input_error(error: ModelInputBuildError) -> PyErr {
    let code = match &error {
        ModelInputBuildError::InvalidPolicy { .. } => "model_input.invalid_policy",
        ModelInputBuildError::InvalidInputHash { .. } => "workflow.invalid_input_hash",
        ModelInputBuildError::EmptyTokenizedSequence { .. }
        | ModelInputBuildError::InvalidTokenizedSequence { .. } => "model_input.invalid_sequence",
    };
    py_error(code, error.to_string(), None)
}

fn set_py_error_attrs<'py>(
    py: Python<'py>,
    value: &Bound<'py, PyAny>,
    code: &'static str,
    message: String,
    location: Option<ErrorLocation>,
) -> PyResult<()> {
    value.setattr("code", code)?;
    value.setattr("message", message)?;
    match location {
        Some(location) => value.setattr("location", location_to_dict(py, location)?)?,
        None => value.setattr("location", py.None())?,
    }
    Ok(())
}

fn location_to_dict<'py>(py: Python<'py>, location: ErrorLocation) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("line", location.line)?;
    dict.set_item("record_index", location.record_index)?;
    Ok(dict)
}
