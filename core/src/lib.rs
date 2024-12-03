//! Core definitions library for FANDANGO. You almost certainly want the `fandango` crate instead.

#![allow(bindings_with_variant_name)] // old school macros in py_literal
#![warn(missing_docs)]

extern crate alloc;
extern crate core;

#[macro_use]
#[path = "py_literal/parse_macros.rs"]
mod parse_macros;

#[macro_use]
pub mod graph;
pub mod lang;
pub mod typing;
