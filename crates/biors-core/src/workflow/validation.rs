use crate::model_input::ModelInputBuildError;
use crate::sequence::{
    validate_sequence_record, ProteinSequence, ResidueIssue, SequenceRecord,
    SequenceValidationIssue, SequenceValidationReport, ValidatedSequence,
};
use crate::tokenizer::ProteinTokenizerProfile;

pub(super) fn validate_records_for_profile(
    records: &[ProteinSequence],
    profile: ProteinTokenizerProfile,
) -> SequenceValidationReport {
    let kind = profile.sequence_kind();
    let sequences = records
        .iter()
        .map(|record| {
            let validation = validate_sequence_record(&SequenceRecord {
                id: record.id.clone(),
                sequence: String::from_utf8_lossy(&record.sequence).into_owned(),
                kind,
            });
            ValidatedSequence {
                id: validation.id,
                sequence: validation.sequence,
                alphabet: validation.alphabet,
                valid: validation.valid,
                warnings: validation
                    .warnings
                    .into_iter()
                    .map(issue_to_residue_issue)
                    .collect(),
                errors: validation
                    .errors
                    .into_iter()
                    .map(issue_to_residue_issue)
                    .collect(),
            }
        })
        .collect();
    crate::sequence::summarize_validated_sequences(sequences)
}

pub(super) fn validate_workflow_input_hash(input_hash: &str) -> Result<(), ModelInputBuildError> {
    if crate::verification::is_stable_input_hash(input_hash) {
        Ok(())
    } else {
        Err(ModelInputBuildError::InvalidInputHash {
            input_hash: input_hash.to_string(),
        })
    }
}

fn issue_to_residue_issue(issue: SequenceValidationIssue) -> ResidueIssue {
    ResidueIssue {
        residue: issue.symbol,
        position: issue.position,
    }
}
