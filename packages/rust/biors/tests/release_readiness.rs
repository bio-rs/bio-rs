use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

mod common;

#[test]
fn full_workflow_e2e_covers_researcher_cli_path() {
    let repo = repo_root();
    let fasta = repo.join("examples/protein.fasta");
    let manifest = repo.join("examples/protein-package/manifest.json");
    let observations = repo.join("examples/protein-package/observations.json");

    let validation = run_biors(&["fasta", "validate"], &[&fasta]);
    assert_eq!(validation["data"]["records"], 1);
    assert_eq!(validation["data"]["error_count"], 0);

    let sequence_validation = run_biors(&["seq", "validate"], &[&fasta]);
    assert_eq!(sequence_validation["data"]["records"], 1);
    assert_eq!(sequence_validation["data"]["kind_counts"]["protein"], 1);

    let tokenized = run_biors(&["tokenize"], &[&fasta]);
    assert_eq!(tokenized["data"][0]["alphabet"], "protein-20");
    assert!(
        tokenized["data"][0]["tokens"]
            .as_array()
            .expect("tokens")
            .len()
            >= 4
    );

    let tokenizer = run_biors(
        &["tokenizer", "inspect", "--profile", "protein-20-special"],
        &[],
    );
    assert_eq!(tokenizer["data"]["profile"], "protein-20-special");
    assert_eq!(tokenizer["data"]["special_tokens"]["pad"]["token_id"], 21);

    let model_input = run_biors(&["model-input", "--max-length", "8"], &[&fasta]);
    assert_eq!(model_input["data"]["policy"]["max_length"], 8);
    assert_eq!(
        model_input["data"]["records"][0]["attention_mask"]
            .as_array()
            .expect("attention mask")
            .len(),
        8
    );

    let workflow = run_biors(&["workflow", "--max-length", "8"], &[&fasta]);
    assert_eq!(workflow["data"]["workflow"], "protein_model_input.v0");
    assert_eq!(workflow["data"]["model_ready"], true);
    assert_eq!(workflow["data"]["validation"]["records"], 1);
    assert_eq!(workflow["data"]["tokenization"]["summary"]["records"], 1);
    assert_eq!(workflow["data"]["model_input"]["policy"]["max_length"], 8);
    assert_eq!(
        workflow["data"]["provenance"]["invocation"]["command"],
        "biors workflow"
    );
    assert!(
        workflow["data"]["provenance"]["hashes"]["output_data_sha256"]
            .as_str()
            .expect("workflow output hash")
            .starts_with("sha256:")
    );

    let examples = repo.join("examples");
    let batch = run_biors(&["batch", "validate", "--kind", "auto"], &[&examples]);
    assert!(batch["data"]["summary"]["files"].as_u64().expect("files") >= 3);
    assert!(
        batch["data"]["summary"]["records"]
            .as_u64()
            .expect("records")
            >= 3
    );

    let package_validation = run_biors(&["package", "validate"], &[&manifest]);
    assert_eq!(package_validation["data"]["valid"], true);

    let package_verification = run_biors(&["package", "verify"], &[&manifest, &observations]);
    assert_eq!(package_verification["data"]["failed"], 0);
}

#[test]
fn release_readiness_documentation_surfaces_are_present_and_linked() {
    let repo = repo_root();
    let required = [
        "CITATION.cff",
        "docs/quickstart.md",
        "docs/demo.md",
        "docs/install.md",
        "docs/cli-contract.md",
        "docs/backend-architecture.md",
        "docs/candle-backend.md",
        "docs/error-codes.md",
        "docs/reliability.md",
        "docs/python-interop.md",
        "docs/wasm-readiness.md",
        "docs/public-contract-1.0-candidates.md",
        "docs/versioning.md",
        "docs/final-release-checklist.md",
    ];

    for path in required {
        assert!(
            repo.join(path).exists(),
            "missing release-readiness doc: {path}"
        );
    }

    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    for link in [
        "docs/quickstart.md",
        "docs/demo.md",
        "docs/install.md",
        "docs/cli-contract.md",
        "docs/backend-architecture.md",
        "docs/candle-backend.md",
        "docs/error-codes.md",
        "docs/reliability.md",
        "docs/python-interop.md",
        "docs/wasm-readiness.md",
        "docs/public-contract-1.0-candidates.md",
        "docs/versioning.md",
        "docs/final-release-checklist.md",
        "CITATION.cff",
    ] {
        assert!(readme.contains(link), "README does not link {link}");
    }

    let quickstart = fs::read_to_string(repo.join("docs/quickstart.md")).expect("read quickstart");
    let demo = fs::read_to_string(repo.join("docs/demo.md")).expect("read demo");
    let cli_contract =
        fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract");
    for (name, contents) in [
        ("quickstart", quickstart.as_str()),
        ("demo", demo.as_str()),
        ("CLI contract", cli_contract.as_str()),
    ] {
        assert!(
            contents.contains("biors --version"),
            "{name} does not document version verification"
        );
    }

    assert!(
        readme.contains("## Quickstart"),
        "README does not expose quickstart copy"
    );
    assert!(
        quickstart.contains("First 60 Seconds"),
        "quickstart does not expose first-impression commands"
    );
}

#[test]
fn readme_schema_inventory_lists_all_schema_files() {
    let repo = repo_root();
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    let schemas_dir = repo.join("schemas");

    for entry in fs::read_dir(&schemas_dir).expect("read schemas directory") {
        let entry = entry.expect("read schema entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("schema file name");
        assert!(
            readme.contains(file_name),
            "README schema inventory is missing schemas/{file_name}"
        );
    }
}

#[test]
fn citation_version_matches_workspace_package_version() {
    let repo = repo_root();
    let workspace_version = workspace_package_version(&repo);

    let citation = fs::read_to_string(repo.join("CITATION.cff")).expect("read citation metadata");
    let citation_version = citation
        .lines()
        .find_map(|line| line.strip_prefix("version: "))
        .map(|value| value.trim_matches('"'))
        .expect("citation version");

    assert_eq!(
        citation_version, workspace_version,
        "CITATION.cff version must match the workspace package version"
    );
}

#[test]
fn stale_benchmark_artifact_is_labeled_historical_in_readme() {
    let repo = repo_root();
    let workspace_version = workspace_package_version(&repo);
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    let benchmark: Value = serde_json::from_str(
        &fs::read_to_string(repo.join("benchmarks/fasta_vs_biopython.json"))
            .expect("read benchmark artifact"),
    )
    .expect("parse benchmark artifact");

    let benchmark_version = benchmark["environment"]["biors_core"]
        .as_str()
        .expect("benchmark biors-core version");

    if benchmark_version != workspace_version {
        assert!(
            readme.contains("Historical FASTA benchmark reference"),
            "stale benchmark artifacts must be visibly labeled historical"
        );
        assert!(
            readme.contains("not current-version performance evidence"),
            "README must not present stale benchmark numbers as current release evidence"
        );
        assert!(
            !readme.contains(&format!(
                "The `{workspace_version}` patch keeps those numeric claims pinned"
            )),
            "README must not tie stale numeric benchmark claims to the current version"
        );
    }
}

#[test]
fn final_release_checklist_covers_required_gates() {
    let repo = repo_root();
    let checklist =
        fs::read_to_string(repo.join("docs/final-release-checklist.md")).expect("read checklist");
    let script =
        fs::read_to_string(repo.join("scripts/check-final-release.sh")).expect("read final script");

    for expected in [
        "Full End-To-End Workflow Validation",
        "Public Contract Freeze",
        "Dependency Policy",
        "Breaking Change Cleanup",
        "Benchmark Artifact Coverage",
        "Release Artifact Contents",
        "Registry Preflight",
        "Version Tag",
        "Binary Release Test",
        "Install Flow Final Test",
        "GitHub Release Dry Run",
        "Public Demo Dry Run",
        "Final Release Checklist",
    ] {
        assert!(
            checklist.contains(expected),
            "final checklist missing {expected}"
        );
    }

    for expected in [
        "docs/pre-release-audit-main-2026-05-30.md",
        "scripts/check-dependency-policy.py",
        "Cargo.lock",
        "scripts/check-benchmark-docs.sh",
        "scripts/check-release-artifact-contents.py",
        "scripts/check-registry-versions.py",
        "LICENSE-APACHE",
        "LICENSE-MIT",
    ] {
        assert!(
            checklist.contains(expected),
            "final checklist missing release gate detail {expected}"
        );
    }

    assert!(
        !checklist.contains("No known breaking cleanup is deferred"),
        "final checklist must not claim there is no deferred cleanup while an audit queue exists"
    );

    for expected in [
        "scripts/check.sh",
        "scripts/check-security-audit.sh",
        "cargo build --locked --release -p biors",
        "BIORS_BIN=target/release/biors sh scripts/launch-demo.sh",
        "scripts/check-install-smoke.sh",
        "scripts/check-package-artifacts.sh",
        "python3 scripts/check-release-workflow.py",
    ] {
        assert!(
            script.contains(expected),
            "final release script missing {expected}"
        );
    }
}

#[test]
fn github_templates_cover_promoted_release_surfaces() {
    let repo = repo_root();
    let pr_template = fs::read_to_string(repo.join(".github/pull_request_template.md"))
        .expect("read PR template");
    let benchmark_template =
        fs::read_to_string(repo.join(".github/ISSUE_TEMPLATE/benchmark_performance_idea.md"))
            .expect("read benchmark issue template");

    for expected in [
        "scripts/test-python-wheel.py",
        "wasm-pack test --node packages/rust/biors-wasm",
        "MCP integration tests",
        "Package artifact changes",
        "Schema parity",
        "Dependency/advisory/license audit",
    ] {
        assert!(
            pr_template.contains(expected),
            "PR template missing promoted surface check: {expected}"
        );
    }

    for expected in [
        "model-input construction",
        "workflow or pipeline orchestration",
        "dataset inspect",
        "package validation or verification",
        "Python binding",
        "WASM/JavaScript binding",
        "MCP or service contract",
        "optional Candle backend",
        "binding or request overhead",
        "Surface and non-claim boundaries",
    ] {
        assert!(
            benchmark_template.contains(expected),
            "benchmark issue template missing promoted surface: {expected}"
        );
    }
}

#[test]
fn launch_demo_assets_cover_first_impression_workflow() {
    let repo = repo_root();
    let dataset = repo.join("examples/launch-demo.fasta");
    let script = repo.join("scripts/launch-demo.sh");
    let recorded_script = repo.join("scripts/record-cli-demo.sh");
    let demo = fs::read_to_string(repo.join("docs/demo.md")).expect("read demo doc");
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    let benchmark =
        fs::read_to_string(repo.join("benchmarks/fasta_vs_biopython.md")).expect("read benchmark");
    let fasta = fs::read_to_string(&dataset).expect("read launch demo FASTA");

    assert!(script.exists(), "missing launch demo script");
    assert!(recorded_script.exists(), "missing recorded CLI demo script");
    assert!(fasta.contains(">brca1_human_fragment"));
    assert!(fasta.contains(">cftr_human_fragment"));
    assert!(fasta.contains(">tp53_human_fragment"));

    for expected in [
        "Website Demo Script",
        "Contributor Demo",
        "CLI Recorded Demo Script",
        "scripts/record-cli-demo.sh",
        "Benchmark Visual Draft",
        "Deferred",
        "Browser playground: deferred to a later release pass",
    ] {
        assert!(demo.contains(expected), "demo doc missing {expected}");
    }
    assert!(
        readme.contains("docs/demo.md"),
        "README does not link the demo doc"
    );
    for expected in [
        "## Environment",
        "## Matched results",
        "## Raw result scope",
        "Human Reference Proteome",
        "reasonable claim",
        "Benchmark schema",
    ] {
        assert!(
            benchmark.contains(expected),
            "benchmark doc missing {expected}"
        );
    }

    let validation = run_biors(&["seq", "validate"], &[&dataset]);
    assert_eq!(validation["data"]["records"], 3);
    assert_eq!(validation["data"]["kind_counts"]["protein"], 3);

    let model_input = run_biors(&["model-input", "--max-length", "32"], &[&dataset]);
    assert_eq!(
        model_input["data"]["records"]
            .as_array()
            .expect("records")
            .len(),
        3
    );
    assert_eq!(model_input["data"]["policy"]["max_length"], 32);
}

#[test]
fn python_interop_examples_are_present_and_dependency_light() {
    let repo = repo_root();
    let required = [
        "examples/python/reference_preprocess.py",
        "examples/python/esm_from_biors_json.py",
        "examples/python/protbert_from_biors_json.py",
        "examples/python/pandas_numpy_friendly.py",
        "docs/python-interop.md",
    ];

    for path in required {
        assert!(
            repo.join(path).exists(),
            "missing Python interop asset: {path}"
        );
    }

    let docs = fs::read_to_string(repo.join("docs/python-interop.md")).expect("read Python docs");
    for expected in ["ESM", "ProtBERT", "pandas", "NumPy", "PyO3"] {
        assert!(
            docs.contains(expected),
            "Python interop docs missing {expected}"
        );
    }

    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    assert!(readme.contains("docs/python-interop.md"));
}

#[test]
fn candle_backend_stays_out_of_core_default_build() {
    let repo = repo_root();
    let core_manifest = fs::read_to_string(repo.join("packages/rust/biors-core/Cargo.toml"))
        .expect("read core manifest");
    let candle_manifest =
        fs::read_to_string(repo.join("packages/rust/biors-backend-candle/Cargo.toml"))
            .expect("read Candle backend manifest");
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");

    assert!(
        !core_manifest.contains("candle"),
        "biors-core must not depend on Candle"
    );
    assert!(
        candle_manifest.contains("candle-core.workspace"),
        "Candle backend crate must own the Candle dependency"
    );
    assert!(readme.contains("docs/candle-backend.md"));
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn workspace_package_version(repo: &Path) -> String {
    let workspace_manifest =
        fs::read_to_string(repo.join("Cargo.toml")).expect("read workspace manifest");
    let manifest: toml::Table = workspace_manifest
        .parse()
        .expect("parse workspace manifest");
    manifest
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .expect("workspace package version")
        .to_string()
}

fn run_biors(args: &[&str], paths: &[&Path]) -> Value {
    let output = common::run_biors_paths(args, paths);
    serde_json::from_slice(&output.stdout).expect("valid JSON")
}
