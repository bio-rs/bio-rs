use std::fs;
use std::path::PathBuf;

#[test]
fn wasm_docs_keep_local_browser_boundary_explicit() {
    let repo = repo_root();
    let doc = fs::read_to_string(repo.join("docs/wasm-api.md")).expect("WASM docs");

    for required in [
        "No hosted service or external runtime is required.",
        "no network access",
        "no `fetch` calls",
        "no external model calls",
        "no input uploads",
        "Tokenization currently supports FASTA only.",
        "does not currently export package-manifest validation",
    ] {
        assert!(
            doc.contains(required),
            "WASM docs missing boundary: {required}"
        );
    }
}

#[test]
fn wasm_browser_tests_cover_supported_and_rejected_browser_paths() {
    let repo = repo_root();
    let tests = fs::read_to_string(repo.join("crates/biors-wasm/tests/wasm_browser_api.rs"))
        .expect("WASM browser tests");

    for required in [
        "browser_policy_is_local_and_bounded",
        "browser_validation_accepts_supported_formats",
        "browser_tokenization_returns_fasta_tokens",
        "browser_tokenization_rejects_non_fasta_formats",
        "only FASTA",
    ] {
        assert!(
            tests.contains(required),
            "WASM browser tests missing {required}"
        );
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
