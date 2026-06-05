#!/usr/bin/env python3
"""Reference preprocessing fixture for the protein-20-special contract."""

from __future__ import annotations

import json

VOCAB = {residue: index for index, residue in enumerate("ACDEFGHIKLMNPQRSTVWY")}
SPECIAL = {"unk": 20, "pad": 21, "cls": 22, "sep": 23, "mask": 24}


def tokenize(sequence: str) -> dict[str, object]:
    normalized = "".join(sequence.split()).upper()
    return {
        "id": "seq1",
        "length": len(normalized) + 2,
        "alphabet": "protein-20-special",
        "valid": True,
        "tokens": [SPECIAL["cls"], *[VOCAB[symbol] for symbol in normalized], SPECIAL["sep"]],
        "warnings": [],
        "errors": [],
    }


if __name__ == "__main__":
    print(json.dumps(tokenize("acde"), indent=2))
