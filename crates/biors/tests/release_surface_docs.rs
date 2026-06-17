use std::fs;

mod common;

#[test]
fn service_interface_docs_list_request_response_examples() {
    let repo = common::repo_root();
    let service_doc =
        fs::read_to_string(repo.join("docs/service-interface.md")).expect("read service docs");

    for expected in [
        "Request And Response Schemas",
        "Local HTTP Mode",
        "biors serve",
        "GET /health",
        "GET /openapi.json",
        "POST /v0/batch/sequence/validate",
        "sequence.batch_validate",
        "service-batch-sequence-validate-output.v0.json",
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
    let pipeline_config_schema = fs::read_to_string(repo.join("schemas/pipeline-config.v0.json"))
        .expect("read pipeline config schema");
    let python_api = fs::read_to_string(repo.join("docs/python-api.md")).expect("read Python API");
    let python_stub =
        fs::read_to_string(repo.join("crates/biors-python/python/biors/__init__.pyi"))
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

#[test]
fn researcher_workflow_recipes_map_to_real_cli_and_mcp_surfaces() {
    let repo = common::repo_root();
    let workflows = fs::read_to_string(repo.join("docs/researcher-workflows.md"))
        .expect("read researcher workflows");
    let script = fs::read_to_string(repo.join("scripts/check-researcher-workflows.sh"))
        .expect("read researcher workflow check script");
    let cli_args =
        fs::read_to_string(repo.join("crates/biors/src/cli/args.rs")).expect("read CLI args");
    let package_args = fs::read_to_string(repo.join("crates/biors/src/cli/package_args.rs"))
        .expect("read package CLI args");
    let mcp_server = fs::read_to_string(repo.join("crates/biors-mcp-server/src/server.rs"))
        .expect("read MCP server");

    for recipe_id in [
        "validate-fasta-fastq",
        "validate-sequence-kinds",
        "protein-model-ready-workflow",
        "invalid-workflow-recovery",
        "molecule-structure-validation",
        "package-validate-verify-bridge",
        "local-report-json-output",
        "mcp-agent-sequence",
    ] {
        assert!(
            workflows.contains(recipe_id),
            "researcher workflow docs missing recipe id: {recipe_id}"
        );
        assert!(
            script.contains(recipe_id),
            "workflow check script missing recipe id: {recipe_id}"
        );
    }

    for mode in [
        "--list",
        "--happy",
        "--failure",
        "--package",
        "--check-local-only",
        "--all",
    ] {
        assert!(
            script.contains(mode),
            "workflow check script missing required mode: {mode}"
        );
    }

    for expected_command in [
        "fasta validate",
        "formats validate --format fastq",
        "seq validate --kind protein",
        "seq validate --kind dna",
        "seq validate --kind rna",
        "tokenize --profile protein-20",
        "model-input --max-length",
        "workflow --max-length",
        "molecule validate",
        "structure validate",
        "package inspect",
        "package validate",
        "package verify",
        "package bridge",
        "report generate",
    ] {
        assert!(
            workflows.contains(expected_command),
            "researcher workflow docs missing CLI command: {expected_command}"
        );
    }

    for cli_variant in [
        "pub enum FastaCommand",
        "pub enum FormatsCommand",
        "pub enum SeqCommand",
        "ModelInput",
        "Workflow",
        "Report",
    ] {
        assert!(
            cli_args.contains(cli_variant),
            "CLI inventory missing command marker: {cli_variant}"
        );
    }

    for package_variant in ["Inspect", "Validate", "Verify", "Bridge"] {
        assert!(
            package_args.contains(package_variant),
            "package CLI inventory missing command marker: {package_variant}"
        );
    }

    for tool in [
        "validate",
        "workflow",
        "package_validate_fields",
        "package_validate",
    ] {
        assert!(
            workflows.contains(tool),
            "researcher workflow docs missing MCP tool: {tool}"
        );
        assert!(
            mcp_server.contains(tool),
            "MCP server registration missing tool: {tool}"
        );
    }
}
