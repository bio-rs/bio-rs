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
    records: ModelInputRecord[];
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_WORKFLOW: &'static str = r#"
export interface WorkflowConfig {
    fastaBytes: Uint8Array;
    kind?: "auto" | "protein" | "dna" | "rna";
    profile?: "protein-20" | "protein-20-special";
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
