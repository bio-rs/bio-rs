use serde_json::Value;
use std::fs;
use std::path::Path;

mod common;

#[test]
fn launch_demo_assets_cover_first_impression_workflow() {
    let repo = common::repo_root();
    let dataset = repo.join("testdata/sequences/launch-demo.fasta");
    let script = repo.join("scripts/launch-demo.sh");
    let recorded_script = repo.join("scripts/record-cli-demo.sh");
    let benchmark =
        fs::read_to_string(repo.join("benchmarks/fasta_vs_biopython.md")).expect("read benchmark");
    let fasta = fs::read_to_string(&dataset).expect("read launch demo FASTA");
    let launch_script = fs::read_to_string(&script).expect("read launch demo script");
    let recorded_demo_script =
        fs::read_to_string(&recorded_script).expect("read recorded CLI demo script");

    assert!(script.exists(), "missing launch demo script");
    assert!(recorded_script.exists(), "missing recorded CLI demo script");
    assert!(fasta.contains(">brca1_human_fragment"));
    assert!(fasta.contains(">cftr_human_fragment"));
    assert!(fasta.contains(">tp53_human_fragment"));

    assert!(launch_script.contains("seq validate"));
    assert!(launch_script.contains("model-input"));
    assert!(launch_script.contains("report generate"));
    assert!(recorded_demo_script.contains("--version"));
    assert!(recorded_demo_script.contains("testdata/sequences/launch-demo.fasta"));
    assert!(recorded_demo_script.contains("report generate"));
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

fn run_biors(args: &[&str], paths: &[&Path]) -> Value {
    let output = common::run_biors_paths(args, paths);
    serde_json::from_slice(&output.stdout).expect("valid JSON")
}
