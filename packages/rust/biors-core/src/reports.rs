//! Reproducible report generation for bio-rs JSON outputs.

mod builder;
mod markdown;
mod sections;
mod types;

pub use builder::build_shareable_report_from_json_bytes;
pub use types::{
    ReportBuildError, ReportInputContainer, ReportInputKind, ReportMetric, ReportProvenance,
    ReportSection, ReportStatus, ShareableReport, REPORT_SCHEMA_VERSION,
};
