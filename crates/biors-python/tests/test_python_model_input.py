import pytest

import biors


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
    assert model_input.records[0].truncated is False


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
