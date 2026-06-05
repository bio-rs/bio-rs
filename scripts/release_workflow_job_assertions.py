from __future__ import annotations

from typing import Any


class StepCheck:
    def __init__(
        self,
        *,
        name: str | None = None,
        uses: str | None = None,
        run_contains: list[str] | None = None,
        with_values: dict[str, Any] | None = None,
    ) -> None:
        self.name = name
        self.uses = uses
        self.run_contains = run_contains or []
        self.with_values = with_values or {}

    def matches(self, step: dict[str, Any]) -> bool:
        if self.name is not None and step.get("name") != self.name:
            return False
        if self.uses is not None and step.get("uses") != self.uses:
            return False
        run = step.get("run", "")
        if any(text not in run for text in self.run_contains):
            return False
        with_block = step.get("with", {})
        return all(with_block.get(key) == value for key, value in self.with_values.items())

    def describe(self) -> str:
        parts = []
        if self.name:
            parts.append(f"name={self.name!r}")
        if self.uses:
            parts.append(f"uses={self.uses!r}")
        if self.run_contains:
            parts.append(f"run contains {self.run_contains!r}")
        if self.with_values:
            parts.append(f"with {self.with_values!r}")
        return ", ".join(parts)


def assert_job(
    jobs: dict[str, Any],
    job_name: str,
    *,
    needs: list[str],
    permissions: dict[str, str] | None,
    steps: list[StepCheck],
    tag_only: bool = False,
    env: dict[str, str] | None = None,
    matrix: dict[str, Any] | None = None,
) -> None:
    job = require_mapping(jobs, job_name)
    if normalize_needs(job.get("needs")) != needs:
        raise SystemExit(f"release job {job_name} has unsafe needs: {job.get('needs')!r}")
    if tag_only and job.get("if") != "startsWith(github.ref, 'refs/tags/v')":
        raise SystemExit(f"release job {job_name} must only run on v* tags")
    if permissions is not None and job.get("permissions") != permissions:
        raise SystemExit(
            f"release job {job_name} permissions must be {permissions!r}, "
            f"got {job.get('permissions')!r}"
        )
    if env is not None and job.get("env") != env:
        raise SystemExit(f"release job {job_name} env must be {env!r}")
    if matrix is not None:
        assert_matrix(job_name, job, matrix)
    assert_steps(job_name, require_list(job, "steps"), steps)


def assert_matrix(job_name: str, job: dict[str, Any], expected: dict[str, Any]) -> None:
    strategy = require_mapping(job, "strategy")
    matrix = require_mapping(strategy, "matrix")
    for key, value in expected.items():
        if key == "include":
            actual = matrix.get("include")
            if not isinstance(actual, list):
                raise SystemExit(f"release job {job_name} matrix.include must be a list")
            for expected_entry in value:
                if not any(
                    all(entry.get(k) == v for k, v in expected_entry.items())
                    for entry in actual
                    if isinstance(entry, dict)
                ):
                    raise SystemExit(
                        f"release job {job_name} matrix.include missing {expected_entry!r}"
                    )
        elif matrix.get(key) != value:
            raise SystemExit(
                f"release job {job_name} matrix.{key} must be {value!r}, got {matrix.get(key)!r}"
            )


def assert_steps(
    job_name: str, steps: list[Any], expected_steps: list[StepCheck]
) -> None:
    typed_steps = [step for step in steps if isinstance(step, dict)]
    search_from = 0
    for expected in expected_steps:
        for index in range(search_from, len(typed_steps)):
            if expected.matches(typed_steps[index]):
                search_from = index + 1
                break
        else:
            raise SystemExit(
                f"release job {job_name} is missing ordered step: {expected.describe()}"
            )


def normalize_needs(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    if isinstance(value, list) and all(isinstance(item, str) for item in value):
        return value
    raise SystemExit(f"job needs must be a string or string list, got {value!r}")


def require_mapping(mapping: dict[str, Any], key: str) -> dict[str, Any]:
    value = mapping.get(key)
    if not isinstance(value, dict):
        raise SystemExit(f"release workflow key {key} must be a mapping")
    return value


def require_list(mapping: dict[str, Any], key: str) -> list[Any]:
    value = mapping.get(key)
    if not isinstance(value, list):
        raise SystemExit(f"release workflow key {key} must be a list")
    return value
