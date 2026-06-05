use std::fs;

mod common;

#[test]
fn python_binding_docs_do_not_require_in_repo_example_scripts() {
    let repo = common::repo_root();
    assert!(
        !repo.join("integrations/python").exists(),
        "Python integration examples should not be a renamed examples surface"
    );

    let docs = fs::read_to_string(repo.join("docs/python-api.md")).expect("read Python docs");
    for expected in ["caller-owned", "JSON boundary", "PyO3"] {
        assert!(
            docs.contains(expected),
            "Python binding docs missing boundary wording: {expected}"
        );
    }

    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    assert!(readme.contains("docs/python-api.md"));
}

#[test]
fn candle_backend_stays_out_of_core_default_build() {
    let repo = common::repo_root();
    let core_manifest =
        fs::read_to_string(repo.join("crates/biors-core/Cargo.toml")).expect("read core manifest");
    let candle_manifest = fs::read_to_string(repo.join("crates/biors-backend-candle/Cargo.toml"))
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
