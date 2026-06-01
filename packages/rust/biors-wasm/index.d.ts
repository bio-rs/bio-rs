// index.d.ts
// Hand-written public TypeScript declarations for @bio-rs/biors-wasm.
// Re-exports and refines the auto-generated wasm-pack types.

export {
    parseFasta,
    validateFasta,
    tokenize,
    buildModelInput,
    buildModelInputWithPolicy,
    runWorkflow,
} from "./biors_wasm.js";

export type {
    FastaRecord,
    SequenceValidationIssue,
    ValidatedSequence,
    SequenceKindCounts,
    ValidationReport,
    ResidueIssue,
    TokenizedRecord,
    TokenizeOutput,
    ModelInputRecord,
    ModelInputOutput,
    WorkflowConfig,
    WorkflowReadinessIssue,
    WorkflowOutput,
} from "./biors_wasm.d.ts";

// Refine function signatures for consumers
declare module "./biors_wasm.js" {
    export function parseFasta(bytes: Uint8Array): FastaRecord[];
    export function validateFasta(bytes: Uint8Array, kind: string): ValidationReport;
    export function tokenize(records: FastaRecord[], profile: string): TokenizeOutput;
    export function buildModelInput(tokenized: TokenizedRecord[], maxLength: number): ModelInputOutput;
    export function buildModelInputWithPolicy(
        tokenized: TokenizedRecord[],
        maxLength: number,
        padTokenId: number,
        padding: "fixed_length" | "no_padding"
    ): ModelInputOutput;
    export function runWorkflow(config: WorkflowConfig): WorkflowOutput;
}
