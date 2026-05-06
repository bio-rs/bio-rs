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
        "docs/error-codes.md",
        "docs/reliability.md",
        "docs/python-interop.md",
        "docs/wasm-readiness.md",
        "docs/public-contract-1.0-candidates.md",
        "docs/versioning.md",
        "docs/final-release-checklist.md",
        "CHANGELOG.md",
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
        "docs/error-codes.md",
        "docs/reliability.md",
        "docs/python-interop.md",
        "docs/wasm-readiness.md",
        "docs/public-contract-1.0-candidates.md",
        "docs/versioning.md",
        "docs/final-release-checklist.md",
        "CHANGELOG.md",
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
fn final_release_checklist_covers_required_gates() {
    let repo = repo_root();
    let checklist =
        fs::read_to_string(repo.join("docs/final-release-checklist.md")).expect("read checklist");
    let script =
        fs::read_to_string(repo.join("scripts/check-final-release.sh")).expect("read final script");

    for expected in [
        "Full End-To-End Workflow Validation",
        "Public Contract Freeze",
        "Breaking Change Cleanup",
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
        "scripts/check.sh",
        "cargo build --locked --release -p biors",
        "BIORS_BIN=target/release/biors sh scripts/launch-demo.sh",
        "scripts/check-install-smoke.sh",
        "python3 scripts/check-release-workflow.py",
    ] {
        assert!(
            script.contains(expected),
            "final release script missing {expected}"
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
    for expected in ["ESM", "ProtBERT", "pandas", "NumPy", "No PyO3"] {
        assert!(
            docs.contains(expected),
            "Python interop docs missing {expected}"
        );
    }

    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    assert!(readme.contains("docs/python-interop.md"));
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn run_biors(args: &[&str], paths: &[&Path]) -> Value {
    let output = common::run_biors_paths(args, paths);
    serde_json::from_slice(&output.stdout).expect("valid JSON")
}
