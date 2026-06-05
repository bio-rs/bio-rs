use biors_core::package::PackageManifest;
use serde_json::Value;

mod common;

#[test]
fn package_manifest_deserialization_rejects_unknown_fields() {
    for (field_path, mutate) in [
        ("unexpected_top", add_top_unknown as fn(&mut Value)),
        ("model.unexpected_model", add_model_unknown),
        ("metadata.license.unexpected_license", add_metadata_unknown),
        ("fixtures[0].unexpected_fixture", add_fixture_unknown),
        ("runtime.unexpected_runtime", add_runtime_unknown),
        (
            "preprocessing[0].unexpected_step",
            add_pipeline_step_unknown,
        ),
    ] {
        let mut value = serde_json::to_value(common::example_manifest()).expect("manifest value");
        mutate(&mut value);

        let error = serde_json::from_value::<PackageManifest>(value).expect_err(field_path);
        assert!(
            error.to_string().contains("unknown field"),
            "{field_path}: {error}"
        );
    }
}

fn add_top_unknown(value: &mut Value) {
    value["unexpected_top"] = Value::Bool(true);
}

fn add_model_unknown(value: &mut Value) {
    value["model"]["unexpected_model"] = Value::Bool(true);
}

fn add_metadata_unknown(value: &mut Value) {
    value["metadata"]["license"]["unexpected_license"] = Value::Bool(true);
}

fn add_fixture_unknown(value: &mut Value) {
    value["fixtures"][0]["unexpected_fixture"] = Value::Bool(true);
}

fn add_runtime_unknown(value: &mut Value) {
    value["runtime"]["unexpected_runtime"] = Value::Bool(true);
}

fn add_pipeline_step_unknown(value: &mut Value) {
    value["preprocessing"][0]["unexpected_step"] = Value::Bool(true);
}
