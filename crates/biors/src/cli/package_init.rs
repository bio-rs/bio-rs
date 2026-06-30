use super::{PackageConvertProjectArgs, PackageInitArgs};
use crate::cli::package_skeleton::{create_package_skeleton, PackageSkeletonRequest};
use crate::errors::CliError;
use std::path::{Path, PathBuf};

pub(crate) fn run_package_init(args: PackageInitArgs) -> Result<(), CliError> {
    create_package_skeleton(PackageSkeletonRequest {
        output_dir: args.output_dir,
        name: args.name,
        model: args.model,
        tokenizer_config: args.tokenizer_config,
        fixture_input: args.fixture_input,
        fixture_output: args.fixture_output,
        license: args.license,
        citation: args.citation,
        doi: args.doi,
        model_card_summary: args.model_card_summary,
        intended_use: args.intended_use,
        limitations: args.limitations,
        force: args.force,
    })
}

pub(crate) fn run_package_convert_project(args: PackageConvertProjectArgs) -> Result<(), CliError> {
    let model = match args.model {
        Some(model) => model,
        None => select_single_model_candidate(&args.project_dir, args.include_generated)?,
    };
    let tokenizer_config = match args.tokenizer_config {
        Some(path) => Some(path),
        None => select_optional_tokenizer_config_candidate(
            &args.project_dir,
            "tokenizer_config.json",
            args.include_generated,
        )?,
    };

    create_package_skeleton(PackageSkeletonRequest {
        output_dir: args.output,
        name: args.name,
        model,
        tokenizer_config,
        fixture_input: args.fixture_input,
        fixture_output: args.fixture_output,
        license: args.license,
        citation: args.citation,
        doi: args.doi,
        model_card_summary: args.model_card_summary,
        intended_use: args.intended_use,
        limitations: args.limitations,
        force: args.force,
    })
}

fn select_single_model_candidate(
    root: &Path,
    include_generated: bool,
) -> Result<PathBuf, CliError> {
    let candidates = find_files(root, include_generated, &mut |path| {
        path.extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .is_some_and(|extension| extension == "onnx")
    });

    match candidates.as_slice() {
        [] => Err(CliError::Validation {
            code: "package.project_model_missing",
            message: "could not find an ONNX model in the Python project; pass --model".to_string(),
            location: Some(root.display().to_string()),
        }),
        [candidate] => Ok(candidate.clone()),
        _ => Err(CliError::ValidationDetails {
            code: "package.project_model_ambiguous",
            message: "found multiple ONNX model candidates; pass --model".to_string(),
            location: Some(root.display().to_string()),
            details: serde_json::json!({
                "candidates": candidates
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
            }),
        }),
    }
}

fn select_optional_tokenizer_config_candidate(
    root: &Path,
    name: &str,
    include_generated: bool,
) -> Result<Option<PathBuf>, CliError> {
    let candidates = find_files(root, include_generated, &mut |path| {
        path.file_name().and_then(|value| value.to_str()) == Some(name)
    });

    match candidates.as_slice() {
        [] => Ok(None),
        [candidate] => Ok(Some(candidate.clone())),
        _ => Err(CliError::ValidationDetails {
            code: "package.project_tokenizer_config_ambiguous",
            message: "found multiple tokenizer_config.json candidates; pass --tokenizer-config"
                .to_string(),
            location: Some(root.display().to_string()),
            details: serde_json::json!({
                "candidates": candidates
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
            }),
        }),
    }
}

fn find_files(
    root: &Path,
    include_generated: bool,
    predicate: &mut impl FnMut(&Path) -> bool,
) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect_files(root, include_generated, predicate, &mut found);
    found.sort_by(|left, right| {
        left.to_string_lossy()
            .as_ref()
            .cmp(right.to_string_lossy().as_ref())
    });
    found
}

fn collect_files(
    root: &Path,
    include_generated: bool,
    predicate: &mut impl FnMut(&Path) -> bool,
    found: &mut Vec<PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_file() && predicate(&path) {
            found.push(path);
        } else if file_type.is_dir() {
            if !include_generated && is_generated_dir(&path) {
                continue;
            }
            collect_files(&path, include_generated, predicate, found);
        }
    }
}

fn is_generated_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | ".venv"
            | "venv"
            | "env"
            | ".cache"
            | "__pycache__"
            | ".ipynb_checkpoints"
            | "target"
            | "build"
            | "dist"
    )
}
