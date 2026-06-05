#!/usr/bin/env python3
"""Validate release workflow invariants that affect package publication."""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path
from typing import Any

from release.workflow_job_assertions import require_mapping
from release.workflow_jobs import assert_release_jobs
from release.workflow_text_markers import (
    assert_release_tool_scripts_use_pins,
    assert_secondary_text_markers,
    read_release_tool_versions,
)

WORKFLOW = Path(".github/workflows/release.yml")
UNUSED_RELEASE_TEMPLATE = Path(".github/release_template.md")
PINNED_ACTION = re.compile(r"^[^@]+@[0-9a-f]{40}$")


def main() -> None:
    if UNUSED_RELEASE_TEMPLATE.exists():
        raise SystemExit(
            ".github/release_template.md is not used by the release workflow; "
            "keep GitHub --generate-notes as the release body source of truth"
        )

    tool_versions = read_release_tool_versions()
    workflow = load_workflow_yaml(WORKFLOW)
    workflow_text = WORKFLOW.read_text(encoding="utf-8")

    assert_release_triggers(workflow)
    assert_release_env(workflow, tool_versions)
    assert_release_jobs(workflow)
    assert_action_refs_are_pinned(workflow)
    assert_release_tool_scripts_use_pins(tool_versions)
    assert_secondary_text_markers(workflow_text, tool_versions)


def load_workflow_yaml(path: Path) -> dict[str, Any]:
    ruby = (
        "require 'yaml'; require 'json'; "
        "puts JSON.generate(YAML.safe_load(File.read(ARGV[0]), aliases: true))"
    )
    try:
        output = subprocess.check_output(
            ["ruby", "-e", ruby, str(path)],
            text=True,
            stderr=subprocess.PIPE,
        )
    except FileNotFoundError as exc:
        raise SystemExit("Ruby is required to parse release.yml with YAML.safe_load") from exc
    except subprocess.CalledProcessError as exc:
        raise SystemExit(f"failed to parse {path} as YAML: {exc.stderr}") from exc

    workflow = json.loads(output)
    if not isinstance(workflow, dict):
        raise SystemExit(f"{path} must parse to a YAML mapping")
    return workflow


def assert_release_env(
    workflow: dict[str, Any], tool_versions: dict[str, str]
) -> None:
    assert_mapping_value(workflow, ["permissions", "contents"], "read")
    env = require_mapping(workflow, "env")
    for key, expected in tool_versions.items():
        actual = env.get(key)
        if actual != expected:
            raise SystemExit(
                f"release workflow env {key} must match scripts/release-tool-versions.env: "
                f"expected {expected!r}, got {actual!r}"
            )


def assert_release_triggers(workflow: dict[str, Any]) -> None:
    # Ruby's YAML parser follows YAML 1.1 and reads the GitHub Actions `on` key
    # as boolean true. Accept both shapes while still validating the semantics.
    triggers = workflow.get("on", workflow.get("true"))
    if not isinstance(triggers, dict):
        raise SystemExit("release workflow must define workflow_dispatch and push triggers")
    if "workflow_dispatch" not in triggers:
        raise SystemExit("release workflow must keep workflow_dispatch enabled")
    push = triggers.get("push")
    if not isinstance(push, dict):
        raise SystemExit("release workflow must define push trigger details")
    if push.get("branches") != ["main"]:
        raise SystemExit("release workflow push trigger must be limited to main branch")
    if push.get("tags") != ["v*"]:
        raise SystemExit("release workflow push trigger must be limited to v* tags")


def assert_action_refs_are_pinned(workflow: dict[str, Any]) -> None:
    jobs = require_mapping(workflow, "jobs")
    unpinned = []
    for job_name, job in jobs.items():
        if not isinstance(job, dict):
            continue
        for step in job.get("steps", []):
            if not isinstance(step, dict) or "uses" not in step:
                continue
            action = step["uses"]
            if not isinstance(action, str) or not PINNED_ACTION.match(action):
                unpinned.append(f"{job_name}: {action!r}")
    if unpinned:
        raise SystemExit(
            "release workflow action refs must be pinned to 40-character SHAs: "
            + ", ".join(unpinned)
        )


def assert_mapping_value(
    mapping: dict[str, Any], path: list[str], expected: Any
) -> None:
    current: Any = mapping
    for key in path:
        if not isinstance(current, dict) or key not in current:
            raise SystemExit(f"release workflow is missing {'.'.join(path)}")
        current = current[key]
    if current != expected:
        raise SystemExit(
            f"release workflow {'.'.join(path)} must be {expected!r}, got {current!r}"
        )


if __name__ == "__main__":
    main()
