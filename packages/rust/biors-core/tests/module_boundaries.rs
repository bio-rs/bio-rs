use std::fs;
use std::path::PathBuf;

fn core_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read_src(path: &str) -> String {
    fs::read_to_string(core_src().join(path)).expect("read core source file")
}

#[test]
fn package_facade_delegates_to_focused_modules() {
    let package = read_src("package.rs");

    for module in [
        "mod artifacts;",
        "mod checksum;",
        "mod runtime;",
        "mod summary;",
        "mod validation;",
        "mod types;",
    ] {
        assert!(package.contains(module), "package.rs missing {module}");
    }

    assert!(
        !package.contains("use sha2"),
        "checksum implementation must live outside the package facade"
    );
    assert!(
        !package.contains("fn validate_artifact"),
        "artifact validation must live outside the package facade"
    );
}

#[test]
fn sequence_facade_delegates_residue_normalization_and_reports() {
    let sequence = read_src("sequence.rs");

    for module in [
        "mod normalization;",
        "mod report;",
        "mod residue;",
        "mod types;",
        "mod validation;",
    ] {
        assert!(sequence.contains(module), "sequence.rs missing {module}");
    }

    assert!(
        !sequence.contains("const PROTEIN_20_RESIDUE_LOOKUP"),
        "residue lookup tables must live in the residue module"
    );
}

#[test]
fn verification_facade_delegates_fixture_workflow() {
    let verification = read_src("verification.rs");

    assert!(
        verification.contains("mod fixtures;"),
        "fixture verification workflow must live in a dedicated module"
    );
    assert!(
        !verification.contains("manifest.fixtures.iter().map"),
        "verification facade must not own fixture iteration details"
    );
}

#[test]
fn fasta_scanner_encapsulates_ascii_unchecked_boundaries() {
    let scanner = read_src("fasta_scan.rs");
    let direct_unchecked_calls = scanner.matches("std::str::from_utf8_unchecked").count();

    assert_eq!(
        direct_unchecked_calls, 1,
        "unchecked UTF-8 conversion must be centralized in one helper"
    );
    assert!(
        scanner.contains("fn ascii_utf8_unchecked"),
        "FASTA scanner must name the ASCII-only unchecked boundary"
    );
}
