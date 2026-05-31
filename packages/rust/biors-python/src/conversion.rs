use biors_core::{model_input, sequence, tokenizer};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass(name = "ResidueIssue")]
#[derive(Clone, Debug)]
pub struct PyResidueIssue {
    #[pyo3(get)]
    pub residue: String,
    #[pyo3(get)]
    pub position: usize,
}

#[pyclass(name = "TokenizedProtein")]
#[derive(Clone, Debug)]
pub struct PyTokenizedProtein {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub alphabet: String,
    #[pyo3(get)]
    pub valid: bool,
    #[pyo3(get)]
    pub tokens: Vec<usize>,
    #[pyo3(get)]
    pub length: usize,
    #[pyo3(get)]
    pub warnings: Vec<PyResidueIssue>,
    #[pyo3(get)]
    pub errors: Vec<PyResidueIssue>,
}

#[pyclass(name = "ModelInputRecord")]
#[derive(Clone, Debug)]
pub struct PyModelInputRecord {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub input_ids: Vec<usize>,
    #[pyo3(get)]
    pub attention_mask: Vec<usize>,
    #[pyo3(get)]
    pub truncated: bool,
}

pub(crate) fn parse_padding_policy(padding: &str) -> PyResult<model_input::PaddingPolicy> {
    match padding {
        "fixed_length" => Ok(model_input::PaddingPolicy::FixedLength),
        "no_padding" => Ok(model_input::PaddingPolicy::NoPadding),
        other => Err(PyValueError::new_err(format!(
            "invalid padding: '{other}'. Expected 'fixed_length' or 'no_padding'"
        ))),
    }
}

pub(crate) fn tokenized_protein_to_py(record: tokenizer::TokenizedProtein) -> PyTokenizedProtein {
    PyTokenizedProtein {
        id: record.id,
        alphabet: record.alphabet,
        valid: record.valid,
        tokens: record.tokens.into_iter().map(usize::from).collect(),
        length: record.length,
        warnings: residue_issues_to_py(record.warnings),
        errors: residue_issues_to_py(record.errors),
    }
}

fn residue_issues_to_py(issues: Vec<sequence::ResidueIssue>) -> Vec<PyResidueIssue> {
    issues
        .into_iter()
        .map(|issue| PyResidueIssue {
            residue: issue.residue.to_string(),
            position: issue.position,
        })
        .collect()
}

pub(crate) fn residue_issues_from_py(
    issues: Vec<PyResidueIssue>,
) -> PyResult<Vec<sequence::ResidueIssue>> {
    issues
        .into_iter()
        .map(|issue| {
            let mut chars = issue.residue.chars();
            let residue = chars.next().ok_or_else(|| {
                PyValueError::new_err("residue issue must contain exactly one residue")
            })?;
            if chars.next().is_some() {
                return Err(PyValueError::new_err(
                    "residue issue must contain exactly one residue",
                ));
            }
            Ok(sequence::ResidueIssue {
                residue,
                position: issue.position,
            })
        })
        .collect()
}
