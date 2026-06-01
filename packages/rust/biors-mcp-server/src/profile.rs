use biors_core::{
    sequence::{KindAwareSequenceValidationReport, SequenceKind, SequenceKindSelection},
    tokenizer::{ProteinTokenizerConfig, ProteinTokenizerProfile},
};
use rmcp::model::ErrorData as McpError;

pub(crate) fn map_kind(kind: &str) -> Result<SequenceKindSelection, McpError> {
    match kind {
        "auto" => Ok(SequenceKindSelection::Auto),
        "protein" => Ok(SequenceKindSelection::Explicit(SequenceKind::Protein)),
        "dna" => Ok(SequenceKindSelection::Explicit(SequenceKind::Dna)),
        "rna" => Ok(SequenceKindSelection::Explicit(SequenceKind::Rna)),
        _ => Err(McpError::invalid_params(
            "invalid kind",
            Some(serde_json::json!({
                "kind": kind,
                "expected": ["auto", "protein", "dna", "rna"]
            })),
        )),
    }
}

pub(crate) fn config_for_profile(profile: &str) -> Result<ProteinTokenizerConfig, McpError> {
    Ok(biors_core::tokenizer::protein_tokenizer_config_for_profile(
        map_profile(profile)?,
    ))
}

pub(crate) fn workflow_profile(
    kind: &str,
    profile: Option<&str>,
    fasta_text: &str,
) -> Result<ProteinTokenizerProfile, McpError> {
    let selection = map_kind(kind)?;
    let report = biors_core::sequence::kind_validation::validate_fasta_input_with_kind(
        fasta_text, selection,
    )
    .map_err(|error| {
        McpError::invalid_params(
            error.to_string(),
            Some(serde_json::json!({
                "code": error.code(),
                "location": error.location(),
            })),
        )
    })?;

    let profile = match profile {
        Some(profile) => map_profile(profile)?,
        None => default_profile(kind, &report)?,
    };

    if kind != "auto" && profile.sequence_kind() != selection.explicit_kind().unwrap() {
        return Err(McpError::invalid_params(
            "workflow kind/profile mismatch",
            Some(serde_json::json!({
                "kind": kind,
                "profile": profile.as_str(),
                "message": "workflow kind must match tokenizer profile sequence kind"
            })),
        ));
    }

    Ok(profile)
}

fn map_profile(profile: &str) -> Result<ProteinTokenizerProfile, McpError> {
    match profile {
        "protein-20" => Ok(ProteinTokenizerProfile::Protein20),
        "protein-20-special" => Ok(ProteinTokenizerProfile::Protein20Special),
        "dna-iupac" => Ok(ProteinTokenizerProfile::DnaIupac),
        "dna-iupac-special" => Ok(ProteinTokenizerProfile::DnaIupacSpecial),
        "rna-iupac" => Ok(ProteinTokenizerProfile::RnaIupac),
        "rna-iupac-special" => Ok(ProteinTokenizerProfile::RnaIupacSpecial),
        _ => Err(McpError::invalid_params(
            "invalid profile",
            Some(serde_json::json!({
                "profile": profile,
                "expected": [
                    "protein-20",
                    "protein-20-special",
                    "dna-iupac",
                    "dna-iupac-special",
                    "rna-iupac",
                    "rna-iupac-special"
                ]
            })),
        )),
    }
}

fn default_profile(
    kind: &str,
    report: &KindAwareSequenceValidationReport,
) -> Result<ProteinTokenizerProfile, McpError> {
    match kind {
        "protein" => Ok(ProteinTokenizerProfile::Protein20),
        "dna" => Ok(ProteinTokenizerProfile::DnaIupac),
        "rna" => Ok(ProteinTokenizerProfile::RnaIupac),
        "auto" => profile_for_auto_detected_report(report),
        _ => unreachable!("map_kind already validated workflow kind"),
    }
}

fn profile_for_auto_detected_report(
    report: &KindAwareSequenceValidationReport,
) -> Result<ProteinTokenizerProfile, McpError> {
    match (
        report.kind_counts.protein > 0,
        report.kind_counts.dna > 0,
        report.kind_counts.rna > 0,
    ) {
        (true, false, false) => Ok(ProteinTokenizerProfile::Protein20),
        (false, true, false) => Ok(ProteinTokenizerProfile::DnaIupac),
        (false, false, true) => Ok(ProteinTokenizerProfile::RnaIupac),
        (false, false, false) => Ok(ProteinTokenizerProfile::Protein20),
        _ => Err(McpError::invalid_params(
            "mixed auto-detected workflow kinds",
            Some(serde_json::json!({
                "detected_kind_counts": report.kind_counts,
                "message": "MCP workflow auto mode requires one detected sequence kind; pass an explicit kind and matching profile for mixed inputs"
            })),
        )),
    }
}
