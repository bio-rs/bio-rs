use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

    let model_input = run_biors(&["model-input", "--max-length", "8"], &[&fasta]);
    assert_eq!(model_input["data"]["policy"]["max_length"], 8);
    assert_eq!(
        model_input["data"]["records"][0]["attention_mask"]
            .as_array()
            .expect("attention mask")
            .len(),
        8
    );

    let package_validation = run_biors(&["package", "validate"], &[&manifest]);
    assert_eq!(package_validation["data"]["valid"], true);

    let package_verification = run_biors(&["package", "verify"], &[&manifest, &observations]);
    assert_eq!(package_verification["data"]["failed"], 0);
}

#[test]
fn release_candidate_documentation_surfaces_are_present_and_linked() {
    let repo = repo_root();
    let required = [
        "CITATION.cff",
        "docs/quickstart.md",
        "docs/demo.md",
        "docs/cli-contract.md",
        "docs/error-codes.md",
        "docs/public-contract-1.0-candidates.md",
        "docs/versioning.md",
    ];

    for path in required {
        assert!(
            repo.join(path).exists(),
            "missing release-candidate doc: {path}"
        );
    }

    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    for link in [
        "docs/quickstart.md",
        "docs/demo.md",
        "docs/cli-contract.md",
        "docs/error-codes.md",
        "docs/public-contract-1.0-candidates.md",
        "docs/versioning.md",
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
}

#[test]
fn launch_demo_assets_cover_first_impression_workflow() {
    let repo = repo_root();
    let dataset = repo.join("examples/launch-demo.fasta");
    let script = repo.join("scripts/launch-demo.sh");
    let demo = fs::read_to_string(repo.join("docs/demo.md")).expect("read demo doc");
    let fasta = fs::read_to_string(&dataset).expect("read launch demo FASTA");

    assert!(script.exists(), "missing launch demo script");
    assert!(fasta.contains(">brca1_human_fragment"));
    assert!(fasta.contains(">cftr_human_fragment"));
    assert!(fasta.contains(">tp53_human_fragment"));

    for expected in [
        "Website Demo Script",
        "Contributor Demo",
        "Benchmark Visual Draft",
        "Browser Playground Concept",
        "Upload or paste FASTA",
    ] {
        assert!(demo.contains(expected), "demo doc missing {expected}");
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

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn run_biors(args: &[&str], paths: &[&Path]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(args)
        .args(paths)
        .output()
        .expect("run biors command");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("valid JSON")
}
