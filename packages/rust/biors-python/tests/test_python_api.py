import json
from pathlib import Path

import biors
import jsonschema
import pytest

REPO_ROOT = Path(__file__).resolve().parents[4]

def test_parse_fasta_records():
    fasta = ">seq1\nACDEFG\n>seq2\nMKWVT"
    records = biors.parse_fasta_records(fasta)
    assert len(records) == 2
    assert records[0].id == "seq1"
    assert records[0].sequence == "ACDEFG"

def test_validate_fasta_input():
    fasta = ">seq1\nACDEFG"
    report = biors.validate_fasta_input(fasta)
    assert report.records == 1
    assert report.valid_records == 1
    assert report.error_count == 0

def test_tokenize_fasta_records():
    fasta = ">seq1\nACDEFG"
    tokenized = biors.tokenize_fasta_records(fasta)
    assert len(tokenized) == 1
    assert tokenized[0].id == "seq1"
    assert tokenized[0].alphabet == "protein-20"
    assert tokenized[0].valid == True
    assert len(tokenized[0].tokens) == 6
    assert tokenized[0].warnings == []
    assert tokenized[0].errors == []

def test_tokenize_fasta_records_exposes_residue_diagnostics():
    fasta = ">seq1\nAX*\n"
    tokenized = biors.tokenize_fasta_records(fasta)
    assert tokenized[0].valid == False
    assert tokenized[0].warnings[0].residue == "X"
    assert tokenized[0].warnings[0].position == 2
    assert tokenized[0].errors[0].residue == "*"
    assert tokenized[0].errors[0].position == 3

def test_tokenize_protein_normalizes_like_fasta_tokenization():
    direct = biors.tokenize_protein("ac de\tfg")
    from_fasta = biors.tokenize_fasta_records(">seq1\nac de\tfg\n")[0]
    assert direct.valid == True
    assert direct.tokens == from_fasta.tokens
    assert direct.length == from_fasta.length
    assert direct.warnings == []
    assert direct.errors == []

def test_checked_model_input_rejects_non_model_ready_tokenization():
    fasta = ">seq1\nAX*\n"
    tokenized = biors.tokenize_fasta_records(fasta)
    try:
        biors.build_model_inputs_checked(tokenized, max_length=10)
    except ValueError as exc:
        assert "not model-ready" in str(exc)
        assert "1 warnings and 1 errors" in str(exc)
    else:
        raise AssertionError("expected non-model-ready tokenization to be rejected")

def test_build_model_inputs():
    fasta = ">seq1\nACDEFG"
    tokenized = biors.tokenize_fasta_records(fasta)
    model_input = biors.build_model_inputs_checked(tokenized, max_length=10)
    assert len(model_input.records) == 1
    assert len(model_input.records[0].input_ids) == 6
    assert model_input.records[0].truncated == False

def test_build_model_inputs_fixed_length_padding():
    fasta = ">seq1\nACDEFG"
    tokenized = biors.tokenize_fasta_records(fasta)
    model_input = biors.build_model_inputs_checked(
        tokenized,
        max_length=10,
        pad_token_id=21,
        padding="fixed_length",
    )
    assert model_input.records[0].input_ids == [0, 1, 2, 3, 4, 5, 21, 21, 21, 21]
    assert model_input.records[0].attention_mask == [1, 1, 1, 1, 1, 1, 0, 0, 0, 0]

def test_prepare_workflow():
    fasta = ">seq1\nACDEFG"
    records = biors.parse_fasta_records(fasta)
    output = biors.prepare_workflow("hash123", records, max_length=10, padding="fixed_length")
    assert output.model_ready == True
    assert output.input_hash == "hash123"
    assert len(output.records) == 1
    assert len(output.records[0].input_ids) == 10

def test_prepare_workflow_marks_direct_empty_sequence_not_model_ready():
    records = [biors.ProteinSequence("empty", "")]
    output = biors.prepare_workflow("hash-empty", records, max_length=10, padding="fixed_length")
    assert output.model_ready == False
    assert output.records == []

def test_prepare_workflow_from_fasta_computes_input_hash_internally():
    fasta = ">seq1\nACDEFG"
    output = biors.prepare_workflow_from_fasta(
        fasta,
        max_length=10,
        padding="fixed_length",
    )
    assert output.model_ready == True
    assert output.input_hash.startswith("fnv1a64:")
    assert output.input_hash == biors.prepare_workflow_from_fasta(
        fasta,
        max_length=10,
        padding="fixed_length",
    ).input_hash
    assert len(output.records) == 1
    assert len(output.records[0].input_ids) == 10

def test_package_manifest_inspection_is_exported():
    manifest_json = """
    {
      "schema_version": "biors.package.v0",
      "name": "protein-seed",
      "model": {
        "format": "onnx",
        "path": "models/protein-seed.onnx"
      },
      "preprocessing": [
        {
          "name": "protein_fasta_tokenize",
          "implementation": "biors-core",
          "contract": "protein-20"
        }
      ],
      "postprocessing": [],
      "runtime": {
        "backend": "onnx-webgpu",
        "target": "browser-wasm-webgpu"
      },
      "fixtures": [
        {
          "name": "tiny-protein",
          "input": "fixtures/tiny.fasta",
          "expected_output": "fixtures/tiny.output.json"
        }
      ]
    }
    """
    assert "inspect_package_manifest" in biors.__all__
    summary = json.loads(biors.inspect_package_manifest(manifest_json))
    assert_matches_schema(summary, "package-inspect-output.v0.json")
    assert summary["name"] == "protein-seed"
    assert summary["model_format"] == "onnx"
    assert summary["runtime_backend"] == "onnx-webgpu"
    assert summary["preprocessing_steps"] == 1

def test_package_json_helpers_match_shared_schemas():
    manifest_json = (REPO_ROOT / "examples/protein-package/manifest.json").read_text()
    validation = json.loads(biors.validate_package_manifest(manifest_json))
    bridge = json.loads(biors.plan_runtime_bridge(manifest_json))

    assert validation["valid"] == True
    assert bridge["ready"] == True
    assert_matches_schema(validation, "package-validation-report.v0.json")
    assert_matches_schema(bridge, "package-bridge-output.v0.json")

def test_package_json_helpers_reject_unknown_manifest_fields():
    manifest = json.loads((REPO_ROOT / "examples/protein-package/manifest.json").read_text())
    manifest["unexpected_top"] = True

    with pytest.raises(ValueError, match="unknown field"):
        biors.validate_package_manifest(json.dumps(manifest))

def assert_matches_schema(value, schema_name):
    schema = json.loads((REPO_ROOT / "schemas" / schema_name).read_text())
    jsonschema.Draft202012Validator(schema).validate(value)
