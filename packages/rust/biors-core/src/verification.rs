//! Package fixture verification, stable input hashing, and content diff reporting.

mod diff;
mod fixtures;
mod hash;
mod types;

pub use fixtures::{verify_package_outputs, verify_package_outputs_with_observation_base};
pub use hash::{stable_input_hash, StableInputHasher};
pub use types::*;
