use crate::compact::{
    compact_tokenize_response, compact_validate_response, compact_workflow_response,
    should_compact_fasta,
};
use crate::package_validation::{PackageValidateFieldsParams, PackageValidateParams};
use biors_core::{
    error::BioRsError,
    model_input::{ModelInputBuildError, ModelInputPolicy, PaddingPolicy},
    tokenizer::ProteinTokenizerProfile,
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
    /// Tokenizer profile: protein-20, protein-20-special, dna-iupac,
    /// dna-iupac-special, rna-iupac, or rna-iupac-special.
    #[serde(default = "default_protein_profile")]
    pub profile: String,
    #[serde(default)]
    pub include_records: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ValidateParams {
    /// FASTA text to validate.
    pub fasta_text: String,
    /// Sequence kind: "auto", "protein", "dna", or "rna".
    #[serde(default = "default_auto_kind")]
    pub kind: String,
    #[serde(default)]
    pub include_records: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WorkflowParams {
    /// FASTA text for the workflow.
    pub fasta_text: String,
    /// Sequence kind: "auto", "protein", "dna", or "rna".
    #[serde(default = "default_auto_kind")]
    pub kind: String,
    /// Optional tokenizer profile. Defaults from kind, or auto-detected kind when kind is "auto".
    #[serde(default)]
    pub profile: Option<String>,
    /// Maximum token length per record.
    #[serde(default = "default_max_length")]
    pub max_length: usize,
    /// Token ID used for fixed-length padding.
    #[serde(default)]
    pub pad_token_id: u8,
    /// Padding policy: "fixed_length" or "no_padding".
    #[serde(default = "default_padding")]
    pub padding: String,
    #[serde(default)]
    pub include_payload: bool,
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

fn fasta_invalid_params(error: BioRsError) -> McpError {
    McpError::invalid_params(
        error.to_string(),
        Some(serde_json::json!({
            "code": error.code(),
            "location": error.location(),
        })),
    )
}

fn model_input_invalid_params(error: ModelInputBuildError) -> McpError {
    McpError::invalid_params(
        error.to_string(),
        Some(serde_json::json!({
            "code": "model_input.invalid_policy_or_record"
        })),
    )
}

fn workflow_invocation(
    params: &WorkflowParams,
    profile: ProteinTokenizerProfile,
) -> SequenceWorkflowInvocation {
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
            "--profile".to_string(),
            profile.as_str().to_string(),
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
    #[tool(description = "Tokenize FASTA text into stable protein, DNA, or RNA token IDs")]
    fn tokenize(
        &self,
        Parameters(params): Parameters<TokenizeParams>,
    ) -> Result<CallToolResult, McpError> {
        let records = biors_core::fasta::parse_fasta_records(&params.fasta_text)
            .map_err(fasta_invalid_params)?;

        let config = crate::profile::config_for_profile(&params.profile)?;

        let mut tokenized = Vec::with_capacity(records.len());
        for record in records {
            let protein = biors_core::sequence::ProteinSequence {
                id: record.id.clone(),
                sequence: record.sequence.clone(),
            };
            let t = biors_core::tokenizer::tokenize_protein_with_config(&protein, &config);
            tokenized.push(t);
        }

        if should_compact_fasta(&params.fasta_text, params.include_records) {
            json_response(&compact_tokenize_response(&tokenized))
        } else {
            json_response(&tokenized)
        }
    }

    #[tool(description = "Validate biological sequences (protein, DNA, RNA, or auto-detect)")]
    fn validate(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        let selection = crate::profile::map_kind(&params.kind)?;
        let report = biors_core::sequence::kind_validation::validate_fasta_input_with_kind(
            &params.fasta_text,
            selection,
        )
        .map_err(fasta_invalid_params)?;

        if should_compact_fasta(&params.fasta_text, params.include_records) {
            json_response(&compact_validate_response(&report))
        } else {
            json_response(&report)
        }
    }

    #[tool(description = "Run end-to-end validation -> tokenization -> model-input workflow")]
    fn workflow(
        &self,
        Parameters(params): Parameters<WorkflowParams>,
    ) -> Result<CallToolResult, McpError> {
        let profile = crate::profile::workflow_profile(
            &params.kind,
            params.profile.as_deref(),
            &params.fasta_text,
        )?;
        let records = biors_core::fasta::parse_fasta_records(&params.fasta_text)
            .map_err(fasta_invalid_params)?;

        let policy = ModelInputPolicy {
            max_length: params.max_length,
            pad_token_id: params.pad_token_id,
            padding: map_padding(&params.padding)?,
        };

        let input_hash = stable_input_hash(&params.fasta_text);
        let config = biors_core::tokenizer::protein_tokenizer_config_for_profile(profile);
        let output = biors_core::workflow::prepare_model_input_workflow_with_config(
            input_hash,
            &records,
            policy,
            config,
            workflow_invocation(&params, profile),
        )
        .map_err(model_input_invalid_params)?;

        if should_compact_fasta(&params.fasta_text, params.include_payload) {
            json_response(&compact_workflow_response(&output))
        } else {
            json_response(&output)
        }
    }

    #[tool(
        description = "Validate a package manifest JSON string without filesystem artifact checks"
    )]
    fn package_validate_fields(
        &self,
        Parameters(params): Parameters<PackageValidateFieldsParams>,
    ) -> Result<CallToolResult, McpError> {
        let report = crate::package_validation::validate_fields(params)?;
        json_response(&report)
    }

    #[tool(description = "Validate a package manifest and its filesystem artifacts")]
    fn package_validate(
        &self,
        Parameters(params): Parameters<PackageValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        let report = crate::package_validation::validate(params)?;
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
