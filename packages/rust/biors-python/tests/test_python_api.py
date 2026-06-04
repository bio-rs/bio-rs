import json
import shutil
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

def test_python_errors_expose_stable_code_and_location_for_empty_fasta():
    with pytest.raises(biors.BioRsError) as exc_info:
        biors.parse_fasta_records("")

    assert exc_info.value.code == "fasta.empty_input"
    assert exc_info.value.message == "FASTA input is empty"
    assert exc_info.value.location is None

def test_python_errors_expose_stable_code_and_location_for_invalid_fasta():
    with pytest.raises(biors.BioRsError) as exc_info:
        biors.parse_fasta_records("ACDE")

    assert exc_info.value.code == "fasta.missing_header"
    assert exc_info.value.location == {"line": 1, "record_index": None}

def test_validate_fasta_input():
    fasta = ">seq1\nACDEFG"
    report = biors.validate_fasta_input(fasta)
    assert report.records == 1
    assert report.valid_records == 1
    assert report.error_count == 0
    assert len(report.sequences) == 1
    assert report.sequences[0].id == "seq1"
    assert report.sequences[0].sequence == "ACDEFG"
    assert report.sequences[0].valid == True

def test_validate_fasta_input_exposes_sequence_diagnostics():
    fasta = ">seq1\nAC*X\n"
    report = biors.validate_fasta_input(fasta)
    assert report.records == 1
    assert report.valid_records == 0
    assert report.warning_count == 1
    assert report.error_count == 1

    sequence = report.sequences[0]
    assert sequence.id == "seq1"
    assert sequence.sequence == "AC*X"
    assert sequence.alphabet == "protein-20"
    assert sequence.valid == False
    assert [(issue.residue, issue.position) for issue in sequence.warnings] == [("X", 4)]
    assert [(issue.residue, issue.position) for issue in sequence.errors] == [("*", 3)]

def test_validate_fasta_input_with_kind_accepts_nucleotides():
    assert "validate_fasta_input_with_kind" in biors.__all__

    dna = biors.validate_fasta_input_with_kind(">dna\nACGTN\n", "dna")
    rna = biors.validate_fasta_input_with_kind(">rna\nACGUN\n", "rna")

    assert dna.records == 1
    assert dna.error_count == 0
    assert dna.warning_count == 1
    assert dna.sequences[0].alphabet == "dna-iupac"
    assert [(issue.residue, issue.position) for issue in dna.sequences[0].warnings] == [("N", 5)]
    assert rna.records == 1
    assert rna.error_count == 0
    assert rna.warning_count == 1
    assert rna.sequences[0].alphabet == "rna-iupac"

def test_validate_fasta_input_with_kind_rejects_unknown_kind():
    with pytest.raises(biors.BioRsError) as exc_info:
        biors.validate_fasta_input_with_kind(">seq\nACDE\n", "bad")

    assert exc_info.value.code == "sequence.invalid_kind"

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

def test_tokenize_fasta_records_accepts_nucleotide_profiles():
    dna = biors.tokenize_fasta_records(">dna\nACGTN\n", profile="dna-iupac")[0]
    rna = biors.tokenize_fasta_records(">rna\nACGUN\n", profile="rna-iupac")[0]

    assert dna.alphabet == "dna-iupac"
    assert dna.tokens == [0, 1, 2, 3, 4]
    assert dna.valid == False
    assert dna.warnings[0].residue == "N"
    assert rna.alphabet == "rna-iupac"
    assert rna.tokens == [0, 1, 2, 3, 4]

def test_tokenize_protein_normalizes_like_fasta_tokenization():
    direct = biors.tokenize_protein("ac de\tfg")
    from_fasta = biors.tokenize_fasta_records(">seq1\nac de\tfg\n")[0]
    assert direct.id == "user"
    assert direct.valid == True
    assert direct.tokens == from_fasta.tokens
    assert direct.length == from_fasta.length
    assert direct.warnings == []
    assert direct.errors == []

def test_tokenize_protein_preserves_caller_provided_id():
    tokenized = biors.tokenize_protein("ACDE", id="sample-42")
    assert tokenized.id == "sample-42"
    assert tokenized.tokens == [0, 1, 2, 3]

def test_tokenize_protein_accepts_nucleotide_profile_for_direct_sequences():
    tokenized = biors.tokenize_protein("acgt", id="dna-1", profile="dna-iupac")
    assert tokenized.id == "dna-1"
    assert tokenized.alphabet == "dna-iupac"
    assert tokenized.tokens == [0, 1, 2, 3]

def test_tokenize_rejects_unknown_profile_with_stable_error_code():
    with pytest.raises(biors.BioRsError) as exc_info:
        biors.tokenize_fasta_records(">seq\nACGT\n", profile="bad")

    assert exc_info.value.code == "tokenizer.invalid_profile"

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

def test_python_errors_expose_stable_code_for_invalid_padding():
    with pytest.raises(biors.BioRsError) as exc_info:
        biors.build_model_inputs_checked([], max_length=10, padding="left")

    assert exc_info.value.code == "model_input.invalid_policy"
    assert exc_info.value.location is None

def test_python_errors_expose_stable_code_for_invalid_model_input_policy():
    tokenized = [biors.TokenizedProtein("seq1", [0, 1])]

    with pytest.raises(biors.BioRsError) as exc_info:
        biors.build_model_inputs_checked(tokenized, max_length=0)

    assert exc_info.value.code == "model_input.invalid_policy"

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

def test_direct_tokenized_protein_construction_builds_model_input():
    tokenized = [biors.TokenizedProtein("seq-memory", [0, 1, 2, 3])]
    model_input = biors.build_model_inputs_checked(
        tokenized,
        max_length=6,
        pad_token_id=21,
        padding="fixed_length",
    )

    assert model_input.records[0].id == "seq-memory"
    assert model_input.records[0].input_ids == [0, 1, 2, 3, 21, 21]
    assert model_input.records[0].attention_mask == [1, 1, 1, 1, 0, 0]

def test_direct_tokenized_protein_preserves_diagnostics():
    warning = biors.ResidueIssue("X", 2)
    tokenized = [
        biors.TokenizedProtein(
            "seq-invalid",
            [0, 20],
            valid=False,
            warnings=[warning],
        )
    ]

    with pytest.raises(ValueError, match="not model-ready"):
        biors.build_model_inputs_checked(tokenized, max_length=6)

def test_residue_issue_constructor_rejects_multi_character_residue():
    with pytest.raises(ValueError, match="exactly one residue"):
        biors.ResidueIssue("XX", 1)

def test_prepare_workflow():
    fasta = ">seq1\nACDEFG"
    records = biors.parse_fasta_records(fasta)
    output = biors.prepare_workflow(
        "fnv1a64:0000000000000001",
        records,
        max_length=10,
        padding="fixed_length",
    )
    assert output.model_ready == True
    assert output.input_hash == "fnv1a64:0000000000000001"
    assert len(output.records) == 1
    assert len(output.records[0].input_ids) == 10
    report = json.loads(output.report_json)
    assert report["model_ready"] == True
    assert report["provenance"]["input_hash"] == "fnv1a64:0000000000000001"
    assert report["readiness_issues"] == []

def test_prepare_workflow_rejects_invalid_input_hash():
    records = [biors.ProteinSequence("seq-memory", "ACDE")]
    with pytest.raises(biors.BioRsError) as exc_info:
        biors.prepare_workflow("hash-memory", records, max_length=6, padding="fixed_length")
    assert exc_info.value.code == "workflow.invalid_input_hash"

def test_prepare_workflow_marks_direct_empty_sequence_not_model_ready():
    records = [biors.ProteinSequence("empty", "")]
    output = biors.prepare_workflow(
        "fnv1a64:0000000000000002",
        records,
        max_length=10,
        padding="fixed_length",
    )
    assert output.model_ready == False
    assert output.records == []
    report = json.loads(output.report_json)
    assert report["model_ready"] == False
    assert report["validation"]["records"] == 1
    assert report["readiness_issues"][0]["code"] == "sequence.not_model_ready"
    assert report["readiness_issues"][0]["id"] == "empty"
    assert report["provenance"]["input_hash"] == "fnv1a64:0000000000000002"

def test_prepare_workflow_report_json_exposes_invalid_sequence_diagnostics():
    records = [biors.ProteinSequence("bad", "AX*")]
    output = biors.prepare_workflow(
        "fnv1a64:0000000000000003",
        records,
        max_length=10,
        padding="fixed_length",
    )
    report = json.loads(output.report_json)

    assert output.model_ready == False
    assert output.records == []
    assert report["validation"]["warning_count"] == 1
    assert report["validation"]["error_count"] == 1
    assert report["tokenization"]["summary"]["warning_count"] == 1
    assert report["tokenization"]["summary"]["error_count"] == 1
    assert report["readiness_issues"][0]["code"] == "sequence.not_model_ready"
    assert report["readiness_issues"][0]["id"] == "bad"

def test_direct_protein_sequence_construction_prepares_workflow():
    records = [biors.ProteinSequence("seq-memory", "ACDE")]
    output = biors.prepare_workflow(
        "fnv1a64:0000000000000004",
        records,
        max_length=6,
        padding="fixed_length",
    )

    assert output.model_ready == True
    assert output.input_hash == "fnv1a64:0000000000000004"
    assert output.records[0].id == "seq-memory"
    assert output.records[0].attention_mask == [1, 1, 1, 1, 0, 0]

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

def test_prepare_workflow_from_fasta_accepts_nucleotide_profiles():
    output = biors.prepare_workflow_from_fasta(
        ">dna\nACGT\n",
        max_length=6,
        padding="fixed_length",
        profile="dna-iupac",
    )
    report = json.loads(output.report_json)

    assert output.model_ready == True
    assert output.records[0].input_ids == [0, 1, 2, 3, 0, 0]
    assert report["workflow"] == "sequence_model_input.v0"
    assert report["provenance"]["validation_alphabet"] == "dna-iupac"
    assert report["provenance"]["tokenizer"]["name"] == "dna-iupac"
    assert report["tokenization"]["records"][0]["alphabet"] == "dna-iupac"

def test_prepare_workflow_accepts_nucleotide_profiles_for_direct_records():
    records = [biors.ProteinSequence("rna-memory", "ACGU")]
    output = biors.prepare_workflow(
        "fnv1a64:0000000000000005",
        records,
        max_length=4,
        profile="rna-iupac",
    )
    report = json.loads(output.report_json)

    assert output.model_ready == True
    assert output.records[0].input_ids == [0, 1, 2, 3]
    assert report["provenance"]["validation_alphabet"] == "rna-iupac"
    assert report["provenance"]["tokenizer"]["name"] == "rna-iupac"

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
    assert "validate_package_manifest_artifacts" in biors.__all__
    assert "validate_package_manifest_file" in biors.__all__
    validation = json.loads(biors.validate_package_manifest(manifest_json))
    artifact_validation = json.loads(
        biors.validate_package_manifest_artifacts(
            manifest_json, str(REPO_ROOT / "examples/protein-package")
        )
    )
    file_validation = json.loads(
        biors.validate_package_manifest_file(
            str(REPO_ROOT / "examples/protein-package/manifest.json")
        )
    )
    bridge = json.loads(biors.plan_runtime_bridge(manifest_json))

    assert validation["valid"] == True
    assert artifact_validation["valid"] == True
    assert file_validation["valid"] == True
    assert bridge["ready"] == True
    assert_matches_schema(validation, "package-validation-report.v0.json")
    assert_matches_schema(artifact_validation, "package-validation-report.v0.json")
    assert_matches_schema(file_validation, "package-validation-report.v0.json")
    assert_matches_schema(bridge, "package-bridge-output.v0.json")

def test_package_artifact_validation_reports_missing_files(tmp_path):
    manifest_json = (REPO_ROOT / "examples/protein-package/manifest.json").read_text()

    validation = json.loads(
        biors.validate_package_manifest_artifacts(manifest_json, str(tmp_path))
    )

    assert validation["valid"] == False
    codes = {issue["code"] for issue in validation["structured_issues"]}
    assert "asset_read_failed" in codes
    assert_matches_schema(validation, "package-validation-report.v0.json")

def test_package_file_validation_reports_checksum_mismatch(tmp_path):
    package_dir = tmp_path / "protein-package"
    shutil.copytree(REPO_ROOT / "examples/protein-package", package_dir)
    (package_dir / "models/protein-seed.onnx").write_bytes(b"changed model")

    validation = json.loads(
        biors.validate_package_manifest_file(str(package_dir / "manifest.json"))
    )

    assert validation["valid"] == False
    codes = {issue["code"] for issue in validation["structured_issues"]}
    assert "checksum_mismatch" in codes
    assert_matches_schema(validation, "package-validation-report.v0.json")

def test_package_file_validation_reports_invalid_pipeline_config(tmp_path):
    package_dir = tmp_path / "protein-package"
    shutil.copytree(REPO_ROOT / "examples/protein-package", package_dir)
    config_path = package_dir / "pipelines/protein.toml"
    config = config_path.read_text()
    config_path.write_text(config.replace("max_length = 8", "max_length = 0"))
    manifest_path = package_dir / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    manifest["preprocessing"][0]["config"]["checksum"] = sha256_file(config_path)
    manifest_path.write_text(json.dumps(manifest))

    validation = json.loads(biors.validate_package_manifest_file(str(manifest_path)))

    assert validation["valid"] == False
    codes = {issue["code"] for issue in validation["structured_issues"]}
    assert "invalid_pipeline_config" in codes
    assert_matches_schema(validation, "package-validation-report.v0.json")

def test_package_artifact_validation_reports_invalid_pipeline_config(tmp_path):
    package_dir = tmp_path / "protein-package"
    shutil.copytree(REPO_ROOT / "examples/protein-package", package_dir)
    config_path = package_dir / "pipelines/protein.toml"
    config = config_path.read_text()
    config_path.write_text(config.replace("padding = \"fixed_length\"", "padding = \"bad\""))
    manifest_path = package_dir / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    manifest["preprocessing"][0]["config"]["checksum"] = sha256_file(config_path)

    validation = json.loads(
        biors.validate_package_manifest_artifacts(json.dumps(manifest), str(package_dir))
    )

    assert validation["valid"] == False
    codes = {issue["code"] for issue in validation["structured_issues"]}
    assert "invalid_pipeline_config" in codes
    assert_matches_schema(validation, "package-validation-report.v0.json")

def test_python_errors_expose_stable_code_for_invalid_package_json():
    with pytest.raises(biors.BioRsError) as exc_info:
        biors.inspect_package_manifest("{")

    assert exc_info.value.code == "json.invalid"
    assert "invalid JSON" in exc_info.value.message

def test_package_json_helpers_reject_unknown_manifest_fields():
    manifest = json.loads((REPO_ROOT / "examples/protein-package/manifest.json").read_text())
    manifest["unexpected_top"] = True

    with pytest.raises(biors.BioRsError, match="unknown field") as exc_info:
        biors.validate_package_manifest(json.dumps(manifest))
    assert exc_info.value.code == "json.invalid"

def assert_matches_schema(value, schema_name):
    schema = json.loads((REPO_ROOT / "schemas" / schema_name).read_text())
    jsonschema.Draft202012Validator(schema).validate(value)

def sha256_file(path):
    import hashlib

    return f"sha256:{hashlib.sha256(path.read_bytes()).hexdigest()}"
