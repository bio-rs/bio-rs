pub mod args;
mod doctor;
pub mod handlers;
pub use args::{Cli, Command, FastaCommand, KindArg, PackageCommand, PaddingArg, SeqCommand};
pub(crate) use doctor::build_doctor_report;
pub use handlers::run;
