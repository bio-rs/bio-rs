mod annotation;
mod molecule;
mod sequence;
mod structure;
mod table;

use super::FormatCapability;

/// Return the current format support matrix.
pub fn format_capabilities() -> Vec<FormatCapability> {
    let mut capabilities = Vec::new();
    capabilities.extend(sequence::capabilities());
    capabilities.extend(annotation::capabilities());
    capabilities.extend(structure::capabilities());
    capabilities.extend(table::capabilities());
    capabilities.extend(molecule::capabilities());
    capabilities
}
