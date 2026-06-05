use std::fs;

mod common;

#[test]
fn release_readiness_documentation_surfaces_are_present_and_linked() {
    let repo = common::repo_root();
    let required = [
        "CITATION.cff",
        "docs/quickstart.md",
        "docs/install.md",
        "docs/cli-contract.md",
        "docs/candle-backend.md",
        "docs/error-codes.md",
        "docs/formats.md",
        "docs/molecule.md",
        "docs/package-conversion.md",
        "docs/package-format.md",
        "docs/pipeline-config.md",
        "docs/python-api.md",
        "docs/rust-api.md",
        "docs/sequence-kind-support.md",
        "docs/service-interface.md",
        "docs/structure.md",
        "docs/versioning.md",
        "docs/wasm-api.md",
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
        "docs/install.md",
        "docs/cli-contract.md",
        "docs/candle-backend.md",
        "docs/error-codes.md",
        "docs/formats.md",
        "docs/molecule.md",
        "docs/package-conversion.md",
        "docs/package-format.md",
        "docs/pipeline-config.md",
        "docs/python-api.md",
        "docs/rust-api.md",
        "docs/sequence-kind-support.md",
        "docs/service-interface.md",
        "docs/structure.md",
        "docs/versioning.md",
        "docs/wasm-api.md",
        "CITATION.cff",
    ] {
        assert!(readme.contains(link), "README does not link {link}");
    }

    let quickstart = fs::read_to_string(repo.join("docs/quickstart.md")).expect("read quickstart");
    let cli_contract =
        fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract");
    for (name, contents) in [
        ("quickstart", quickstart.as_str()),
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
fn readme_presents_full_bio_ai_contract_surface() {
    let repo = common::repo_root();
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");

    assert!(
        !readme.contains("FASTA -> validated sequence -> token IDs -> model-ready JSON"),
        "README must not reduce bio-rs to the old single FASTA pipeline"
    );
    assert!(
        !readme.contains("## Workspace"),
        "README should use contributor-facing repository language instead of Workspace"
    );

    for expected in [
        "raw sequence data + package metadata",
        "pipeline locks",
        "package manifests",
        "model artifacts",
        "Rust, CLI, Python, WASM, MCP, and service hosts",
        "Made For Sharing",
        "Repository Map",
        "biors seq validate --kind auto examples/multi.fasta",
        "workflow --profile dna-iupac",
        "biors package validate examples/protein-package/manifest.json",
        "biors service contract",
        "docs/formats.md",
        "docs/structure.md",
        "docs/sequence-kind-support.md",
        "docs/cli-contract.md",
    ] {
        assert!(
            readme.contains(expected),
            "README is missing broad bio-AI contract positioning: {expected}"
        );
    }
}

#[test]
fn cli_contract_schema_inventory_lists_all_schema_files() {
    let repo = common::repo_root();
    let cli_contract =
        fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract");
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
            cli_contract.contains(file_name),
            "CLI contract schema inventory is missing schemas/{file_name}"
        );
    }
}
