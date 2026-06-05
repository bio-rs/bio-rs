use super::TemplateModelField;

pub(super) static CLASSIFICATION_FIELDS: [TemplateModelField; 5] = [
    field("sequence_id", "stable source sequence id", true),
    field("sequence_length", "normalized residue count", true),
    field(
        "tokenizer_profile",
        "tokenizer profile used for input IDs",
        true,
    ),
    field(
        "input_ids",
        "token IDs after normalization and truncation policy",
        true,
    ),
    field("attention_mask", "mask aligned to input_ids", true),
];
pub(super) static EMBEDDING_FIELDS: [TemplateModelField; 5] = [
    field("sequence_id", "stable source sequence id", true),
    field(
        "tokenizer_profile",
        "tokenizer profile used for input IDs",
        true,
    ),
    field(
        "input_ids",
        "token IDs after normalization and truncation policy",
        true,
    ),
    field("attention_mask", "mask aligned to input_ids", true),
    field(
        "pooling_policy",
        "caller-selected embedding pooling description",
        true,
    ),
];
pub(super) static VARIANT_FIELDS: [TemplateModelField; 7] = [
    field("variant_id", "stable variant id", true),
    field("sequence_id", "reference sequence id", true),
    field("position", "one-based protein residue position", true),
    field("reference_residue", "validated residue at position", true),
    field(
        "alternate_residue",
        "validated alternate protein residue",
        true,
    ),
    field(
        "context_input_ids",
        "token IDs for the validated context window",
        true,
    ),
    field(
        "context_attention_mask",
        "mask aligned to context_input_ids",
        true,
    ),
];
pub(super) static MOLECULE_FIELDS: [TemplateModelField; 6] = [
    field("molecule_id", "stable molecule id", true),
    field(
        "canonical_graph_key",
        "deterministic atom and bond graph key",
        true,
    ),
    field("formula", "derived molecular formula", true),
    field("exact_mass", "derived exact mass", true),
    field(
        "descriptor_values",
        "named deterministic descriptor values",
        true,
    ),
    field("fingerprint", "deterministic hashed fingerprint", true),
];
pub(super) static STRUCTURE_FIELDS: [TemplateModelField; 6] = [
    field("structure_id", "stable structure id when available", true),
    field("chain_id", "validated chain identifier", true),
    field(
        "coordinate_sequence",
        "coordinate-derived protein sequence",
        true,
    ),
    field(
        "residue_positions",
        "source residue positions aligned to sequence",
        true,
    ),
    field(
        "atom_coordinates",
        "finite atom coordinates grouped by residue",
        true,
    ),
    field(
        "sequence_mapping_status",
        "explicit coordinate to SEQRES mapping status",
        true,
    ),
];
pub(super) static SEARCH_FIELDS: [TemplateModelField; 5] = [
    field("record_id", "stable source record id", true),
    field("sequence_kind", "protein, DNA, or RNA", true),
    field(
        "normalized_sequence",
        "normalized sequence used for preprocessing",
        true,
    ),
    field("sequence_length", "normalized residue or base count", true),
    field(
        "source_hash",
        "stable input or record hash for traceability",
        true,
    ),
];

const fn field(
    name: &'static str,
    description: &'static str,
    required: bool,
) -> TemplateModelField {
    TemplateModelField {
        name,
        description,
        required,
    }
}
