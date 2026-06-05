#!/usr/bin/env python3
"""Adapt bio-rs model-input JSON for ESM-style Python workflows.

This example intentionally has no third-party dependency. It validates the
shape emitted by `biors pipeline` or `biors workflow` and returns plain Python
lists that can be converted to torch tensors by downstream ESM code.
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


def model_input_records(data: dict[str, Any]) -> list[dict[str, Any]]:
    if "workflow" in data and "workflow" in data["workflow"]:
        model_input = data["workflow"].get("model_input")
    else:
        model_input = data.get("model_input")
    if model_input is None:
        raise ValueError("bio-rs payload is not model-ready")
    return list(model_input["records"])


def to_esm_batch(data: dict[str, Any]) -> dict[str, list[list[int]]]:
    records = model_input_records(data)
    return {
        "input_ids": [list(record["input_ids"]) for record in records],
        "attention_mask": [list(record["attention_mask"]) for record in records],
    }


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: esm_from_biors_json.py <biors-output.json>")
    batch = to_esm_batch(load_biors_json(Path(sys.argv[1])))
    print(json.dumps(batch, indent=2))


if __name__ == "__main__":
    main()
