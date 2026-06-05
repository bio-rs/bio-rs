use biors_core::{
    formats::BioFormat,
    templates::{
        find_task_template, task_template_ids, task_templates, CoreReaderSupport, TemplateEntity,
        TASK_TEMPLATE_SCHEMA_VERSION,
    },
};

#[test]
fn task_template_ids_are_stable_and_ordered() {
    assert_eq!(
        task_template_ids(),
        &[
            "protein-classification-v0",
            "protein-embedding-generation-v0",
            "variant-effect-prediction-v0",
            "molecule-property-prediction-v0",
            "structure-validation-v0",
            "sequence-similarity-preprocess-v0",
        ]
    );

    let listed_ids: Vec<_> = task_templates()
        .iter()
        .map(|template| template.id)
        .collect();
    assert_eq!(listed_ids, task_template_ids());
}

#[test]
fn task_templates_include_required_sections_and_local_execution_boundary() {
    for template in task_templates() {
        assert_eq!(template.schema_version, TASK_TEMPLATE_SCHEMA_VERSION);
        assert!(template.id.ends_with("-v0"), "{template:?}");
        assert!(!template.supported_inputs.is_empty(), "{template:?}");
        assert!(!template.required_validations.is_empty(), "{template:?}");
        assert!(!template.model_ready_fields.is_empty(), "{template:?}");
        assert!(!template.output_expectations.is_empty(), "{template:?}");
        assert!(!template.execution.external_model_calls, "{template:?}");
        assert!(!template.execution.uploads_input_data, "{template:?}");
        assert_eq!(template.execution.network_access, "none");

        for input in template.supported_inputs {
            assert!(!input.formats.is_empty(), "{template:?}");
            assert!(!input.required_fields.is_empty(), "{template:?}");
        }
        assert!(
            template
                .model_ready_fields
                .iter()
                .any(|field| field.required),
            "{template:?}"
        );
        assert!(
            template
                .output_expectations
                .iter()
                .any(|output| output.required),
            "{template:?}"
        );
    }
}

#[test]
fn query_api_returns_expected_template_fields() {
    let classification =
        find_task_template("protein-classification-v0").expect("classification template");
    assert!(field_names(classification).contains(&"input_ids"));
    assert!(field_names(classification).contains(&"attention_mask"));

    let variant = find_task_template("variant-effect-prediction-v0").expect("variant template");
    assert!(field_names(variant).contains(&"alternate_residue"));
    assert!(field_names(variant).contains(&"context_input_ids"));

    let molecule =
        find_task_template("molecule-property-prediction-v0").expect("molecule template");
    assert!(field_names(molecule).contains(&"canonical_graph_key"));
    assert!(field_names(molecule).contains(&"fingerprint"));

    let structure = find_task_template("structure-validation-v0").expect("structure template");
    assert!(field_names(structure).contains(&"atom_coordinates"));
    assert!(output_names(structure).contains(&"issues"));

    let search = find_task_template("sequence-similarity-preprocess-v0").expect("search template");
    assert!(field_names(search).contains(&"normalized_sequence"));
    assert!(output_names(search).contains(&"no_ranking_claim"));

    assert!(find_task_template("missing-template").is_none());
}

#[test]
fn variant_table_inputs_do_not_claim_core_parser_support() {
    let variant = find_task_template("variant-effect-prediction-v0").expect("variant template");
    let table_input = variant
        .supported_inputs
        .iter()
        .find(|input| input.entity == TemplateEntity::ProteinVariant)
        .expect("variant table input");

    let formats: Vec<_> = table_input
        .formats
        .iter()
        .map(|format| format.format)
        .collect();
    assert_eq!(formats, vec![BioFormat::Csv, BioFormat::Tsv]);
    assert!(table_input
        .formats
        .iter()
        .all(|format| format.core_reader == CoreReaderSupport::ContractOnly));
}

fn field_names(template: &biors_core::templates::TaskTemplate) -> Vec<&'static str> {
    template
        .model_ready_fields
        .iter()
        .map(|field| field.name)
        .collect()
}

fn output_names(template: &biors_core::templates::TaskTemplate) -> Vec<&'static str> {
    template
        .output_expectations
        .iter()
        .map(|output| output.name)
        .collect()
}
