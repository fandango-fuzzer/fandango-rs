//! `#[derive]` functionality for FANDANGO, to be used as a re-export from the main `fandango`
//! crate.

use fandango_generator::{FandangoDerivation, derive_fandango_or_emit_error};
use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Perform the `#[derive]`!
#[proc_macro_derive(Fandango, attributes(grammar))]
pub fn derive_fandango(item: TokenStream) -> TokenStream {
    let source = parse_macro_input!(item as FandangoDerivation);
    match derive_fandango_or_emit_error(source) {
        Ok(s) | Err(s) => s.into(),
    }
}
