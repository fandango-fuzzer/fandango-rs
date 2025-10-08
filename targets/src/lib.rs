//! Constraint implementations for various grammars used in the evaluation.
//!
//! Grammars known to have incorrect constraints are marked as deprecated to prevent accidental
//! misuse. Corrected versions of these grammars are provided according to the used constructor.

#![no_std]

extern crate alloc;

#[allow(unused)]
macro_rules! maybe_deref {
    ($value: ident) => {
        loop {
            #[cfg(no_opt_indirect)]
            {
                break &**$value;
            }
            #[cfg(not(no_opt_indirect))]
            {
                break $value;
            }
        }
    };
}

#[allow(unused)]
macro_rules! maybe_deref_mut {
    ($value: ident) => {
        loop {
            #[cfg(no_opt_indirect)]
            {
                break &mut **$value;
            }
            #[cfg(not(no_opt_indirect))]
            {
                break $value;
            }
        }
    };
}

/// The module for c language testing.
#[cfg(feature = "clang")]
pub mod clang;
#[cfg(feature = "csv")]
pub mod csv;
#[cfg(feature = "rest")]
pub mod rest;
#[cfg(feature = "scriptsizec")]
pub mod scriptsizec;
#[cfg(feature = "xml")]
pub mod xml;
