#!/usr/bin/env python3
"""Prepare a bio-rs patch release version across package metadata and fixtures."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path


REPO = Path(__file__).resolve().parents[1]
VERSION_PATTERN = re.compile(r"^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$")

TEXT_VERSION_FILES = [
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "CITATION.cff",
    "docs/install.md",
    "docs/quickstart.md",
    "docs/formats.md",
    "docs/rust-api.md",
    "benchmarks/fasta_vs_biopython.md",
    "benchmarks/fasta_vs_biopython.json",
    "benchmarks/backend_smoke.json",
    "benchmarks/cli_surfaces.json",
    "benchmarks/mcp_server.json",
    "benchmarks/python_bindings.json",
    "benchmarks/wasm_bindings.json",
    "packages/rust/biors-python/pyproject.toml",
    "packages/rust/biors-wasm/package.json",
    "examples/protein-package/docs/CITATION.cff",
]


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Update release version metadata, example package checksums, and the "
            "checked-in pipeline lockfile in one command."
        ),
    )
    parser.add_argument("version", help="new release version, for example 0.48.0")
    parser.add_argument(
        "--verify",
        action="store_true",
        help="run focused release-prep checks after updating files",
    )
    args = parser.parse_args()

    new_version = args.version
    if not VERSION_PATTERN.match(new_version):
        raise SystemExit(f"invalid version: {new_version}")

    current_version = workspace_version()
    if current_version == new_version:
        raise SystemExit(f"workspace is already at {new_version}")

    assert_clean_worktree()
    replace_versions(current_version, new_version)
    update_example_manifest(new_version)
    regenerate_pipeline_lock()

    if args.verify:
        verify_release_prep()

    print(f"prepared bio-rs {current_version} -> {new_version}")
    return 0


def workspace_version() -> str:
    metadata = run(["cargo", "metadata", "--no-deps", "--format-version", "1"])
    parsed = json.loads(metadata)
    for package in parsed["packages"]:
        if package["name"] == "biors":
            return package["version"]
    raise SystemExit("cargo metadata did not include biors")


def assert_clean_worktree() -> None:
    status = run(["git", "status", "--short"])
    if status:
        raise SystemExit(
            "working tree must be clean before preparing a release version\n"
            "commit, stash, or remove local changes first"
        )


def replace_versions(current_version: str, new_version: str) -> None:
    missing: list[str] = []
    for relative_path in TEXT_VERSION_FILES:
        path = REPO / relative_path
        text = path.read_text(encoding="utf-8")
        updated = text.replace(current_version, new_version)
        if updated == text:
            missing.append(relative_path)
            continue
        path.write_text(updated, encoding="utf-8")

    if missing:
        joined = "\n".join(f"- {path}" for path in missing)
        raise SystemExit(f"version string was not found in expected file(s):\n{joined}")


def update_example_manifest(version: str) -> None:
    manifest_path = REPO / "examples/protein-package/manifest.json"
    citation_path = REPO / "examples/protein-package/docs/CITATION.cff"
    checksum = hashlib.sha256(citation_path.read_bytes()).hexdigest()

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    citation = manifest["metadata"]["citation"]
    citation["preferred_citation"] = f"bio-rs protein package fixture, version {version}"
    citation["file"]["checksum"] = f"sha256:{checksum}"

    manifest_path.write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def regenerate_pipeline_lock() -> None:
    run(
        [
            "cargo",
            "run",
            "--locked",
            "-p",
            "biors",
            "--",
            "pipeline",
            "--config",
            "examples/protein-package/pipelines/protein.toml",
            "--package",
            "examples/protein-package/manifest.json",
            "--write-lock",
            "examples/pipeline/pipeline.lock",
        ]
    )


def verify_release_prep() -> None:
    checks = [
        ["python3", "scripts/check-registry-versions.py"],
        ["python3", "scripts/check-release-workflow.py"],
        [
            "cargo",
            "test",
            "-p",
            "biors",
            "--test",
            "cli_pipeline_lock",
            "checked_in_pipeline_lock_matches_current_generator",
        ],
        ["cargo", "test", "-p", "biors", "--test", "release_metadata_versions"],
        ["cargo", "test", "-p", "biors", "--test", "release_doc_inventory"],
        ["cargo", "test", "-p", "biors", "--test", "release_contributor_docs"],
        ["cargo", "test", "-p", "biors", "--test", "release_gate_policy"],
    ]
    for check in checks:
        run(check)


def run(command: list[str]) -> str:
    completed = subprocess.run(
        command,
        cwd=REPO,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return completed.stdout.strip()


if __name__ == "__main__":
    raise SystemExit(main())
