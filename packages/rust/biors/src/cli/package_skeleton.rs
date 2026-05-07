use crate::cli::package_skeleton_files::{
    copy_asset, file_sha256, fixture_name, validate_required_list, write_docs,
    write_pipeline_config, write_tokenizer_config,
};
use crate::errors::CliError;
use crate::output::print_success;
use biors_core::package::{
    ModelArtifact, ModelFormat, PackageDirectoryLayout, PackageFixture, PackageManifest,
    PipelineConfigArtifact, PipelineStep, RuntimeBackend, RuntimeTarget, RuntimeTargetPlatform,
    SchemaVersion,
};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct PackageSkeletonOutput {
    package: String,
    output_dir: String,
    manifest_path: String,
    manifest_sha256: String,
    created_files: Vec<String>,
    next_commands: Vec<String>,
    notes: Vec<String>,
}

pub(crate) struct PackageSkeletonRequest {
    pub(crate) output_dir: PathBuf,
    pub(crate) name: String,
    pub(crate) model: PathBuf,
    pub(crate) tokenizer_config: Option<PathBuf>,
    pub(crate) fixture_input: PathBuf,
    pub(crate) fixture_output: PathBuf,
    pub(crate) license: String,
    pub(crate) citation: String,
    pub(crate) doi: Option<String>,
    pub(crate) model_card_summary: String,
    pub(crate) intended_use: Vec<String>,
    pub(crate) limitations: Vec<String>,
    pub(crate) force: bool,
}

pub(crate) fn create_package_skeleton(request: PackageSkeletonRequest) -> Result<(), CliError> {
    validate_required_list("--intended-use", &request.intended_use)?;
    validate_required_list("--limitation", &request.limitations)?;

    let manifest_path = request.output_dir.join("manifest.json");
    if manifest_path.exists() && !request.force {
        return Err(CliError::Validation {
            code: "package.init_exists",
            message: "package manifest already exists; pass --force to overwrite".to_string(),
            location: Some(manifest_path.display().to_string()),
        });
    }

    for dir in [
        "models",
        "tokenizers",
        "pipelines",
        "fixtures",
        "observed",
        "docs",
    ] {
        std::fs::create_dir_all(request.output_dir.join(dir)).map_err(CliError::Write)?;
    }

    let mut created_files = Vec::new();
    let model_rel = copy_asset(
        &request.model,
        &request.output_dir,
        "models",
        &mut created_files,
    )?;
    let fixture_input_rel = copy_asset(
        &request.fixture_input,
        &request.output_dir,
        "fixtures",
        &mut created_files,
    )?;
    let fixture_output_rel = copy_asset(
        &request.fixture_output,
        &request.output_dir,
        "fixtures",
        &mut created_files,
    )?;
    let (tokenizer_asset, tokenizer_profile, notes) =
        write_tokenizer_config(&request, &mut created_files)?;
    let pipeline_rel = write_pipeline_config(
        &request.output_dir,
        &fixture_input_rel,
        tokenizer_profile,
        &mut created_files,
    )?;
    let metadata = write_docs(&request, &mut created_files)?;

    let manifest = PackageManifest {
        schema_version: SchemaVersion::BiorsPackageV1,
        name: request.name.clone(),
        package_layout: Some(PackageDirectoryLayout {
            manifest: "manifest.json".to_string(),
            models: "models".to_string(),
            tokenizers: Some("tokenizers".to_string()),
            vocabs: None,
            pipelines: Some("pipelines".to_string()),
            fixtures: "fixtures".to_string(),
            observed: Some("observed".to_string()),
            docs: "docs".to_string(),
        }),
        metadata: Some(metadata),
        model: ModelArtifact {
            format: ModelFormat::Onnx,
            path: model_rel.clone(),
            checksum: Some(file_sha256(&request.output_dir.join(&model_rel))?),
        },
        tokenizer: Some(tokenizer_asset),
        vocab: None,
        preprocessing: vec![PipelineStep {
            name: "protein_fasta_tokenize".to_string(),
            implementation: "biors-core".to_string(),
            contract: tokenizer_profile.as_str().to_string(),
            contract_version: Some(format!("{}.v0", tokenizer_profile.as_str())),
            config: Some(PipelineConfigArtifact {
                path: pipeline_rel.clone(),
                schema_version: biors_core::versioning::PipelineConfigVersion::BiorsPipelineV0,
                checksum: Some(file_sha256(&request.output_dir.join(&pipeline_rel))?),
            }),
        }],
        postprocessing: Vec::new(),
        runtime: RuntimeTarget {
            backend: RuntimeBackend::OnnxWebgpu,
            target: RuntimeTargetPlatform::BrowserWasmWebgpu,
            version: Some("onnx-webgpu.v0".to_string()),
        },
        expected_input: None,
        expected_output: None,
        fixtures: vec![PackageFixture {
            name: fixture_name(&request.fixture_input),
            input: fixture_input_rel.clone(),
            expected_output: fixture_output_rel.clone(),
            input_hash: Some(file_sha256(&request.output_dir.join(&fixture_input_rel))?),
            expected_output_hash: Some(file_sha256(&request.output_dir.join(&fixture_output_rel))?),
        }],
    };

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(CliError::Serialization)?;
    std::fs::write(&manifest_path, format!("{manifest_json}\n")).map_err(CliError::Write)?;
    created_files.push(manifest_path.display().to_string());
    let manifest_sha256 = file_sha256(&manifest_path)?;

    let output = PackageSkeletonOutput {
        package: request.name,
        output_dir: request.output_dir.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        manifest_sha256,
        created_files,
        next_commands: vec![
            format!("biors package validate {}", manifest_path.display()),
            format!(
                "biors package verify {} <observations.json>",
                manifest_path.display()
            ),
        ],
        notes,
    };
    print_success(None, output)
}
