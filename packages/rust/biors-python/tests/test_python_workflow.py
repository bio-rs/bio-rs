import json

import pytest

import biors


def test_prepare_workflow():
    fasta = ">seq1\nACDEFG"
    records = biors.parse_fasta_records(fasta)
    output = biors.prepare_workflow(
        "fnv1a64:0000000000000001",
        records,
        max_length=10,
        padding="fixed_length",
    )
    assert output.model_ready is True
    assert output.input_hash == "fnv1a64:0000000000000001"
    assert len(output.records) == 1
    assert len(output.records[0].input_ids) == 10
    report = json.loads(output.report_json)
    assert report["model_ready"] is True
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
    assert output.model_ready is False
    assert output.records == []
    report = json.loads(output.report_json)
    assert report["model_ready"] is False
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

    assert output.model_ready is False
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

    assert output.model_ready is True
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
    assert output.model_ready is True
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

    assert output.model_ready is True
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

    assert output.model_ready is True
    assert output.records[0].input_ids == [0, 1, 2, 3]
    assert report["provenance"]["validation_alphabet"] == "rna-iupac"
    assert report["provenance"]["tokenizer"]["name"] == "rna-iupac"
