import biors

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
    assert len(tokenized[0].tokens) == 6

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
    assert len(output.records) == 1
    assert len(output.records[0].input_ids) == 10
