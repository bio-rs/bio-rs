#!/usr/bin/env python3
"""Validate the committed benchmark JSON has basic structure."""

from __future__ import annotations

import json
from pathlib import Path

RESULT_PATH = Path("benchmarks/fasta_vs_biopython.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())

    for workload in ["parse", "validate", "tokenize"]:
        if workload not in result:
            raise AssertionError(f"missing benchmark workload: {workload}")
        data = result[workload]
        for field in ["rust_ms", "python_ms", "speedup"]:
            if field not in data:
                raise AssertionError(f"missing field {field} in {workload}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
