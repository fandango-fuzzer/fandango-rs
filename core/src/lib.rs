//! Core definitions library for FANDANGO. You almost certainly want the `fandango` crate instead.

#[macro_use]
#[path = "py_literal/parse_macros.rs"]
mod parse_macros;

#[macro_use]
pub mod graph;
pub mod lang;
pub mod typing;
pub mod visitor;

pub use rand_core;
