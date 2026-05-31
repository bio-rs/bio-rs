#!/usr/bin/env python3
"""Validate release workflow invariants that affect package publication."""

from pathlib import Path


WORKFLOW = Path(".github/workflows/release.yml")
RELEASE_TOOL_VERSIONS = Path("scripts/release-tool-versions.env")
UNUSED_RELEASE_TEMPLATE = Path(".github/release_template.md")
RUST_TOOLCHAIN_ACTION = "dtolnay/rust-toolchain@98e1b82157cd469e843cb7f524c1313b4ad9492c"


def main() -> None:
    if UNUSED_RELEASE_TEMPLATE.exists():
        raise SystemExit(
            ".github/release_template.md is not used by the release workflow; "
            "keep GitHub --generate-notes as the release body source of truth"
        )

    tool_versions = read_release_tool_versions()
    lines = WORKFLOW.read_text(encoding="utf-8").splitlines()

    publish_order = [
        "pre-tag-registry-check:",
        "- name: Check package versions are unpublished",
        "- name: Dry-run publish biors-core",
        "- name: Publish biors-core",
        "- name: Wait for biors-core index",
        "- name: Dry-run publish biors-mcp-server",
        "- name: Publish biors-mcp-server",
        "- name: Dry-run publish biors-backend-candle",
        "- name: Publish biors-backend-candle",
        "- name: Wait for biors-backend-candle index",
        "- name: Dry-run publish biors",
        "- name: Publish biors",
        "build-python-wheels:",
        "- name: Check Python distribution license files",
        "- name: Check Python source distribution license files",
        "- name: Test installed Python wheel",
        "publish-python:",
        "- name: Publish Python distributions to PyPI with token",
        "- name: Publish Python distributions to PyPI with trusted publishing",
        "publish-wasm-npm:",
        "- name: Test WASM package",
        "- name: Build npm package",
        "- name: Check npm package artifact contents",
        "- name: Publish npm package with trusted publishing",
        "build-release-binaries:",
        "- name: Package release binary",
        "- name: Check binary archive contents",
        "- name: Upload binary artifact",
        "create-github-release:",
        "- name: Download binary artifacts",
        "- name: Create release if missing",
    ]

    positions: list[tuple[str, int]] = []
    for marker in publish_order:
        matching_lines = [
            line_number
            for line_number, line in enumerate(lines, start=1)
            if line.strip() == marker
        ]
        if not matching_lines:
            raise SystemExit(f"release workflow is missing step: {marker}")
        positions.append((marker, matching_lines[0]))

    required_text = [
        f"BIORS_RELEASE_MATURIN_VERSION: '{tool_versions['BIORS_RELEASE_MATURIN_VERSION']}'",
        f"BIORS_RELEASE_WASM_PACK_VERSION: '{tool_versions['BIORS_RELEASE_WASM_PACK_VERSION']}'",
        f"BIORS_RELEASE_NODE_VERSION: '{tool_versions['BIORS_RELEASE_NODE_VERSION']}'",
        "node-version: ${{ env.BIORS_RELEASE_NODE_VERSION }}",
        '"maturin==${BIORS_RELEASE_MATURIN_VERSION}"',
        'cargo install wasm-pack --locked --version "${BIORS_RELEASE_WASM_PACK_VERSION}"',
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
        "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
        "actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c",
        "pypa/gh-action-pypi-publish@cef221092ed1bacb1cc03d23a2d87d1d172e277b",
        "PYPI_API_TOKEN: ${{ secrets.PYPI_API_TOKEN }}",
        "wasm-pack test --node packages/rust/biors-wasm",
        "scripts/build-wasm-npm-package.sh",
        "scripts/check-release-artifact-contents.py python-dist dist",
        "scripts/test-python-wheel.py --dist-dir dist",
        "scripts/check-release-artifact-contents.py wasm-package packages/rust/biors-wasm/pkg",
        'scripts/check-release-artifact-contents.py binary-tarball "${{ matrix.archive }}"',
        "npm publish packages/rust/biors-wasm/pkg --access public",
        "tar -C dist -czf \"${{ matrix.archive }}\" biors README.md LICENSE-APACHE LICENSE-MIT",
        "scripts/check-registry-versions.py",
        "cargo install --locked cargo-deny",
        "scripts/check-security-audit.sh",
        "--generate-notes",
        "dist/*.tar.gz",
    ]
    workflow_text = "\n".join(lines)
    for text in required_text:
        if text not in workflow_text:
            raise SystemExit(f"release workflow is missing binary packaging text: {text}")

    assert_release_tool_scripts_use_pins(tool_versions)

    assert_rust_toolchain_for_cargo_jobs(lines)

    out_of_order = [
        (previous, current)
        for (previous, previous_position), (current, current_position) in zip(
            positions, positions[1:]
        )
        if previous_position >= current_position
    ]
    if out_of_order:
        details = "; ".join(
            f"{previous} must appear before {current}"
            for previous, current in out_of_order
        )
        raise SystemExit(f"release workflow publish order is unsafe: {details}")


def assert_rust_toolchain_for_cargo_jobs(lines: list[str]) -> None:
    missing = []
    for job_name, job_lines in workflow_jobs(lines).items():
        body = "\n".join(job_lines)
        needs_rust = any(
            marker in body
            for marker in ["cargo ", "scripts/check.sh", "maturin ", "wasm-pack "]
        )
        if needs_rust and RUST_TOOLCHAIN_ACTION not in body:
            missing.append(job_name)

    if missing:
        raise SystemExit(
            "release workflow jobs that run Rust tooling must install the pinned Rust toolchain: "
            + ", ".join(sorted(missing))
        )


def workflow_jobs(lines: list[str]) -> dict[str, list[str]]:
    jobs: dict[str, list[str]] = {}
    current: str | None = None
    in_jobs = False

    for line in lines:
        if line == "jobs:":
            in_jobs = True
            continue
        if not in_jobs:
            continue
        if line and not line.startswith(" "):
            break
        if line.startswith("  ") and not line.startswith("    ") and line.strip().endswith(":"):
            current = line.strip()[:-1]
            jobs[current] = []
            continue
        if current is not None:
            jobs[current].append(line)

    return jobs


def read_release_tool_versions() -> dict[str, str]:
    versions = {}
    for line in RELEASE_TOOL_VERSIONS.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        key, value = line.split("=", 1)
        versions[key] = value

    required = {
        "BIORS_RELEASE_MATURIN_VERSION",
        "BIORS_RELEASE_WASM_PACK_VERSION",
        "BIORS_RELEASE_NODE_VERSION",
    }
    missing = sorted(required - versions.keys())
    if missing:
        raise SystemExit(f"release tool versions file is missing: {', '.join(missing)}")
    return versions


def assert_release_tool_scripts_use_pins(tool_versions: dict[str, str]) -> None:
    package_artifacts = Path("scripts/check-package-artifacts.sh").read_text(encoding="utf-8")
    wasm_package = Path("scripts/build-wasm-npm-package.sh").read_text(encoding="utf-8")

    for text, name, script in [
        (
            "maturin==$BIORS_RELEASE_MATURIN_VERSION",
            "maturin pin",
            package_artifacts,
        ),
        (
            "cargo install wasm-pack --locked --version $BIORS_RELEASE_WASM_PACK_VERSION",
            "wasm-pack pin",
            wasm_package,
        ),
    ]:
        if text not in script:
            raise SystemExit(f"release tool local script is missing {name}: {text}")

    for key, version in tool_versions.items():
        if not version or version.count(".") < 2:
            raise SystemExit(f"{key} must be pinned to an exact patch version, got {version!r}")


if __name__ == "__main__":
    main()
