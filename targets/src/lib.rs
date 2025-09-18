//! Constraint implementations for various grammars used in the evaluation.
//!
//! Grammars known to have incorrect constraints are marked as deprecated to prevent accidental
//! misuse. Corrected versions of these grammars are provided according to the used constructor.

#![no_std]

extern crate alloc;

#[cfg(feature = "csv")]
pub mod csv;
#[cfg(feature = "rest")]
pub mod rest;
#[cfg(feature = "scriptsizec")]
pub mod scriptsizec;
#[cfg(feature = "xml")]
pub mod xml;
