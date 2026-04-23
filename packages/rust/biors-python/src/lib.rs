use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
fn tokenize(input: &str) -> PyResult<String> {
    let result =
        biors_core::tokenize_fasta(input).map_err(|e| PyValueError::new_err(e.to_string()))?;

    serde_json::to_string(&result).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[pymodule]
fn biors(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tokenize, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
