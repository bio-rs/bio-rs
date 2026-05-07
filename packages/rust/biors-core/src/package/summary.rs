use super::{
    PackageDirectoryLayout, PackageDirectoryLayoutSummary, PackageLayoutSummary, PackageManifest,
    PackageManifestSummary, PackageMetadata, PackageMetadataSummary,
};

/// Build a compact summary of a package manifest for inspect-style output.
pub fn inspect_package_manifest(manifest: &PackageManifest) -> PackageManifestSummary {
    PackageManifestSummary {
        schema_version: manifest.schema_version,
        name: manifest.name.clone(),
        package_layout: manifest.package_layout.as_ref().map(package_layout_summary),
        metadata: manifest.metadata.as_ref().map(metadata_summary),
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

fn package_layout_summary(layout: &PackageDirectoryLayout) -> PackageDirectoryLayoutSummary {
    PackageDirectoryLayoutSummary {
        manifest: layout.manifest.clone(),
        models: layout.models.clone(),
        tokenizers: layout.tokenizers.clone(),
        vocabs: layout.vocabs.clone(),
        fixtures: layout.fixtures.clone(),
        observed: layout.observed.clone(),
        docs: layout.docs.clone(),
    }
}

fn metadata_summary(metadata: &PackageMetadata) -> PackageMetadataSummary {
    PackageMetadataSummary {
        license: metadata.license.expression.clone(),
        citation: metadata.citation.preferred_citation.clone(),
        model_card: metadata.model_card.path.clone(),
    }
}
