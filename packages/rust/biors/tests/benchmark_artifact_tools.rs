use std::fs;
use std::process::Command;

use serde_json::json;

mod common;

#[test]
fn compare_benchmark_artifacts_fails_on_removed_workload_by_default() {
    let temp = common::TempDir::new("benchmark-compare-missing");
    let before = temp.write(
        "before.json",
        &json!({
            "datasets": [{
                "label": "sample",
                "benchmarks": {
                    "parse": {
                        "bio-rs": {
                            "summary": {
                                "mean_s": 1.0,
                                "residues_per_sec": 100.0,
                                "peak_memory_bytes": 1024
                            }
                        }
                    },
                    "tokenize": {
                        "bio-rs": {
                            "summary": {
                                "mean_s": 2.0,
                                "residues_per_sec": 50.0,
                                "peak_memory_bytes": 2048
                            }
                        }
                    }
                }
            }]
        })
        .to_string(),
    );
    let after = temp.write(
        "after.json",
        &json!({
            "datasets": [{
                "label": "sample",
                "benchmarks": {
                    "parse": {
                        "bio-rs": {
                            "summary": {
                                "mean_s": 1.5,
                                "residues_per_sec": 90.0,
                                "peak_memory_bytes": 1024
                            }
                        }
                    }
                }
            }]
        })
        .to_string(),
    );

    let output = run_compare(&[before.to_str().unwrap(), after.to_str().unwrap()]);

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("| removed | workload | sample / tokenize |"));
    assert!(
        stdout.contains("| sample | parse | bio-rs | 1.000s | 1.500s | +50.0% | -10.0% | +0.0% |")
    );
}

#[test]
fn compare_benchmark_artifacts_allow_missing_reports_added_and_removed_coverage() {
    let temp = common::TempDir::new("benchmark-compare-allow-missing");
    let before = temp.write(
        "before.json",
        &json!({
            "datasets": [{
                "label": "sample",
                "benchmarks": {
                    "parse": {
                        "bio-rs": {
                            "summary": {
                                "mean_s": 1.0,
                                "residues_per_sec": 100.0
                            }
                        },
                        "baseline": {
                            "summary": {
                                "mean_s": 1.2,
                                "residues_per_sec": 80.0
                            }
                        }
                    }
                }
            }]
        })
        .to_string(),
    );
    let after = temp.write(
        "after.json",
        &json!({
            "datasets": [{
                "label": "sample",
                "benchmarks": {
                    "parse": {
                        "bio-rs": {
                            "summary": {
                                "mean_s": 0.8,
                                "residues_per_sec": 125.0
                            }
                        }
                    },
                    "validate": {
                        "bio-rs": {
                            "summary": {
                                "mean_s": 0.9,
                                "residues_per_sec": 110.0
                            }
                        }
                    }
                }
            }]
        })
        .to_string(),
    );

    let output = run_compare(&[
        "--allow-missing",
        before.to_str().unwrap(),
        after.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("| removed | implementation | sample / parse / baseline |"));
    assert!(stdout.contains("| added | workload | sample / validate |"));
}

fn run_compare(args: &[&str]) -> std::process::Output {
    let script = common::repo_root().join("scripts/compare-benchmark-artifacts.py");
    assert!(fs::metadata(&script)
        .expect("benchmark compare script")
        .is_file());
    Command::new("python3")
        .arg(script)
        .args(args)
        .output()
        .expect("run benchmark compare script")
}
