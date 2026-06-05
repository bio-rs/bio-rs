use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_FASTA_RECORD: &'static str = r#"
export interface FastaRecord {
    id: string;
    sequence: string;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_VALIDATION_ISSUE: &'static str = r#"
export interface SequenceValidationIssue {
    symbol: string;
    position: number;
    kind: "protein" | "dna" | "rna";
    code: "ambiguous_symbol" | "invalid_symbol";
    message: string;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_VALIDATED_SEQUENCE: &'static str = r#"
export interface ValidatedSequence {
    id: string;
    sequence: string;
    kind: "protein" | "dna" | "rna";
    alphabet: string;
    valid: boolean;
    warnings: SequenceValidationIssue[];
    errors: SequenceValidationIssue[];
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_KIND_COUNTS: &'static str = r#"
export interface SequenceKindCounts {
    protein: number;
    dna: number;
    rna: number;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_VALIDATION_REPORT: &'static str = r#"
export interface ValidationReport {
    records: number;
    valid_records: number;
    warning_count: number;
    error_count: number;
    kind_counts: SequenceKindCounts;
    sequences: ValidatedSequence[];
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_RESIDUE_ISSUE: &'static str = r#"
export interface ResidueIssue {
    residue: string;
    position: number;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_TOKENIZED: &'static str = r#"
export interface TokenizedRecord {
    id: string;
    tokens: number[];
    length: number;
    alphabet: string;
    valid: boolean;
    warnings: ResidueIssue[];
    errors: ResidueIssue[];
}

export interface TokenizeOutput {
    inputIds: number[][];
    attentionMask: number[][];
    ids: string[];
    records: TokenizedRecord[];
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_MODEL_INPUT: &'static str = r#"
export interface ModelInputRecord {
    id: string;
    input_ids: number[];
    attention_mask: number[];
    truncated: boolean;
}

export interface ModelInputOutput {
    policy: {
        max_length: number;
        pad_token_id: number;
        padding: "fixed_length" | "no_padding";
    };
    records: ModelInputRecord[];
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_WORKFLOW: &'static str = r#"
export interface WorkflowConfig {
    fastaBytes: Uint8Array;
    kind?: "auto" | "protein" | "dna" | "rna";
    profile?: "protein-20" | "protein-20-special" | "dna-iupac" | "dna-iupac-special" | "rna-iupac" | "rna-iupac-special";
    maxLength: number;
    padding?: "fixed_length" | "no_padding";
    padTokenId?: number;
}

export interface WorkflowReadinessIssue {
    code: string;
    id: string;
    warning_count: number;
    error_count: number;
    message: string;
}

export interface WorkflowOutput {
    workflow: string;
    model_ready: boolean;
    validation: ValidationReport;
    tokenization: {
        summary: {
            records: number;
            total_length: number;
            valid_records: number;
            warning_count: number;
            error_count: number;
        };
        records: TokenizedRecord[];
    };
    model_input: ModelInputOutput | null;
    readiness_issues: WorkflowReadinessIssue[];
    provenance: {
        biors_core_version: string;
        invocation: {
            command: string;
            arguments: string[];
        };
        input_hash: string;
        normalization: string;
        validation_alphabet: string;
        tokenizer: {
            name: string;
            vocab_size: number;
            unknown_token_id: number;
            unknown_token_policy: string;
        };
        model_input_policy: {
            max_length: number;
            pad_token_id: number;
            padding: string;
        };
        hashes: {
            vocabulary_sha256: string;
            output_data_sha256: string;
        };
    };
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_BROWSER_TOOLING: &'static str = r#"
export type BrowserBioFormat = "fasta" | "fastq" | "pdb" | "smiles";

export interface BrowserFileInput {
    bytes: Uint8Array;
    name?: string;
    format?: BrowserBioFormat;
    kind?: "auto" | "protein" | "dna" | "rna";
    profile?: "protein-20" | "protein-20-special" | "dna-iupac" | "dna-iupac-special" | "rna-iupac" | "rna-iupac-special";
}

export interface BrowserExecutionPolicy {
    schema_version: "biors.browser_tooling.v0";
    execution_mode: "wasm_local";
    network_access: "none";
    uploads_input_data: false;
    external_model_calls: false;
    persistence: "caller_controlled";
    max_input_bytes: number;
    warning_input_bytes: number;
    streaming: {
        supported: false;
        behavior: string;
        caller_guidance: string;
    };
    supported_validation_formats: BrowserBioFormat[];
    supported_tokenization_formats: "fasta"[];
}

export interface BrowserFileDescriptor {
    name?: string;
    format: BrowserBioFormat;
    size_bytes: number;
    content_sha256: string;
    input_hash?: string;
}

export interface BrowserFileWarning {
    code: string;
    message: string;
}

export interface BrowserFileInspection {
    schema_version: "biors.browser_tooling.v0";
    file: BrowserFileDescriptor;
    accepted: boolean;
    warnings: BrowserFileWarning[];
}

export interface BrowserValidationOutput {
    schema_version: "biors.browser_tooling.v0";
    file: BrowserFileDescriptor;
    report: unknown;
    warnings: BrowserFileWarning[];
}

export interface BrowserTokenizationOutput {
    schema_version: "biors.browser_tooling.v0";
    file: BrowserFileDescriptor;
    tokenization: TokenizeOutput;
    model_input_policy_hint: {
        max_length_required: boolean;
        supported_padding: Array<"fixed_length" | "no_padding">;
        note: string;
    };
    warnings: BrowserFileWarning[];
}
"#;
