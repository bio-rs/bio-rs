use crate::molecule::MoleculeValidationIssue;
use crate::sequence::{
    SequenceValidationIssue, SequenceValidationIssueCode, ValidatedSequenceRecord,
};
use crate::structure::StructureValidationIssue;

use super::types::{
    ConversionIssue, ConversionIssueCode, ConversionIssueSeverity, ConversionValidation,
};

pub(crate) fn sequence_validation(
    validation: &ValidatedSequenceRecord,
    extra_errors: Vec<ConversionIssue>,
) -> ConversionValidation {
    let warnings = validation
        .warnings
        .iter()
        .map(sequence_warning)
        .collect::<Vec<_>>();
    let mut errors = validation
        .errors
        .iter()
        .map(sequence_error)
        .collect::<Vec<_>>();
    errors.extend(extra_errors);

    let valid = warnings.is_empty() && errors.is_empty();
    ConversionValidation::new(valid, valid, warnings, errors)
}

pub(crate) fn empty_sequence_issue(id: &str) -> ConversionIssue {
    ConversionIssue::new(
        ConversionIssueSeverity::Error,
        ConversionIssueCode::EmptySequence,
        format!("sequence '{id}' is not model-ready: empty sequences cannot be converted"),
    )
}

pub(crate) fn fastq_quality_length_mismatch(
    id: &str,
    sequence_len: usize,
    quality_len: usize,
) -> ConversionIssue {
    ConversionIssue::new(
        ConversionIssueSeverity::Error,
        ConversionIssueCode::FastqQualityLengthMismatch,
        format!(
            "FASTQ record '{id}' has sequence length {sequence_len} but quality length {quality_len}"
        ),
    )
}

pub(crate) fn structure_warning(issue: &StructureValidationIssue) -> ConversionIssue {
    structure_issue(
        issue,
        ConversionIssueSeverity::Warning,
        ConversionIssueCode::StructureValidationWarning,
    )
}

pub(crate) fn structure_error(issue: &StructureValidationIssue) -> ConversionIssue {
    structure_issue(
        issue,
        ConversionIssueSeverity::Error,
        ConversionIssueCode::StructureValidationError,
    )
}

pub(crate) fn molecule_warning(issue: &MoleculeValidationIssue) -> ConversionIssue {
    molecule_issue(
        issue,
        ConversionIssueSeverity::Warning,
        ConversionIssueCode::MoleculeValidationWarning,
    )
}

pub(crate) fn molecule_error(issue: &MoleculeValidationIssue) -> ConversionIssue {
    molecule_issue(
        issue,
        ConversionIssueSeverity::Error,
        ConversionIssueCode::MoleculeValidationError,
    )
}

fn sequence_warning(issue: &SequenceValidationIssue) -> ConversionIssue {
    let code = match issue.code {
        SequenceValidationIssueCode::AmbiguousSymbol => {
            ConversionIssueCode::SequenceAmbiguousSymbol
        }
        SequenceValidationIssueCode::InvalidSymbol => ConversionIssueCode::SequenceInvalidSymbol,
    };
    ConversionIssue::new(
        ConversionIssueSeverity::Warning,
        code,
        issue.message.clone(),
    )
    .with_source_code(issue.code.as_str())
    .with_position(issue.position)
}

fn sequence_error(issue: &SequenceValidationIssue) -> ConversionIssue {
    let code = match issue.code {
        SequenceValidationIssueCode::AmbiguousSymbol => {
            ConversionIssueCode::SequenceAmbiguousSymbol
        }
        SequenceValidationIssueCode::InvalidSymbol => ConversionIssueCode::SequenceInvalidSymbol,
    };
    ConversionIssue::new(ConversionIssueSeverity::Error, code, issue.message.clone())
        .with_source_code(issue.code.as_str())
        .with_position(issue.position)
}

fn structure_issue(
    issue: &StructureValidationIssue,
    severity: ConversionIssueSeverity,
    code: ConversionIssueCode,
) -> ConversionIssue {
    let mut converted = ConversionIssue::new(severity, code, issue.message.clone())
        .with_source_code(issue.code.as_str());
    if let Some(chain_id) = &issue.chain_id {
        converted = converted.with_chain_id(chain_id.clone());
    }
    if let Some(residue_number) = issue.residue_number {
        converted = converted.with_residue_number(residue_number);
    }
    if let Some(atom_serial) = issue.atom_serial {
        converted = converted.with_atom_serial(atom_serial);
    }
    converted
}

fn molecule_issue(
    issue: &MoleculeValidationIssue,
    severity: ConversionIssueSeverity,
    code: ConversionIssueCode,
) -> ConversionIssue {
    let mut converted = ConversionIssue::new(severity, code, issue.message.clone())
        .with_source_code(issue.code.as_str());
    if let Some(atom_index) = issue.atom_index {
        converted = converted.with_atom_index(atom_index);
    }
    converted
}
