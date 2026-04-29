#!/usr/bin/env python3
"""Validate release workflow invariants that affect crates.io publication."""

from pathlib import Path


WORKFLOW = Path(".github/workflows/release.yml")


def main() -> None:
    lines = WORKFLOW.read_text(encoding="utf-8").splitlines()

    required_order = [
        "- name: Dry-run publish biors-core",
        "- name: Publish biors-core",
        "- name: Wait for biors-core index",
        "- name: Dry-run publish biors",
        "- name: Publish biors",
    ]

    positions: list[tuple[str, int]] = []
    for marker in required_order:
        matching_lines = [
            line_number
            for line_number, line in enumerate(lines, start=1)
            if line.strip() == marker
        ]
        if not matching_lines:
            raise SystemExit(f"release workflow is missing step: {marker}")
        positions.append((marker, matching_lines[0]))

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
