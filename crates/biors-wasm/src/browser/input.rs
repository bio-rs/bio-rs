use super::types::{
    BrowserFileDescriptor, BrowserFileWarning, MAX_BROWSER_INPUT_BYTES, WARN_BROWSER_INPUT_BYTES,
};
use biors_core::hash::sha256_bytes_digest;
use biors_core::sequence::{SequenceKind, SequenceKindSelection};
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub(super) struct BrowserFileInput {
    pub(super) name: Option<String>,
    pub(super) format: BrowserFileFormat,
    pub(super) bytes: Vec<u8>,
    pub(super) kind: Option<String>,
    pub(super) profile: Option<String>,
}

impl BrowserFileInput {
    pub(super) fn from_js(input: &JsValue) -> Result<Self, JsValue> {
        let name = get_string_opt(input, "name")?;
        let bytes = get_bytes(input, "bytes")?;
        let format = BrowserFileFormat::resolve(get_string_opt(input, "format")?, name.as_deref())?;
        let kind = get_string_opt(input, "kind")?;
        let profile = get_string_opt(input, "profile")?;
        Ok(Self {
            name,
            format,
            bytes,
            kind,
            profile,
        })
    }

    pub(super) fn ensure_size(&self) -> Result<(), JsValue> {
        if self.bytes.len() > MAX_BROWSER_INPUT_BYTES {
            return Err(JsValue::from_str(&format!(
                "browser input is {} bytes; maximum supported size is {} bytes",
                self.bytes.len(),
                MAX_BROWSER_INPUT_BYTES
            )));
        }
        Ok(())
    }

    pub(super) fn descriptor(&self, input_hash: Option<String>) -> BrowserFileDescriptor {
        BrowserFileDescriptor {
            name: self.name.clone(),
            format: self.format.as_str(),
            size_bytes: self.bytes.len(),
            content_sha256: sha256_bytes_digest(&self.bytes),
            input_hash,
        }
    }

    pub(super) fn memory_warnings(&self) -> Vec<BrowserFileWarning> {
        if self.bytes.len() >= WARN_BROWSER_INPUT_BYTES {
            vec![BrowserFileWarning {
                code: "browser_input_large",
                message: "input is accepted, but browser memory pressure may be high",
            }]
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BrowserFileFormat {
    Fasta,
    Fastq,
    Pdb,
    Smiles,
}

impl BrowserFileFormat {
    fn resolve(format: Option<String>, name: Option<&str>) -> Result<Self, JsValue> {
        if let Some(format) = format {
            return Self::parse(&format);
        }

        let Some(name) = name else {
            return Err(JsValue::from_str(
                "format is required when file name is unavailable",
            ));
        };
        Self::infer_from_name(name).ok_or_else(|| {
            JsValue::from_str("could not infer format from file name; pass format explicitly")
        })
    }

    fn parse(format: &str) -> Result<Self, JsValue> {
        match format {
            "fasta" => Ok(Self::Fasta),
            "fastq" => Ok(Self::Fastq),
            "pdb" => Ok(Self::Pdb),
            "smiles" => Ok(Self::Smiles),
            _ => Err(JsValue::from_str(
                "format must be one of: fasta, fastq, pdb, smiles",
            )),
        }
    }

    fn infer_from_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        if [".fasta", ".fa", ".faa", ".fna"]
            .iter()
            .any(|ext| lower.ends_with(ext))
        {
            Some(Self::Fasta)
        } else if [".fastq", ".fq"].iter().any(|ext| lower.ends_with(ext)) {
            Some(Self::Fastq)
        } else if lower.ends_with(".pdb") {
            Some(Self::Pdb)
        } else if [".smi", ".smiles"].iter().any(|ext| lower.ends_with(ext)) {
            Some(Self::Smiles)
        } else {
            None
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Fasta => "fasta",
            Self::Fastq => "fastq",
            Self::Pdb => "pdb",
            Self::Smiles => "smiles",
        }
    }
}

pub(super) fn sequence_kind_selection(
    kind: Option<&str>,
) -> Result<SequenceKindSelection, JsValue> {
    match kind.unwrap_or("auto") {
        "auto" => Ok(SequenceKindSelection::Auto),
        "protein" => Ok(SequenceKindSelection::Explicit(SequenceKind::Protein)),
        "dna" => Ok(SequenceKindSelection::Explicit(SequenceKind::Dna)),
        "rna" => Ok(SequenceKindSelection::Explicit(SequenceKind::Rna)),
        _ => Err(JsValue::from_str(
            "kind must be one of: auto, protein, dna, rna",
        )),
    }
}

fn get_bytes(obj: &JsValue, key: &str) -> Result<Vec<u8>, JsValue> {
    let val = js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .map_err(|_| JsValue::from_str(&format!("missing field: {key}")))?;
    let arr = val
        .dyn_into::<js_sys::Uint8Array>()
        .map_err(|_| JsValue::from_str(&format!("field {key} must be a Uint8Array")))?;
    Ok(arr.to_vec())
}

fn get_string_opt(obj: &JsValue, key: &str) -> Result<Option<String>, JsValue> {
    let val = js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .map_err(|_| JsValue::from_str(&format!("field {key} must be a string")))?;
    if val.is_undefined() || val.is_null() {
        return Ok(None);
    }
    val.as_string()
        .map(Some)
        .ok_or_else(|| JsValue::from_str(&format!("field {key} must be a string")))
}
