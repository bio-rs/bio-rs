#!/usr/bin/env python3
"""Fixture process for biors-core runtime integration tests."""

from __future__ import annotations

import json
import os
import sys
import time


def main() -> int:
    mode = sys.argv[1]

    if mode == "sleep":
        time.sleep(1.0)
        return 0

    if mode == "early-fail":
        print("process failed before consuming stdin", file=sys.stderr)
        return 9

    context = json.load(sys.stdin)

    if mode == "fail":
        print("private stderr ACDEFG should stay out of error messages", file=sys.stderr)
        return 7

    if mode == "invalid-output":
        print("not-json")
        return 0

    if mode == "big-stdout":
        print("X" * 4096)
        return 0

    if mode not in {"echo", "wrong-output"}:
        print(f"unsupported fixture mode: {mode}", file=sys.stderr)
        return 64

    metadata = list(context.get("metadata", []))
    metadata.append({"key": "external_process_fixture", "value": "echo"})
    metadata.append(
        {
            "key": "explicit_env_visible",
            "value": os.environ.get("BIORS_RUNTIME_ALLOWED", "no"),
        }
    )
    metadata.append(
        {
            "key": "parent_secret_visible",
            "value": os.environ.get("BIORS_RUNTIME_PARENT_SECRET", "no"),
        }
    )

    json.dump(
        {
            "trace_id": context.get("trace_id"),
            "output_format": (
                "biors.alternate.v0"
                if mode == "wrong-output"
                else context.get("requested_output_format") or "biors.echo.v0"
            ),
            "payload": context["payload"],
            "metadata": metadata,
        },
        sys.stdout,
    )
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
