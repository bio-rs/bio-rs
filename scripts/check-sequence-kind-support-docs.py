#!/usr/bin/env python3
"""Validate sequence-kind support docs against the machine-readable fixture."""

from __future__ import annotations

import json
from pathlib import Path

DOC_PATH = Path("docs/sequence-kind-support.md")
FIXTURE_PATH = Path("fixtures/support/sequence-kind-support.json")


def main() -> int:
    fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
    if fixture.get("schema_version") != "biors.sequence_kind_support.v1":
        raise AssertionError("sequence-kind support fixture must use schema v1")

    docs_rows = parse_markdown_table(DOC_PATH.read_text(encoding="utf-8"))
    for row in fixture.get("surfaces", []):
        surface = row["surface"]
        if surface not in docs_rows:
            raise AssertionError(f"missing sequence-kind support row for {surface!r}")
        docs_row = docs_rows[surface]
        for key, column in [
            ("protein", "Protein"),
            ("dna", "DNA"),
            ("rna", "RNA"),
        ]:
            expected = row[key]
            actual = docs_row[column]
            if actual != expected:
                raise AssertionError(
                    f"{surface!r} {column} support mismatch: "
                    f"expected {expected!r}, found {actual!r}"
                )
    return 0


def parse_markdown_table(text: str) -> dict[str, dict[str, str]]:
    rows: dict[str, dict[str, str]] = {}
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != 5 or cells[0] in {"Surface", "---"}:
            continue
        rows[cells[0]] = {
            "Protein": cells[1],
            "DNA": cells[2],
            "RNA": cells[3],
            "Notes": cells[4],
        }
    return rows


if __name__ == "__main__":
    raise SystemExit(main())
