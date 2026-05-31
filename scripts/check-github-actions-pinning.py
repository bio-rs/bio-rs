#!/usr/bin/env python3
"""Require GitHub Actions workflow actions to be pinned to immutable SHAs."""

from pathlib import Path
import re
import sys


WORKFLOWS = sorted(Path(".github/workflows").glob("*.yml"))
PINNED_REF = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+@[0-9a-f]{40}$")


def main() -> int:
    errors: list[str] = []
    for workflow in WORKFLOWS:
        for line_number, line in enumerate(workflow.read_text(encoding="utf-8").splitlines(), 1):
            match = re.search(r"\buses:\s*([^#\s]+)", line)
            if not match:
                continue
            action = match.group(1)
            if action.startswith("./"):
                continue
            if not PINNED_REF.fullmatch(action):
                errors.append(f"{workflow}:{line_number}: action must be pinned to a 40-char SHA: {action}")

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
