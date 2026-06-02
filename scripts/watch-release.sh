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

gh run watch "$run_id" --exit-status

echo "==> compact release status"
python3 scripts/release-status.py

echo "==> github release"
gh release view "$tag" --json tagName,url,isDraft,isPrerelease,name
