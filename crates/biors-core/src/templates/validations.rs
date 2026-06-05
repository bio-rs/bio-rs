use super::TemplateValidation;

pub(super) static PROTEIN_VALIDATIONS: [TemplateValidation; 4] = [
    validation("sequence_kind_protein", "record must validate as protein"),
    validation(
        "normalized_sequence",
        "sequence must be uppercase normalized residues",
    ),
    validation(
        "no_invalid_residues",
        "invalid residues must be rejected before model input",
    ),
    validation(
        "tokenizer_profile",
        "tokenizer profile must be recorded with token IDs",
    ),
];
pub(super) static VARIANT_VALIDATIONS: [TemplateValidation; 5] = [
    validation(
        "reference_sequence_valid",
        "reference sequence must validate as protein",
    ),
    validation(
        "variant_position_in_range",
        "variant position must be within the reference",
    ),
    validation(
        "reference_residue_matches",
        "reference residue must match the sequence",
    ),
    validation(
        "alternate_residue_valid",
        "alternate residue must be a valid protein residue",
    ),
    validation(
        "variant_id_stable",
        "variant identifier must be stable within the batch",
    ),
];
pub(super) static MOLECULE_VALIDATIONS: [TemplateValidation; 4] = [
    validation(
        "molecule_parse",
        "molecule source must parse into an atom and bond graph",
    ),
    validation(
        "valence_check",
        "valence errors must be rejected or reported",
    ),
    validation(
        "derived_features",
        "canonical graph key, formula, mass, and fingerprint are required",
    ),
    validation(
        "record_identity",
        "record identity must be stable for output joins",
    ),
];
pub(super) static STRUCTURE_VALIDATIONS: [TemplateValidation; 5] = [
    validation(
        "pdb_parse",
        "PDB ATOM/HETATM records must parse deterministically",
    ),
    validation(
        "coordinate_finiteness",
        "atom coordinates must be finite numbers",
    ),
    validation(
        "chain_sequence_extract",
        "coordinate-derived chain sequences must be extracted",
    ),
    validation(
        "missing_residue_capture",
        "REMARK 465 missing residues must be preserved",
    ),
    validation(
        "seqres_mapping",
        "SEQRES mapping status must be explicit when present",
    ),
];
pub(super) static SEARCH_VALIDATIONS: [TemplateValidation; 4] = [
    validation(
        "kind_detection",
        "sequence kind must be explicit or deterministically detected",
    ),
    validation(
        "normalized_sequence",
        "sequence must be normalized before indexing or querying",
    ),
    validation(
        "length_policy",
        "empty records and over-limit records must be handled",
    ),
    validation(
        "stable_record_hash",
        "source hash and record id must be available for joins",
    ),
];

const fn validation(id: &'static str, description: &'static str) -> TemplateValidation {
    TemplateValidation { id, description }
}
