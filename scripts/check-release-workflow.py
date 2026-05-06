#!/usr/bin/env python3
"""Validate release workflow invariants that affect crates.io publication."""

from pathlib import Path


WORKFLOW = Path(".github/workflows/release.yml")


def main() -> None:
    lines = WORKFLOW.read_text(encoding="utf-8").splitlines()

    publish_order = [
        "- name: Dry-run publish biors-core",
        "- name: Publish biors-core",
        "- name: Wait for biors-core index",
        "- name: Dry-run publish biors",
        "- name: Publish biors",
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
        "actions/upload-artifact@v4",
        "actions/download-artifact@v4",
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
