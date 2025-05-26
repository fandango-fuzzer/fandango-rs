//! Constraint implementations for various grammars used in the evaluation.
//!
//! Grammars known to have incorrect constraints are marked as deprecated to prevent accidental
//! misuse. Corrected versions of these grammars are provided according to the used constructor.

#![no_std]

extern crate alloc;

pub mod operators;

pub mod csv;
mod rest;
pub mod scriptsizec;
pub mod xml;
