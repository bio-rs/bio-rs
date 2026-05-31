from collections.abc import Sequence


class ResidueIssue:
    residue: str
    position: int


class ProteinSequence:
    id: str
    sequence: str


class SequenceValidationReport:
    records: int
    valid_records: int
    warning_count: int
    error_count: int


class TokenizedProtein:
    id: str
    alphabet: str
    valid: bool
    tokens: list[int]
    length: int
    warnings: list[ResidueIssue]
    errors: list[ResidueIssue]


class ModelInputRecord:
    id: str
    input_ids: list[int]
    attention_mask: list[int]
    truncated: bool


class ModelInput:
    records: list[ModelInputRecord]


class SequenceWorkflowOutput:
    model_ready: bool
    input_hash: str
    records: list[ModelInputRecord]


def parse_fasta_records(fasta_text: str) -> list[ProteinSequence]: ...


def validate_fasta_input(fasta_text: str) -> SequenceValidationReport: ...


def tokenize_fasta_records(fasta_text: str) -> list[TokenizedProtein]: ...


def tokenize_protein(sequence: str) -> TokenizedProtein: ...


def build_model_inputs_checked(
    tokenized: Sequence[TokenizedProtein],
    max_length: int,
    pad_token_id: int = 0,
    padding: str = "no_padding",
) -> ModelInput: ...


def prepare_workflow(
    input_hash: str,
    records: Sequence[ProteinSequence],
    max_length: int,
    pad_token_id: int = 0,
    padding: str = "no_padding",
) -> SequenceWorkflowOutput: ...


def prepare_workflow_from_fasta(
    fasta_text: str,
    max_length: int,
    pad_token_id: int = 0,
    padding: str = "no_padding",
) -> SequenceWorkflowOutput: ...


def inspect_package_manifest(manifest_json: str) -> str: ...


def validate_package_manifest(manifest_json: str) -> str: ...


def plan_runtime_bridge(manifest_json: str) -> str: ...
