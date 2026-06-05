#!/usr/bin/env python3
"""Shared benchmark artifact freshness policy."""

from __future__ import annotations

import json
import subprocess


def validate_release_status(
    result: dict,
    *,
    environment_key: str,
    package_name: str,
) -> None:
    environment = result.get("environment")
    if not isinstance(environment, dict):
        raise AssertionError("benchmark artifact environment must be an object")
    artifact_version = environment.get(environment_key)
    if not artifact_version:
        raise AssertionError(f"benchmark artifact environment missing {environment_key}")

    current_version = cargo_package_version(package_name)
    release_status = result.get("release_status")
    if not isinstance(release_status, dict):
        raise AssertionError("benchmark artifact must include release_status")

    current_commit = command_output(["git", "rev-parse", "HEAD"])
    artifact_commit = environment.get("git_commit")
    if artifact_version == current_version and artifact_commit == current_commit:
        if release_status.get("status") != "current":
            raise AssertionError(
                "current benchmark artifacts must use release_status.status=current"
            )
        return

    if release_status.get("status") not in {"regression_guard", "historical"}:
        raise AssertionError(
            "stale benchmark artifacts must use release_status.status=regression_guard or historical"
        )
    if release_status.get("recorded_package_version") != artifact_version:
        raise AssertionError("release_status.recorded_package_version must match environment")
    if release_status.get("current_workspace_version") != current_version:
        raise AssertionError("release_status.current_workspace_version must match workspace")
    if not release_status.get("stale_reason"):
        raise AssertionError("stale benchmark artifacts must explain stale_reason")
    if "not current-version performance evidence" not in release_status.get("claim_policy", ""):
        raise AssertionError(
            "stale benchmark artifacts must define a non-current-version claim_policy"
        )


def cargo_package_version(package_name: str) -> str:
    output = command_output(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"])
    if output is None:
        raise AssertionError("cargo metadata is required for benchmark freshness checks")
    for package in json.loads(output).get("packages", []):
        if package.get("name") == package_name:
            return str(package.get("version"))
    raise AssertionError(f"cargo metadata missing package {package_name}")


def command_output(command: list[str]) -> str | None:
    try:
        completed = subprocess.run(
            command,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    return completed.stdout.strip()
