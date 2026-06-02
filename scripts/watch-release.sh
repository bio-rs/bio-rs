#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

version="${1:-}"
if [ -z "$version" ]; then
  version="$(cargo metadata --no-deps --format-version 1 \
    | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == "biors"))')"
fi

tag="v${version#v}"
echo "==> waiting for release workflow for $tag"

run_id=""
attempts=0
while [ -z "$run_id" ] && [ "$attempts" -lt 30 ]; do
  run_id="$(gh run list \
    --workflow release.yml \
    --branch "$tag" \
    --limit 1 \
    --json databaseId \
    --jq '.[0].databaseId // empty')"
  if [ -z "$run_id" ]; then
    attempts=$((attempts + 1))
    sleep 10
  fi
done

if [ -z "$run_id" ]; then
  echo "release workflow for $tag was not found" >&2
  exit 1
fi

python3 - "$run_id" <<'PY'
import json
import subprocess
import sys
import time

run_id = sys.argv[1]
for _ in range(180):
    output = subprocess.check_output(
        ["gh", "run", "view", run_id, "--json", "status,conclusion,jobs"],
        text=True,
    )
    data = json.loads(output)
    jobs = "; ".join(
        f"{job['name']}:{job['status']}:{job.get('conclusion') or '-'}"
        for job in data["jobs"]
    )
    print(f"run:{data['status']}:{data.get('conclusion') or '-'} | {jobs}", flush=True)
    if data["status"] == "completed":
        raise SystemExit(0 if data.get("conclusion") == "success" else 1)
    time.sleep(30)

raise SystemExit("release workflow did not complete in time")
PY

echo "==> compact release status"
python3 scripts/release-status.py

echo "==> github release"
gh release view "$tag" --json tagName,url,isDraft,isPrerelease,name
