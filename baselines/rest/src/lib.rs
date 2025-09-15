//! Benchmarking definitions for the REST grammar.

#![no_std]

extern crate alloc;

use common::DynamicBenchmarkSuite;
use core::convert::Infallible;
use fandango_targets::rest;

/// The [`BenchmarkSuite`] definition for REST.
pub struct Benchmark(Infallible);

impl DynamicBenchmarkSuite for Benchmark {
    const NAME: &'static str = "rest";

    fn program() -> &'static fandango::lang::Program<'static> {
        rest::STRUCTURE.inner()
    }
}

#[cfg(feature = "static_defs")]
mod static_defs {
    use crate::Benchmark;
    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use common::{BenchmarkSuite, StdGenerator, StdSampler};
    use fandango::generation::Generated;
    use fandango::visitor::Visitor;
    use fandango_targets::{Checker, crossover, rest};

    impl BenchmarkSuite<StdSampler, StdGenerator> for Benchmark {
        type Start = rest::nonterminal_start;

        fn generate(sampler: &mut StdSampler, generator: &mut StdGenerator) -> Self::Start {
            rest::nonterminal_start::generate(sampler, generator, 0)
        }

        fn fix(item: &mut Self::Start, _sampler: &mut StdSampler, _generator: &mut StdGenerator) {
            rest::ConstraintFixer::evaluated()
                .visit(item, 0)
                .unwrap()
                .continue_value()
                .unwrap();
        }

        fn check(item: &Self::Start) -> Vec<VecDeque<usize>> {
            rest::ConstraintVisitor::evaluated()
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
