//! Runtimes which enable the generation of inputs with [`fandango`] using evolutionary algorithms

#![no_std]

extern crate alloc;

pub mod evolvers;
pub mod measurement;
pub mod operators;
pub mod population;
