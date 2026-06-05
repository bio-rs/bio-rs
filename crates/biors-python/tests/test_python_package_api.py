import json
import shutil

import pytest

import biors
from python_api_support import REPO_ROOT, assert_matches_schema, sha256_file


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
    manifest_json = (REPO_ROOT / "testdata/protein-package/manifest.json").read_text()
    assert "validate_package_manifest_artifacts" in biors.__all__
    assert "validate_package_manifest_file" in biors.__all__
    validation = json.loads(biors.validate_package_manifest(manifest_json))
    artifact_validation = json.loads(
        biors.validate_package_manifest_artifacts(
            manifest_json, str(REPO_ROOT / "testdata/protein-package")
        )
    )
    file_validation = json.loads(
        biors.validate_package_manifest_file(
            str(REPO_ROOT / "testdata/protein-package/manifest.json")
        )
    )
    bridge = json.loads(biors.plan_runtime_bridge(manifest_json))

    assert validation["valid"] is True
    assert artifact_validation["valid"] is True
    assert file_validation["valid"] is True
    assert bridge["ready"] is True
    assert_matches_schema(validation, "package-validation-report.v0.json")
    assert_matches_schema(artifact_validation, "package-validation-report.v0.json")
    assert_matches_schema(file_validation, "package-validation-report.v0.json")
    assert_matches_schema(bridge, "package-bridge-output.v0.json")


def test_package_artifact_validation_reports_missing_files(tmp_path):
    manifest_json = (REPO_ROOT / "testdata/protein-package/manifest.json").read_text()

    validation = json.loads(
        biors.validate_package_manifest_artifacts(manifest_json, str(tmp_path))
    )

    assert validation["valid"] is False
    codes = {issue["code"] for issue in validation["structured_issues"]}
    assert "asset_read_failed" in codes
    assert_matches_schema(validation, "package-validation-report.v0.json")


def test_package_file_validation_reports_checksum_mismatch(tmp_path):
    package_dir = tmp_path / "protein-package"
    shutil.copytree(REPO_ROOT / "testdata/protein-package", package_dir)
    (package_dir / "models/protein-seed.onnx").write_bytes(b"changed model")

    validation = json.loads(
        biors.validate_package_manifest_file(str(package_dir / "manifest.json"))
    )

    assert validation["valid"] is False
    codes = {issue["code"] for issue in validation["structured_issues"]}
    assert "checksum_mismatch" in codes
    assert_matches_schema(validation, "package-validation-report.v0.json")


def test_package_file_validation_reports_invalid_pipeline_config(tmp_path):
    package_dir = tmp_path / "protein-package"
    shutil.copytree(REPO_ROOT / "testdata/protein-package", package_dir)
    config_path = package_dir / "pipelines/protein.toml"
    config = config_path.read_text()
    config_path.write_text(config.replace("max_length = 8", "max_length = 0"))
    manifest_path = package_dir / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    manifest["preprocessing"][0]["config"]["checksum"] = sha256_file(config_path)
    manifest_path.write_text(json.dumps(manifest))

    validation = json.loads(biors.validate_package_manifest_file(str(manifest_path)))

    assert validation["valid"] is False
    codes = {issue["code"] for issue in validation["structured_issues"]}
    assert "invalid_pipeline_config" in codes
    assert_matches_schema(validation, "package-validation-report.v0.json")


def test_package_artifact_validation_reports_invalid_pipeline_config(tmp_path):
    package_dir = tmp_path / "protein-package"
    shutil.copytree(REPO_ROOT / "testdata/protein-package", package_dir)
    config_path = package_dir / "pipelines/protein.toml"
    config = config_path.read_text()
    config_path.write_text(config.replace("padding = \"fixed_length\"", "padding = \"bad\""))
    manifest_path = package_dir / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    manifest["preprocessing"][0]["config"]["checksum"] = sha256_file(config_path)

    validation = json.loads(
        biors.validate_package_manifest_artifacts(json.dumps(manifest), str(package_dir))
    )

    assert validation["valid"] is False
    codes = {issue["code"] for issue in validation["structured_issues"]}
    assert "invalid_pipeline_config" in codes
    assert_matches_schema(validation, "package-validation-report.v0.json")


def test_python_errors_expose_stable_code_for_invalid_package_json():
    with pytest.raises(biors.BioRsError) as exc_info:
        biors.inspect_package_manifest("{")

    assert exc_info.value.code == "json.invalid"
    assert "invalid JSON" in exc_info.value.message


def test_package_json_helpers_reject_unknown_manifest_fields():
    manifest = json.loads((REPO_ROOT / "testdata/protein-package/manifest.json").read_text())
    manifest["unexpected_top"] = True

    with pytest.raises(biors.BioRsError, match="unknown field") as exc_info:
        biors.validate_package_manifest(json.dumps(manifest))
    assert exc_info.value.code == "json.invalid"
