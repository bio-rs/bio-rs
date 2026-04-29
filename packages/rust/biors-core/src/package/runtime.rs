use super::{validate_package_manifest, PackageManifest, RuntimeBridgeReport};

/// Build a runtime bridge readiness report from a package manifest.
pub fn plan_runtime_bridge(manifest: &PackageManifest) -> RuntimeBridgeReport {
    let blocking_issues = validate_package_manifest(manifest).issues;

    RuntimeBridgeReport {
        ready: blocking_issues.is_empty(),
        backend: manifest.runtime.backend,
        target: manifest.runtime.target,
        execution_provider: "webgpu".to_string(),
        blocking_issues,
    }
}
