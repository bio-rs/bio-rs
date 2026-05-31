use std::fs;

mod common;

#[test]
fn python_interop_examples_are_present_and_dependency_light() {
    let repo = common::repo_root();
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
    let repo = common::repo_root();
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
