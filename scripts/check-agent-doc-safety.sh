#!/bin/sh
set -eu

if [ "$#" -eq 0 ]; then
  echo "usage: $0 <doc>..." >&2
  exit 2
fi

status=0
for file in "$@"; do
  [ -f "$file" ] || {
    echo "error: missing file: $file" >&2
    status=1
    continue
  }
  awk '
    {
      line = tolower($0)
      banned = line ~ /(autonomous research planning|literature review|remote lab automation|hosted execution service|hosted execution|cloud model calls|cloud model|long-term memory|memory layer|telemetry)/
      allowed = line ~ /(not |non-goal|non-goals|unsupported|out-of-scope|must not|does not|no )/
      if (banned && !allowed) {
        print FILENAME ":" FNR ": prohibited agent/hosted claim: " $0 > "/dev/stderr"
        bad = 1
      }
    }
    END { exit bad }
  ' "$file" || status=1
done

exit "$status"
