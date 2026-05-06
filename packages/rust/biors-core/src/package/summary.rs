use super::{PackageLayoutSummary, PackageManifest, PackageManifestSummary};

/// Build a compact summary of a package manifest for inspect-style output.
pub fn inspect_package_manifest(manifest: &PackageManifest) -> PackageManifestSummary {
    PackageManifestSummary {
        schema_version: manifest.schema_version,
        name: manifest.name.clone(),
        model_format: manifest.model.format,
        has_model_checksum: manifest.model.checksum.is_some(),
        tokenizer: manifest
            .tokenizer
            .as_ref()
            .map(|tokenizer| tokenizer.name.clone()),
        vocab: manifest.vocab.as_ref().map(|vocab| vocab.name.clone()),
        runtime_backend: manifest.runtime.backend,
        runtime_target: manifest.runtime.target,
        preprocessing_steps: manifest.preprocessing.len(),
        postprocessing_steps: manifest.postprocessing.len(),
        fixtures: manifest.fixtures.len(),
        layout: PackageLayoutSummary {
            model: manifest.model.path.clone(),
            tokenizer: manifest
                .tokenizer
                .as_ref()
                .map(|tokenizer| tokenizer.path.clone()),
            vocab: manifest.vocab.as_ref().map(|vocab| vocab.path.clone()),
            fixture_inputs: manifest
                .fixtures
                .iter()
                .map(|fixture| fixture.input.clone())
                .collect(),
            fixture_outputs: manifest
                .fixtures
                .iter()
                .map(|fixture| fixture.expected_output.clone())
                .collect(),
        },
    }
}
