use super::types::{ReportMetric, ReportProvenance, ReportSection, ReportStatus};

pub(super) fn render_markdown(
    title: &str,
    summary: &str,
    status: ReportStatus,
    provenance: &ReportProvenance,
    sections: &[ReportSection],
) -> String {
    let mut out = String::new();
    out.push_str("# ");
    out.push_str(&clean_line(title));
    out.push_str("\n\n");
    out.push_str(&clean_line(summary));
    out.push_str("\n\n");

    out.push_str("## Status\n\n");
    out.push_str("- Status: `");
    out.push_str(status.as_str());
    out.push_str("`\n\n");

    out.push_str("## Reproducibility\n\n");
    push_metric(
        &mut out,
        &ReportMetric::new("bio-rs core version", &provenance.biors_core_version),
    );
    push_metric(
        &mut out,
        &ReportMetric::new("generator", &provenance.generator),
    );
    push_metric(
        &mut out,
        &ReportMetric::new("input container", provenance.input_container.as_str()),
    );
    push_metric(
        &mut out,
        &ReportMetric::new("input kind", provenance.input_kind.as_str()),
    );
    if let Some(schema) = &provenance.source_schema_version {
        push_metric(
            &mut out,
            &ReportMetric::new("source schema version", schema),
        );
    }
    if let Some(version) = &provenance.source_biors_version {
        push_metric(
            &mut out,
            &ReportMetric::new("source bio-rs version", version),
        );
    }
    if let Some(input_hash) = &provenance.source_input_hash {
        push_metric(
            &mut out,
            &ReportMetric::new("source input hash", input_hash),
        );
    }
    push_metric(
        &mut out,
        &ReportMetric::new("input raw SHA-256", &provenance.input_raw_sha256),
    );
    push_metric(
        &mut out,
        &ReportMetric::new(
            "input canonical JSON SHA-256",
            &provenance.input_canonical_sha256,
        ),
    );
    out.push('\n');

    for section in sections {
        out.push_str("## ");
        out.push_str(&clean_line(&section.title));
        out.push_str("\n\n");
        out.push_str(&clean_line(&section.summary));
        out.push_str("\n\n");
        if !section.metrics.is_empty() {
            for metric in &section.metrics {
                push_metric(&mut out, metric);
            }
            out.push('\n');
        }
        if !section.items.is_empty() {
            for item in &section.items {
                out.push_str("- ");
                out.push_str(&clean_line(item));
                out.push('\n');
            }
            out.push('\n');
        }
    }

    out
}

fn push_metric(out: &mut String, metric: &ReportMetric) {
    out.push_str("- ");
    out.push_str(&clean_line(&metric.label));
    out.push_str(": `");
    out.push_str(&clean_line(&metric.value));
    out.push_str("`\n");
}

fn clean_line(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('`', "'")
}
