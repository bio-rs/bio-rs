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
        "status": "not_benchmarked",
        "claim_scope": "No numeric performance claim.",
        "evidence": ["missing committed WASM/Node benchmark artifact"],
    },
    {
        "feature": "mcp_service_contract",
        "status": "not_benchmarked",
        "claim_scope": "No numeric performance claim.",
        "evidence": ["missing committed MCP/service benchmark artifact"],
    },
    {
        "feature": "package_validation_bridge",
        "status": "not_benchmarked",
        "claim_scope": "No numeric performance claim.",
        "evidence": ["missing committed package validation/bridge benchmark artifact"],
    },
    {
        "feature": "candle_cpu_linear_probe",
        "status": "criterion_regression_guard",
        "claim_scope": "Synthetic Criterion guard only; no committed public numeric artifact yet.",
        "evidence": ["packages/rust/biors-backend-candle/benches/candle_linear_probe.rs"],
    },
]

REQUIRED_FEATURE_COVERAGE = {
    entry["feature"]: entry["status"] for entry in FEATURE_COVERAGE
}
