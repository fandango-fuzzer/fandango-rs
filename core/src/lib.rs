//! Core definitions library for FANDANGO. You almost certainly want the `fandango` crate instead.

#[macro_use]
#[path = "py_literal/parse_macros.rs"]
mod parse_macros;

#[path = "libafl/type_eq.rs"]
mod type_eq;

#[macro_use]
pub mod graph;
pub mod generation;
pub mod lang;
pub mod typing;
pub mod visitor;
