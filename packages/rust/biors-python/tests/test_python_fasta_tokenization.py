import pytest

import biors


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
    assert report.sequences[0].valid is True


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
    assert sequence.valid is False
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
    assert tokenized[0].valid is True
    assert len(tokenized[0].tokens) == 6
    assert tokenized[0].warnings == []
    assert tokenized[0].errors == []


def test_tokenize_fasta_records_exposes_residue_diagnostics():
    fasta = ">seq1\nAX*\n"
    tokenized = biors.tokenize_fasta_records(fasta)
    assert tokenized[0].valid is False
    assert tokenized[0].warnings[0].residue == "X"
    assert tokenized[0].warnings[0].position == 2
    assert tokenized[0].errors[0].residue == "*"
    assert tokenized[0].errors[0].position == 3


def test_tokenize_fasta_records_accepts_nucleotide_profiles():
    dna = biors.tokenize_fasta_records(">dna\nACGTN\n", profile="dna-iupac")[0]
    rna = biors.tokenize_fasta_records(">rna\nACGUN\n", profile="rna-iupac")[0]

    assert dna.alphabet == "dna-iupac"
    assert dna.tokens == [0, 1, 2, 3, 4]
    assert dna.valid is False
    assert dna.warnings[0].residue == "N"
    assert rna.alphabet == "rna-iupac"
    assert rna.tokens == [0, 1, 2, 3, 4]


def test_tokenize_protein_normalizes_like_fasta_tokenization():
    direct = biors.tokenize_protein("ac de\tfg")
    from_fasta = biors.tokenize_fasta_records(">seq1\nac de\tfg\n")[0]
    assert direct.id == "user"
    assert direct.valid is True
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
