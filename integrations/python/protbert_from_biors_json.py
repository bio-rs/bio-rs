#!/usr/bin/env python3
"""Adapt bio-rs tokenization JSON for ProtBERT-style preprocessing.

ProtBERT examples commonly consume whitespace-separated amino acid strings
before a model-specific tokenizer runs. This helper reads `biors debug` JSON so
the sequence normalization and residue diagnostics remain controlled by bio-rs.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


def load_biors_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text())
    if not payload.get("ok"):
        raise ValueError("expected a successful bio-rs JSON envelope")
    return payload["data"]


def to_protbert_sequences(data: dict[str, Any]) -> list[dict[str, str]]:
    if data.get("view") != "sequence_debug.v0":
        raise ValueError("expected `biors debug` output")
    rows = []
    for record in data["records"]:
        token_map = record["token_map"]
        if any(step["status"] == "error" for step in token_map):
            raise ValueError(f"record {record['id']} has invalid residues")
        rows.append(
            {
                "id": record["id"],
                "protbert_sequence": " ".join(step["residue"] for step in token_map),
            }
        )
    return rows


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: protbert_from_biors_json.py <biors-debug.json>")
    rows = to_protbert_sequences(load_biors_json(Path(sys.argv[1])))
    print(json.dumps(rows, indent=2))


if __name__ == "__main__":
    main()
