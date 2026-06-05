#!/usr/bin/env python3
"""Check Python source syntax without writing __pycache__ artifacts."""

import sys
from pathlib import Path


DEFAULT_ROOTS = [Path("scripts"), Path("integrations/python")]


def default_files() -> list[Path]:
    files: list[Path] = []
    for root in DEFAULT_ROOTS:
        files.extend(path for path in root.rglob("*.py") if path.is_file())
    return sorted(files)


def main() -> int:
    failed = False
    files = [Path(file_name) for file_name in sys.argv[1:]] if sys.argv[1:] else default_files()
    for path in files:
        try:
            compile(path.read_text(encoding="utf-8"), str(path), "exec")
        except SyntaxError as error:
            print(f"{path}: {error}", file=sys.stderr)
            failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
