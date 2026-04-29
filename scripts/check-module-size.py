#!/usr/bin/env python3
"""Keep Rust modules small enough to preserve single-responsibility boundaries."""

from __future__ import annotations

from pathlib import Path

MAX_RUST_MODULE_LINES = 320
CHECKED_ROOTS = (
    Path("packages/rust/biors-core/src"),
    Path("packages/rust/biors/src"),
)


def main() -> int:
    oversized: list[tuple[Path, int]] = []
    for root in CHECKED_ROOTS:
        for path in sorted(root.rglob("*.rs")):
            line_count = len(path.read_text(encoding="utf-8").splitlines())
            if line_count > MAX_RUST_MODULE_LINES:
                oversized.append((path, line_count))

    if oversized:
        print(
            f"Rust modules must stay at or below {MAX_RUST_MODULE_LINES} lines "
            "to keep responsibilities narrow."
        )
        for path, line_count in oversized:
            print(f"- {path}: {line_count} lines")
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
