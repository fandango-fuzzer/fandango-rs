//! Core definitions library for FANDANGO. You almost certainly want the `fandango` crate instead.

#![no_std]

extern crate alloc;

#[macro_use]
#[path = "py_literal/parse_macros.rs"]
mod parse_macros;

#[macro_use]
#[cfg(feature = "petgraph")]
pub mod graph;
// pub mod constraint;
pub mod dynamic;
pub mod generation;
pub mod lang;
pub mod typing;
pub mod visitor;
