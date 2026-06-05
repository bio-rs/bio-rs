use crate::errors::CliError;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

pub(super) fn parse_metadata(values: Vec<String>) -> Result<BTreeMap<String, String>, CliError> {
    let mut metadata = BTreeMap::new();
    for value in values {
        let Some((key, val)) = value.split_once('=') else {
            return Err(CliError::Validation {
                code: "dataset.invalid_metadata",
                message: "dataset metadata must use key=value".to_string(),
                location: Some(value),
            });
        };
        let key = key.trim();
        let val = val.trim();
        if key.is_empty() || val.is_empty() {
            return Err(CliError::Validation {
                code: "dataset.invalid_metadata",
                message: "dataset metadata keys and values must be non-empty".to_string(),
                location: Some(value),
            });
        }
        match metadata.entry(key.to_string()) {
            Entry::Vacant(entry) => {
                entry.insert(val.to_string());
            }
            Entry::Occupied(_) => {
                return Err(CliError::Validation {
                    code: "dataset.duplicate_metadata_key",
                    message: format!("dataset metadata key '{key}' was provided more than once"),
                    location: Some(key.to_string()),
                });
            }
        }
    }
    Ok(metadata)
}
