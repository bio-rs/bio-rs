use std::fs;

mod common;

#[test]
fn service_interface_docs_list_request_response_examples() {
    let repo = common::repo_root();
    let service_doc =
        fs::read_to_string(repo.join("docs/service-interface.md")).expect("read service docs");

    for expected in [
        "Request And Response Schemas",
        "sequence.validate",
        "sequence.inspect",
        "sequence.tokenize",
        "model_input.build",
        "package.inspect",
        "package.validate",
        "package.bridge.plan",
        "package.compatibility.compare",
        "fasta-validation-output.v0.json",
        "model-input-output.v0.json",
        "package-compatibility-output.v0.json",
    ] {
        assert!(
            service_doc.contains(expected),
            "service interface docs missing request/response example detail: {expected}"
        );
    }
}

#[test]
fn sequence_kind_support_matrix_covers_promoted_surfaces() {
    let repo = common::repo_root();
    let matrix =
        fs::read_to_string(repo.join("docs/sequence-kind-support.md")).expect("read matrix");
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    let service_tokenize_schema =
        fs::read_to_string(repo.join("schemas/service-sequence-tokenize-request.v0.json"))
            .expect("read service tokenize schema");
    let service_model_input_schema =
        fs::read_to_string(repo.join("schemas/service-model-input-request.v0.json"))
            .expect("read service model-input schema");
    let pipeline_config_schema = fs::read_to_string(repo.join("schemas/pipeline-config.v0.json"))
        .expect("read pipeline config schema");
    let python_api = fs::read_to_string(repo.join("docs/python-api.md")).expect("read Python API");
    let python_stub =
        fs::read_to_string(repo.join("packages/rust/biors-python/python/biors/__init__.pyi"))
            .expect("read Python stub");

    assert!(
        readme.contains("docs/sequence-kind-support.md"),
        "README must link the sequence-kind support matrix before broad DNA/RNA claims"
    );

    for expected in [
        "CLI `fasta validate` / `seq validate`",
        "CLI `batch validate`",
        "CLI `tokenize`",
        "CLI `model-input`",
        "CLI `workflow`",
        "CLI `pipeline --config`",
        "Python bindings",
        "WASM / JavaScript bindings",
        "MCP server",
        "Service contract schemas",
        "Package manifest validation",
        "Package conversion from Python/HF projects",
        "Benchmarks",
        "project-conversion limitations",
        "validate_fasta_input_with_kind",
    ] {
        assert!(
            matrix.contains(expected),
            "support matrix missing promoted surface or limitation: {expected}"
        );
    }

    for profile in [
        "protein-20",
        "protein-20-special",
        "dna-iupac",
        "dna-iupac-special",
        "rna-iupac",
        "rna-iupac-special",
    ] {
        assert!(
            matrix.contains(profile),
            "support matrix missing tokenizer profile: {profile}"
        );
        assert!(
            service_tokenize_schema.contains(profile),
            "service tokenize schema missing tokenizer profile: {profile}"
        );
        assert!(
            service_model_input_schema.contains(profile),
            "service model-input schema missing tokenizer profile: {profile}"
        );
        assert!(
            pipeline_config_schema.contains(profile),
            "pipeline config schema missing tokenizer profile: {profile}"
        );
    }

    for kind in ["protein", "dna", "rna"] {
        assert!(
            pipeline_config_schema.contains(kind),
            "pipeline config schema missing sequence kind: {kind}"
        );
    }

    for surface in [python_api.as_str(), python_stub.as_str()] {
        assert!(
            surface.contains("validate_fasta_input_with_kind"),
            "Python kind-aware validation helper must be documented and typed"
        );
    }
}
