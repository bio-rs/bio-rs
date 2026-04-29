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
        "CHANGELOG.md",
        "CITATION.cff",
        "docs/api-review.md",
        "docs/quickstart.md",
        "docs/professional-readiness.md",
        "docs/release-candidate-1.0.md",
        "docs/msrv.md",
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
        "docs/professional-readiness.md",
        "docs/api-review.md",
        "docs/release-candidate-1.0.md",
        "docs/msrv.md",
        "CHANGELOG.md",
        "CITATION.cff",
    ] {
        assert!(readme.contains(link), "README does not link {link}");
    }

    let readiness =
        fs::read_to_string(repo.join("docs/professional-readiness.md")).expect("read readiness");
    for marker in [
        "Phase 1",
        "Phase 2",
        "Researcher-Ready Scope",
        "Known Limits",
    ] {
        assert!(
            readiness.contains(marker),
            "readiness doc missing marker: {marker}"
        );
    }
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
