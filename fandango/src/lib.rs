//! # Fandango
//!
//! This crate replicates and extends the functionality of [FANDANGO](https://github.com/fandango-fuzzer/fandango).
//!
//! ## Development timeline
//!
//! 1. Definition and reimplementation of FANDANGO grammar parsing. (02.12.)
//! 2. Transpilation of FANDANGO grammar into Rust source code. (05.12.)
//! 3. Parsing and implementation of Python generators. (10.12.)
//! 4. Implementation of Rust-native generators. (12.12.)

pub use fandango_core::*;
pub use fandango_macros::*;
