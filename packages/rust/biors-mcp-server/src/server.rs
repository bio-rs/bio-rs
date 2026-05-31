use biors_core::{
    model_input::{ModelInputPolicy, PaddingPolicy},
    sequence::{SequenceKind, SequenceKindSelection},
    verification::stable_input_hash,
    workflow::SequenceWorkflowInvocation,
};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData as McpError},
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct BiorsMcpServer;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TokenizeParams {
    /// FASTA text to tokenize.
    pub fasta_text: String,
    /// Tokenizer profile: "protein-20" or "protein-20-special".
    #[serde(default = "default_protein_profile")]
    pub profile: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ValidateParams {
    /// FASTA text to validate.
    pub fasta_text: String,
    /// Sequence kind: "auto", "protein", "dna", or "rna".
    #[serde(default = "default_auto_kind")]
    pub kind: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WorkflowParams {
    /// FASTA text for the workflow.
    pub fasta_text: String,
    /// Sequence kind: "auto" or "protein". DNA/RNA workflows are not model-input ready yet.
    #[serde(default = "default_auto_kind")]
    pub kind: String,
    /// Maximum token length per record.
    #[serde(default = "default_max_length")]
    pub max_length: usize,
    /// Token ID used for fixed-length padding.
    #[serde(default)]
    pub pad_token_id: u8,
    /// Padding policy: "fixed_length" or "no_padding".
    #[serde(default = "default_padding")]
    pub padding: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PackageValidateParams {
    /// Package manifest JSON string to validate.
    pub manifest_json: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DoctorOutput {
    pub biors_version: String,
    pub platform: String,
    pub mcp_server_ready: bool,
}

fn default_protein_profile() -> String {
    "protein-20".to_string()
}

fn default_auto_kind() -> String {
    "auto".to_string()
}

fn default_max_length() -> usize {
    512
}

fn default_padding() -> String {
    "fixed_length".to_string()
}

fn map_kind(kind: &str) -> Result<SequenceKindSelection, McpError> {
    match kind {
        "auto" => Ok(SequenceKindSelection::Auto),
        "protein" => Ok(SequenceKindSelection::Explicit(SequenceKind::Protein)),
        "dna" => Ok(SequenceKindSelection::Explicit(SequenceKind::Dna)),
        "rna" => Ok(SequenceKindSelection::Explicit(SequenceKind::Rna)),
        _ => Err(McpError::invalid_params(
            "invalid kind",
            Some(serde_json::json!({"kind": kind, "expected": ["auto", "protein", "dna", "rna"]})),
        )),
    }
}

fn map_padding(padding: &str) -> Result<PaddingPolicy, McpError> {
    match padding {
        "fixed_length" => Ok(PaddingPolicy::FixedLength),
        "no_padding" => Ok(PaddingPolicy::NoPadding),
        _ => Err(McpError::invalid_params(
            "invalid padding",
            Some(
                serde_json::json!({"padding": padding, "expected": ["fixed_length", "no_padding"]}),
            ),
        )),
    }
}

fn ensure_protein_workflow_kind(params: &WorkflowParams) -> Result<(), McpError> {
    let selection = map_kind(&params.kind)?;
    let report = biors_core::sequence::kind_validation::validate_fasta_input_with_kind(
        &params.fasta_text,
        selection,
    )
    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if report.kind_counts.dna > 0 || report.kind_counts.rna > 0 {
        return Err(McpError::invalid_params(
            "unsupported workflow kind",
            Some(serde_json::json!({
                "kind": params.kind,
                "detected_kind_counts": report.kind_counts,
                "supported": ["auto protein-only input", "protein"],
                "message": "MCP workflow currently supports protein model-input workflows only"
            })),
        ));
    }

    Ok(())
}

fn workflow_invocation(params: &WorkflowParams) -> SequenceWorkflowInvocation {
    SequenceWorkflowInvocation {
        command: "biors-mcp workflow".to_string(),
        arguments: vec![
            "--kind".to_string(),
            params.kind.clone(),
            "--max-length".to_string(),
            params.max_length.to_string(),
            "--pad-token-id".to_string(),
            params.pad_token_id.to_string(),
            "--padding".to_string(),
            params.padding.clone(),
        ],
    }
}

fn json_response<T: serde::Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

#[tool_router(server_handler)]
impl BiorsMcpServer {
    #[tool(description = "Tokenize protein FASTA text into stable token IDs")]
    fn tokenize(
        &self,
        Parameters(params): Parameters<TokenizeParams>,
    ) -> Result<CallToolResult, McpError> {
        let records = biors_core::fasta::parse_fasta_records(&params.fasta_text)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let config = match params.profile.as_str() {
            "protein-20" => biors_core::tokenizer::ProteinTokenizerConfig {
                profile: biors_core::tokenizer::ProteinTokenizerProfile::Protein20,
                add_special_tokens: false,
            },
            "protein-20-special" => biors_core::tokenizer::ProteinTokenizerConfig {
                profile: biors_core::tokenizer::ProteinTokenizerProfile::Protein20Special,
                add_special_tokens: true,
            },
            _ => {
                return Err(McpError::invalid_params(
                    "invalid profile",
                    Some(
                        serde_json::json!({"profile": params.profile, "expected": ["protein-20", "protein-20-special"]}),
                    ),
                ));
            }
        };

        let mut tokenized = Vec::with_capacity(records.len());
        for record in records {
            let protein = biors_core::sequence::ProteinSequence {
                id: record.id.clone(),
                sequence: record.sequence.clone(),
            };
            let t = biors_core::tokenizer::tokenize_protein_with_config(&protein, &config);
            tokenized.push(t);
        }

        json_response(&tokenized)
    }

    #[tool(description = "Validate biological sequences (protein, DNA, RNA, or auto-detect)")]
    fn validate(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        let selection = map_kind(&params.kind)?;
        let report = biors_core::sequence::kind_validation::validate_fasta_input_with_kind(
            &params.fasta_text,
            selection,
        )
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        json_response(&report)
    }

    #[tool(description = "Run end-to-end validation -> tokenization -> model-input workflow")]
    fn workflow(
        &self,
        Parameters(params): Parameters<WorkflowParams>,
    ) -> Result<CallToolResult, McpError> {
        ensure_protein_workflow_kind(&params)?;
        let records = biors_core::fasta::parse_fasta_records(&params.fasta_text)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let policy = ModelInputPolicy {
            max_length: params.max_length,
            pad_token_id: params.pad_token_id,
            padding: map_padding(&params.padding)?,
        };

        let input_hash = stable_input_hash(&params.fasta_text);
        let output = biors_core::workflow::prepare_protein_model_input_workflow_with_invocation(
            input_hash,
            &records,
            policy,
            workflow_invocation(&params),
        )
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        json_response(&output)
    }

    #[tool(description = "Validate a package manifest JSON string")]
    fn package_validate(
        &self,
        Parameters(params): Parameters<PackageValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        let manifest: biors_core::package::PackageManifest =
            serde_json::from_str(&params.manifest_json)
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        let report = biors_core::package::validate_package_manifest(&manifest);
        json_response(&report)
    }

    #[tool(description = "Report platform readiness diagnostics for the MCP server")]
    fn doctor(&self) -> Result<CallToolResult, McpError> {
        let output = DoctorOutput {
            biors_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            mcp_server_ready: true,
        };
        json_response(&output)
    }
}
