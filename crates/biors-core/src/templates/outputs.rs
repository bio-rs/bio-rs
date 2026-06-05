use super::TemplateOutputExpectation;

pub(super) static CLASSIFICATION_OUTPUTS: [TemplateOutputExpectation; 4] = [
    output(
        "label_namespace",
        "controlled label namespace used by the caller",
        true,
    ),
    output("scores", "per-label numeric scores or probabilities", true),
    output(
        "selected_label",
        "caller-selected label derived from scores",
        true,
    ),
    output(
        "model_provenance",
        "local model or package provenance supplied by caller",
        true,
    ),
];
pub(super) static EMBEDDING_OUTPUTS: [TemplateOutputExpectation; 4] = [
    output("embedding", "numeric vector for each sequence", true),
    output("dimension", "embedding vector length", true),
    output(
        "pooling_policy",
        "pooling policy used to produce the vector",
        true,
    ),
    output(
        "model_provenance",
        "local model or package provenance supplied by caller",
        true,
    ),
];
pub(super) static VARIANT_OUTPUTS: [TemplateOutputExpectation; 4] = [
    output(
        "effect_score",
        "numeric effect score for each variant",
        true,
    ),
    output(
        "effect_label",
        "caller-defined label derived from the score",
        true,
    ),
    output(
        "reference_alignment",
        "validated reference and alternate residue context",
        true,
    ),
    output(
        "model_provenance",
        "local model or package provenance supplied by caller",
        true,
    ),
];
pub(super) static MOLECULE_OUTPUTS: [TemplateOutputExpectation; 4] = [
    output("property_name", "predicted or measured property name", true),
    output(
        "property_value",
        "numeric or categorical property value",
        true,
    ),
    output("property_units", "units when the property is numeric", true),
    output(
        "model_provenance",
        "local model or package provenance supplied by caller",
        true,
    ),
];
pub(super) static STRUCTURE_OUTPUTS: [TemplateOutputExpectation; 4] = [
    output("valid", "aggregate structure validation status", true),
    output(
        "issues",
        "stable validation issue codes and locations",
        true,
    ),
    output(
        "chain_reports",
        "per-chain sequence and coordinate summaries",
        true,
    ),
    output(
        "sequence_mappings",
        "coordinate to SEQRES mapping summaries",
        true,
    ),
];
pub(super) static SEARCH_OUTPUTS: [TemplateOutputExpectation; 4] = [
    output(
        "preprocessed_records",
        "normalized records ready for caller search/index code",
        true,
    ),
    output(
        "deduplication_keys",
        "stable keys available for caller-side deduplication",
        true,
    ),
    output(
        "validation_issues",
        "warnings and errors observed during preprocessing",
        true,
    ),
    output(
        "no_ranking_claim",
        "template does not execute alignment, ANN, or ranking",
        true,
    ),
];

const fn output(
    name: &'static str,
    description: &'static str,
    required: bool,
) -> TemplateOutputExpectation {
    TemplateOutputExpectation {
        name,
        description,
        required,
    }
}
