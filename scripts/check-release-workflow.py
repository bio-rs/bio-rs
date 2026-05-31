#!/usr/bin/env python3
"""Validate release workflow invariants that affect package publication."""

from pathlib import Path


WORKFLOW = Path(".github/workflows/release.yml")


def main() -> None:
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
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
        "actions/upload-artifact@v7",
        "actions/download-artifact@v8",
        "pypa/gh-action-pypi-publish@release/v1",
        "PYPI_API_TOKEN: ${{ secrets.PYPI_API_TOKEN }}",
        "wasm-pack test --node packages/rust/biors-wasm",
        "scripts/build-wasm-npm-package.sh",
        "scripts/check-release-artifact-contents.py python-dist dist",
        "scripts/check-release-artifact-contents.py wasm-package packages/rust/biors-wasm/pkg",
        'scripts/check-release-artifact-contents.py binary-tarball "${{ matrix.archive }}"',
        "npm publish packages/rust/biors-wasm/pkg --access public",
        "tar -C dist -czf \"${{ matrix.archive }}\" biors README.md LICENSE-APACHE LICENSE-MIT",
        "scripts/check-registry-versions.py",
        "dist/*.tar.gz",
    ]
    workflow_text = "\n".join(lines)
    for text in required_text:
        if text not in workflow_text:
            raise SystemExit(f"release workflow is missing binary packaging text: {text}")

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


if __name__ == "__main__":
    main()
