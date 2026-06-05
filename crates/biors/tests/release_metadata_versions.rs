use serde_json::Value;
use std::fs;

mod common;
mod release_support;

use release_support::workspace_package_version;

#[test]
fn citation_version_matches_workspace_package_version() {
    let repo = common::repo_root();
    let workspace_version = workspace_package_version(&repo);

    let citation = fs::read_to_string(repo.join("CITATION.cff")).expect("read citation metadata");
    let citation_version = citation
        .lines()
        .find_map(|line| line.strip_prefix("version: "))
        .map(|value| value.trim_matches('"'))
        .expect("citation version");

    assert_eq!(
        citation_version, workspace_version,
        "CITATION.cff version must match the workspace package version"
    );
}

#[test]
fn example_package_metadata_versions_match_workspace_package_version() {
    let repo = common::repo_root();
    let workspace_version = workspace_package_version(&repo);
    let manifest_path = repo.join("testdata/protein-package/manifest.json");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(&manifest_path).expect("read example package manifest"),
    )
    .expect("parse example package manifest");
    let citation = fs::read_to_string(repo.join("testdata/protein-package/docs/CITATION.cff"))
        .expect("read example package citation");
    let package_format =
        fs::read_to_string(repo.join("docs/package-format.md")).expect("read package format doc");
    let release_script =
        fs::read_to_string(repo.join("scripts/check-final-release.sh")).expect("read final script");

    assert_eq!(
        manifest["metadata"]["citation"]["preferred_citation"],
        format!("bio-rs protein package fixture, version {workspace_version}")
    );
    assert!(
        citation.contains(&format!("version: \"{workspace_version}\"")),
        "example package citation version must match workspace package version"
    );

    for (name, contents) in [
        ("example package manifest", manifest.to_string()),
        ("example package citation", citation),
        ("package format doc", package_format),
    ] {
        assert!(
            !contents.contains("0.31.0"),
            "{name} still contains stale package fixture version text"
        );
    }
    assert!(
        release_script.contains("scripts/check-package-artifacts.sh"),
        "final release gate must verify package artifacts after metadata version changes"
    );
}

#[test]
fn service_template_versions_match_workspace_package_version() {
    let repo = common::repo_root();
    let workspace_version = workspace_package_version(&repo);

    for path in [
        "deploy/service/Dockerfile",
        "deploy/service/README.md",
        "docs/service-interface.md",
        "docs/molecule.md",
    ] {
        let contents =
            fs::read_to_string(repo.join(path)).unwrap_or_else(|_| panic!("read {path}"));
        assert!(
            contents.contains(&workspace_version),
            "{path} must mention current workspace version {workspace_version}"
        );
        for stale in ["0.54.0", "0.50.0"] {
            assert!(
                !contents.contains(stale),
                "{path} still contains stale version {stale}"
            );
        }
    }

    let release_prep =
        fs::read_to_string(repo.join("scripts/prepare-release-version.py")).expect("read prep");
    for path in [
        "deploy/service/Dockerfile",
        "deploy/service/README.md",
        "docs/service-interface.md",
        "docs/molecule.md",
    ] {
        assert!(
            release_prep.contains(path),
            "release prep script must update {path}"
        );
    }
}

#[test]
fn stale_benchmark_artifact_is_labeled_historical_in_readme() {
    let repo = common::repo_root();
    let workspace_version = workspace_package_version(&repo);
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    let benchmark: Value = serde_json::from_str(
        &fs::read_to_string(repo.join("benchmarks/fasta_vs_biopython.json"))
            .expect("read benchmark artifact"),
    )
    .expect("parse benchmark artifact");

    let benchmark_version = benchmark["environment"]["biors_core"]
        .as_str()
        .expect("benchmark biors-core version");

    if benchmark_version != workspace_version {
        assert!(
            readme.contains("Historical FASTA benchmark reference"),
            "stale benchmark artifacts must be visibly labeled historical"
        );
        assert!(
            readme.contains("not current-version performance evidence"),
            "README must not present stale benchmark numbers as current release evidence"
        );
        assert!(
            !readme.contains(&format!(
                "The `{workspace_version}` patch keeps those numeric claims pinned"
            )),
            "README must not tie stale numeric benchmark claims to the current version"
        );
    }
}
