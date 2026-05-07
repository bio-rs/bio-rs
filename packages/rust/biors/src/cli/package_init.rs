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
        None => {
            find_first_file(&args.project_dir, &["onnx"]).ok_or_else(|| CliError::Validation {
                code: "package.project_model_missing",
                message: "could not find an ONNX model in the Python project; pass --model"
                    .to_string(),
                location: Some(args.project_dir.display().to_string()),
            })?
        }
    };
    let tokenizer_config = match args.tokenizer_config {
        Some(path) => Some(path),
        None => find_named_file(&args.project_dir, "tokenizer_config.json"),
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

fn find_named_file(root: &Path, name: &str) -> Option<PathBuf> {
    find_file(root, &mut |path| {
        path.file_name().and_then(|value| value.to_str()) == Some(name)
    })
}

fn find_first_file(root: &Path, extensions: &[&str]) -> Option<PathBuf> {
    find_file(root, &mut |path| {
        path.extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .is_some_and(|extension| extensions.contains(&extension.as_str()))
    })
}

fn find_file(root: &Path, predicate: &mut impl FnMut(&Path) -> bool) -> Option<PathBuf> {
    for entry in std::fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        let file_type = entry.file_type().ok()?;
        if file_type.is_file() && predicate(&path) {
            return Some(path);
        }
        if file_type.is_dir() {
            if let Some(found) = find_file(&path, predicate) {
                return Some(found);
            }
        }
    }
    None
}
