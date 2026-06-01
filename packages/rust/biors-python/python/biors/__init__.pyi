from collections.abc import Sequence


class BioRsError(ValueError):
    code: str
    message: str
    location: dict[str, int | None] | None


class ResidueIssue:
    residue: str
    position: int

    def __init__(self, residue: str, position: int) -> None: ...


class ProteinSequence:
    id: str
    sequence: str

    def __init__(self, id: str, sequence: str) -> None: ...


class ValidatedSequence:
    id: str
    sequence: str
    alphabet: str
    valid: bool
    warnings: list[ResidueIssue]
    errors: list[ResidueIssue]

    def __init__(
        self,
        id: str,
        sequence: str,
        alphabet: str = "protein-20",
        valid: bool = True,
        warnings: Sequence[ResidueIssue] | None = None,
        errors: Sequence[ResidueIssue] | None = None,
    ) -> None: ...


class SequenceValidationReport:
    records: int
    valid_records: int
    warning_count: int
    error_count: int
    sequences: list[ValidatedSequence]


class TokenizedProtein:
    id: str
    alphabet: str
    valid: bool
    tokens: list[int]
    length: int
    warnings: list[ResidueIssue]
    errors: list[ResidueIssue]

    def __init__(
        self,
        id: str,
        tokens: Sequence[int],
        length: int | None = None,
        alphabet: str = "protein-20",
        valid: bool = True,
        warnings: Sequence[ResidueIssue] | None = None,
        errors: Sequence[ResidueIssue] | None = None,
    ) -> None: ...


class ModelInputRecord:
    id: str
    input_ids: list[int]
    attention_mask: list[int]
    truncated: bool

    def __init__(
        self,
        id: str,
        input_ids: Sequence[int],
        attention_mask: Sequence[int],
        truncated: bool,
    ) -> None: ...


class ModelInput:
    records: list[ModelInputRecord]


class SequenceWorkflowOutput:
    model_ready: bool
    input_hash: str
    records: list[ModelInputRecord]
    report_json: str


def parse_fasta_records(fasta_text: str) -> list[ProteinSequence]: ...


def validate_fasta_input(fasta_text: str) -> SequenceValidationReport: ...


def tokenize_fasta_records(fasta_text: str) -> list[TokenizedProtein]: ...


def tokenize_protein(sequence: str, id: str = "user") -> TokenizedProtein: ...


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


def validate_package_manifest_artifacts(manifest_json: str, base_dir: str) -> str: ...


def validate_package_manifest_file(manifest_path: str) -> str: ...


def plan_runtime_bridge(manifest_json: str) -> str: ...
