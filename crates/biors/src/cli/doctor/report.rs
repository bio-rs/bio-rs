use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct DoctorReport {
    pub cli_version: &'static str,
    pub platform: PlatformReport,
    pub toolchain: ToolchainReport,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PlatformReport {
    pub os: &'static str,
    pub arch: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ToolchainReport {
    pub rustc: Option<String>,
    pub cargo: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DoctorCheck {
    pub capability: &'static str,
    pub name: &'static str,
    pub status: DoctorStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DoctorStatus {
    Pass,
    Warn,
    Fail,
}
