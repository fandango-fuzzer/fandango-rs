//! Benchmarking definitions for the lang grammar.

#![no_std]

extern crate alloc;

use common::DynamicBenchmarkSuite;
use core::convert::Infallible;
use fandango_targets::lang;

/// The [`BenchmarkSuite`] definition for lang.
pub struct Benchmark(Infallible);

impl DynamicBenchmarkSuite for Benchmark {
    const NAME: &'static str = "lang";

    fn program() -> &'static fandango::lang::Program<'static> {
        lang::STRUCTURE.inner()
    }
}

#[cfg(feature = "static_defs")]
mod static_defs {
    use crate::Benchmark;
    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use common::{BenchmarkSuite, StdGenerator, StdSampler};
    use fandango::generation::Generated;
    use fandango::visitor::{Visitor, VisitorMut};
    use fandango_targets::{Checker, crossover, lang};

    impl BenchmarkSuite<StdSampler, StdGenerator> for Benchmark {
        type Start = lang::nonterminal_start;

        fn generate(sampler: &mut StdSampler, generator: &mut StdGenerator) -> Self::Start {
            lang::nonterminal_start::generate(sampler, generator, 0)
        }

        fn fix(item: &mut Self::Start, sampler: &mut StdSampler, generator: &mut StdGenerator) {
            lang::ConstraintFixerDefUse{
                sampler: sampler,
                generator: generator,
                defined_vars: &mut alloc::collections::BTreeMap::new(),
            }
                .visit_mut(item, 0)
                .unwrap()
                .continue_value()
                .unwrap();
        }

        fn check(item: &Self::Start) -> Vec<VecDeque<usize>> {
            lang::ConstraintVisitorDefUse::corrected()
                .visit(item, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .violations()
        }

        fn crossover(
            item: &mut Self::Start,
            other: &Self::Start,
            mut choice: VecDeque<usize>,
            sampler: &mut StdSampler,
        ) -> bool {
            crossover!(item, other, choice, sampler).unwrap()
        }
    }
}
