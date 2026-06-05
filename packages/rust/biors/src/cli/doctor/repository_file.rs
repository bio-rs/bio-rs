use super::report::{DoctorCheck, DoctorStatus};
use std::path::{Path, PathBuf};

pub(super) fn repo_file_check(
    capability: &'static str,
    name: &'static str,
    relative_path: &str,
    pass_message: &'static str,
) -> DoctorCheck {
    match find_repo_file(relative_path) {
        Some(path) => DoctorCheck {
            capability,
            name,
            status: DoctorStatus::Pass,
            message: format!("{pass_message}: {}", path.display()),
            hint: None,
        },
        None => DoctorCheck {
            capability,
            name,
            status: DoctorStatus::Warn,
            message: format!("{relative_path} was not found from the current checkout"),
            hint: Some("run doctor from a bio-rs checkout or verify the release artifact contents"),
        },
    }
}

fn find_repo_file(relative_path: &str) -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let direct = current_dir.join(relative_path);
    if direct.exists() {
        return Some(direct);
    }

    let source_checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let source_path = source_checkout.join(relative_path);
    if source_path.exists() {
        return Some(source_path);
    }

    None
}
