use biors_core::sequence::{SequenceKind, SequenceKindSelection};
use biors_core::tokenizer::{
    protein_tokenizer_config_for_profile, ProteinTokenizerConfig, ProteinTokenizerProfile,
};
use wasm_bindgen::prelude::*;

pub(crate) fn tokenizer_config_for_profile(
    profile: &str,
) -> Result<ProteinTokenizerConfig, JsValue> {
    Ok(protein_tokenizer_config_for_profile(parse_profile(
        profile,
    )?))
}

pub(crate) fn resolve_workflow_profile(
    kind: Option<&str>,
    profile: Option<&str>,
    fasta_bytes: &[u8],
) -> Result<ProteinTokenizerProfile, JsValue> {
    let profile = match profile {
        Some(profile) => parse_profile(profile)?,
        None => default_profile_for_kind(kind, fasta_bytes)?,
    };

    if let Some(kind) = kind.filter(|kind| *kind != "auto") {
        let expected_kind = parse_kind(kind)?;
        if profile.sequence_kind() != expected_kind {
            return Err(JsValue::from_str(&format!(
                "workflow kind '{kind}' does not match tokenizer profile '{}'",
                profile.as_str()
            )));
        }
    }

    Ok(profile)
}

fn parse_profile(profile: &str) -> Result<ProteinTokenizerProfile, JsValue> {
    match profile {
        "protein-20" => Ok(ProteinTokenizerProfile::Protein20),
        "protein-20-special" => Ok(ProteinTokenizerProfile::Protein20Special),
        "dna-iupac" => Ok(ProteinTokenizerProfile::DnaIupac),
        "dna-iupac-special" => Ok(ProteinTokenizerProfile::DnaIupacSpecial),
        "rna-iupac" => Ok(ProteinTokenizerProfile::RnaIupac),
        "rna-iupac-special" => Ok(ProteinTokenizerProfile::RnaIupacSpecial),
        other => Err(JsValue::from_str(&format!(
            "invalid profile: '{other}'. Expected one of protein-20, protein-20-special, dna-iupac, dna-iupac-special, rna-iupac, rna-iupac-special"
        ))),
    }
}

fn default_profile_for_kind(
    kind: Option<&str>,
    fasta_bytes: &[u8],
) -> Result<ProteinTokenizerProfile, JsValue> {
    match kind {
        None | Some("protein") => Ok(ProteinTokenizerProfile::Protein20),
        Some("dna") => Ok(ProteinTokenizerProfile::DnaIupac),
        Some("rna") => Ok(ProteinTokenizerProfile::RnaIupac),
        Some("auto") => auto_detect_profile(fasta_bytes),
        Some(other) => Err(JsValue::from_str(&format!(
            "invalid kind: '{other}'. Expected 'auto', 'protein', 'dna', or 'rna'"
        ))),
    }
}

fn parse_kind(kind: &str) -> Result<SequenceKind, JsValue> {
    match kind {
        "protein" => Ok(SequenceKind::Protein),
        "dna" => Ok(SequenceKind::Dna),
        "rna" => Ok(SequenceKind::Rna),
        other => Err(JsValue::from_str(&format!(
            "invalid kind: '{other}'. Expected 'auto', 'protein', 'dna', or 'rna'"
        ))),
    }
}

fn auto_detect_profile(fasta_bytes: &[u8]) -> Result<ProteinTokenizerProfile, JsValue> {
    let fasta_text = std::str::from_utf8(fasta_bytes)
        .map_err(|e| JsValue::from_str(&format!("invalid UTF-8 FASTA input: {e}")))?;
    let report = biors_core::sequence::kind_validation::validate_fasta_input_with_kind(
        fasta_text,
        SequenceKindSelection::Auto,
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let has_protein = report
        .sequences
        .iter()
        .any(|record| record.kind == SequenceKind::Protein);
    let has_dna = report
        .sequences
        .iter()
        .any(|record| record.kind == SequenceKind::Dna);
    let has_rna = report
        .sequences
        .iter()
        .any(|record| record.kind == SequenceKind::Rna);
    match (has_protein, has_dna, has_rna) {
        (true, false, false) => Ok(ProteinTokenizerProfile::Protein20),
        (false, true, false) => Ok(ProteinTokenizerProfile::DnaIupac),
        (false, false, true) => Ok(ProteinTokenizerProfile::RnaIupac),
        (false, false, false) => Ok(ProteinTokenizerProfile::Protein20),
        _ => Err(JsValue::from_str(
            "unsupported workflow kind: runWorkflow auto-detected mixed sequence kinds; pass an explicit kind and matching profile",
        )),
    }
}
