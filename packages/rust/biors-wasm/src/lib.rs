use wasm_bindgen::prelude::*;

mod fasta;
mod model_input;
mod tokenize;
mod types;
mod workflow;

pub use fasta::*;
pub use model_input::*;
pub use tokenize::*;
pub use workflow::*;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
