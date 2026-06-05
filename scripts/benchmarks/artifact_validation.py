from __future__ import annotations

import json
from collections.abc import Iterable
from pathlib import Path
from typing import TypeAlias, cast

JsonScalar: TypeAlias = str | int | float | bool | None
JsonValue: TypeAlias = JsonScalar | list["JsonValue"] | dict[str, "JsonValue"]
JsonObject: TypeAlias = dict[str, JsonValue]


def load_json_object(path: Path) -> JsonObject:
    result = json.loads(path.read_text())
    if not isinstance(result, dict):
        raise AssertionError("benchmark artifact must be a JSON object")
    return cast(JsonObject, result)


def validate_schema_version(result: JsonObject, expected: str, message: str) -> None:
    if result.get("schema_version") != expected:
        raise AssertionError(message)


def require_top_level_fields(result: JsonObject, fields: Iterable[str]) -> None:
    for field in fields:
        if field not in result:
            raise AssertionError(f"missing top-level field: {field}")


def require_object(value: JsonValue, message: str) -> JsonObject:
    if not isinstance(value, dict):
        raise AssertionError(message)
    return value


def require_fields(value: JsonValue, fields: Iterable[str], label: str) -> JsonObject:
    result = require_object(value, f"{label} must be an object")
    for field in fields:
        if field not in result:
            raise AssertionError(f"{label} missing {field}")
    return result


def require_sha256(value: JsonValue, message: str) -> None:
    if not str(value).startswith("sha256:"):
        raise AssertionError(message)


def require_timed_iterations(value: JsonValue, message: str) -> None:
    if not value:
        raise AssertionError(message)


def validate_criterion_estimate(
    estimate: JsonValue,
    name: str,
    *,
    require_confidence_interval_fields: bool,
) -> None:
    estimate = require_fields(
        estimate,
        ["point_estimate", "confidence_interval", "standard_error"],
        f"{name} estimate",
    )
    if require_confidence_interval_fields:
        require_fields(
            estimate["confidence_interval"],
            ["confidence_level", "lower_bound", "upper_bound"],
            f"{name} confidence interval",
        )
