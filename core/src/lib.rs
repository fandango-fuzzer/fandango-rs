//! Core definitions library for FANDANGO. You almost certainly want the `fandango` crate instead.

extern crate alloc;
extern crate core;

#[macro_use]
#[path = "py_literal/parse_macros.rs"]
mod parse_macros;

#[macro_use]
pub mod graph;
pub mod lang;
pub mod typing;

pub use rand_core;
