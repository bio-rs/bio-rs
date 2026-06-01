"""Feature-level benchmark claim boundaries for committed artifacts."""

FEATURE_COVERAGE = [
    {
        "feature": "core_fasta_parse_validate_tokenize",
        "status": "numeric_public_claim",
        "claim_scope": "Matched FASTA parse, validation, and tokenization workloads recorded in this artifact.",
        "evidence": ["benchmarks.fasta_vs_biopython.v1 datasets[].benchmarks"],
    },
    {
        "feature": "core_fixed_length_model_input",
        "status": "criterion_regression_guard",
        "claim_scope": "Criterion guard only; no committed public numeric artifact yet.",
        "evidence": ["packages/rust/biors-core/benches/fasta_workloads.rs"],
    },
    {
        "feature": "cli_workflow",
        "status": "numeric_regression_guard",
        "claim_scope": "Committed CLI regression guard timing only; no public throughput claim.",
        "evidence": ["benchmarks/cli_surfaces.json workload cli_workflow_fixed_length"],
    },
    {
        "feature": "cli_dataset_inspect",
        "status": "numeric_regression_guard",
        "claim_scope": "Committed CLI regression guard timing only; no public throughput claim.",
        "evidence": ["benchmarks/cli_surfaces.json workload cli_dataset_inspect_many_file"],
    },
    {
        "feature": "python_bindings",
        "status": "numeric_regression_guard",
        "claim_scope": "Committed Python binding regression guard timing only; no public throughput claim.",
        "evidence": ["benchmarks/python_bindings.json"],
    },
    {
        "feature": "wasm_bindings",
        "status": "numeric_regression_guard",
        "claim_scope": "Committed Node.js WASM binding regression timings only; no browser or public throughput claim.",
        "evidence": ["benchmarks/wasm_bindings.json"],
    },
    {
        "feature": "mcp_service_contract",
        "status": "numeric_regression_guard",
        "claim_scope": "Committed service-contract CLI and in-process MCP request overhead timings only; no network throughput claim.",
        "evidence": [
            "benchmarks/cli_surfaces.json workload cli_service_contract",
            "benchmarks/mcp_server.json",
        ],
    },
    {
        "feature": "package_validation_bridge",
        "status": "numeric_regression_guard",
        "claim_scope": "Committed package validation and bridge CLI regression timings only; no public throughput claim.",
        "evidence": [
            "benchmarks/cli_surfaces.json workload cli_package_validate_example",
            "benchmarks/cli_surfaces.json workload cli_package_bridge_example",
        ],
    },
    {
        "feature": "candle_cpu_linear_probe",
        "status": "numeric_regression_guard",
        "claim_scope": "Committed optional Candle CPU backend smoke timing only; no broad backend throughput claim.",
        "evidence": ["benchmarks/backend_smoke.json"],
    },
    {
        "feature": "nucleotide_model_ready_workflows",
        "status": "numeric_regression_guard",
        "claim_scope": "Committed DNA/RNA validation, tokenization, model-input, workflow, and binding regression timings only; no public throughput claim.",
        "evidence": [
            "benchmarks/cli_surfaces.json workload cli_seq_validate_dna",
            "benchmarks/cli_surfaces.json workload cli_seq_validate_rna",
            "benchmarks/cli_surfaces.json workload cli_tokenize_dna_iupac",
            "benchmarks/cli_surfaces.json workload cli_tokenize_rna_iupac",
            "benchmarks/cli_surfaces.json workload cli_model_input_dna_iupac",
            "benchmarks/cli_surfaces.json workload cli_model_input_rna_iupac",
            "benchmarks/cli_surfaces.json workload cli_workflow_dna_iupac",
            "benchmarks/cli_surfaces.json workload cli_workflow_rna_iupac",
            "benchmarks/python_bindings.json workloads *_dna_iupac and *_rna_iupac",
            "benchmarks/wasm_bindings.json workloads *_dna_iupac and *_rna_iupac",
        ],
    },
]

REQUIRED_FEATURE_COVERAGE = {
    entry["feature"]: entry["status"] for entry in FEATURE_COVERAGE
}
