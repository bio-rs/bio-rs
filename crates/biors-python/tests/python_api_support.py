import hashlib
import json
from pathlib import Path

import jsonschema

REPO_ROOT = Path(__file__).resolve().parents[3]


def assert_matches_schema(value, schema_name):
    schema = json.loads((REPO_ROOT / "schemas" / schema_name).read_text())
    jsonschema.Draft202012Validator(schema).validate(value)


def sha256_file(path):
    return f"sha256:{hashlib.sha256(path.read_bytes()).hexdigest()}"
