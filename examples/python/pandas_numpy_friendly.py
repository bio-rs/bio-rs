#!/usr/bin/env python3
"""Convert bio-rs JSON into table- and array-friendly Python data.

The returned dictionaries can be passed directly to pandas.DataFrame, and the
nested integer lists can be passed to numpy.asarray by downstream code. This
file avoids importing pandas or NumPy so it stays usable in minimal examples.
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


def table_rows(data: dict[str, Any]) -> list[dict[str, Any]]:
    workflow = data.get("workflow", data)
    model_input = workflow.get("model_input")
    if model_input is None:
        raise ValueError("bio-rs payload is not model-ready")

    rows = []
    for record in model_input["records"]:
        rows.append(
            {
                "id": record["id"],
                "input_ids": list(record["input_ids"]),
                "attention_mask": list(record["attention_mask"]),
                "truncated": bool(record["truncated"]),
            }
        )
    return rows


def array_columns(rows: list[dict[str, Any]]) -> dict[str, list[Any]]:
    return {
        "ids": [row["id"] for row in rows],
        "input_ids": [row["input_ids"] for row in rows],
        "attention_mask": [row["attention_mask"] for row in rows],
        "truncated": [row["truncated"] for row in rows],
    }


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: pandas_numpy_friendly.py <biors-output.json>")
    rows = table_rows(load_biors_json(Path(sys.argv[1])))
    print(json.dumps({"rows": rows, "columns": array_columns(rows)}, indent=2))


if __name__ == "__main__":
    main()
