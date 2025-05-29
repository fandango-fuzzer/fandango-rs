//! Benchmarking definitions for the CSV grammar.

#![no_std]

extern crate alloc;

use common::DynamicBenchmarkSuite;
use core::convert::Infallible;
use fandango_targets::csv;

/// The [`BenchmarkSuite`] definition for CSV.
pub struct Benchmark(Infallible);

impl DynamicBenchmarkSuite for Benchmark {
    const NAME: &'static str = "csv";

    fn program() -> &'static fandango::lang::Program<'static> {
        csv::STRUCTURE.inner()
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
    use fandango_targets::{Checker, crossover, csv};

    impl BenchmarkSuite<StdSampler, StdGenerator> for Benchmark {
        type Start = csv::nonterminal_start;

        fn generate(sampler: &mut StdSampler, generator: &mut StdGenerator) -> Self::Start {
            csv::nonterminal_start::generate(sampler, generator, 0)
        }

        fn fix(item: &mut Self::Start, sampler: &mut StdSampler, generator: &mut StdGenerator) {
            csv::ConstraintFixer::evaluated(sampler, generator)
                .visit(item, 0)
                .unwrap()
                .continue_value()
                .unwrap();
        }

        fn check(item: &mut Self::Start) -> Vec<VecDeque<usize>> {
            csv::ConstraintVisitor::evaluated()
                .visit(item, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .violations()
        }

        fn crossover(
            item: &mut Self::Start,
            other: &mut Self::Start,
            mut choice: VecDeque<usize>,
            sampler: &mut StdSampler,
        ) -> bool {
            crossover!(item, other, choice, sampler).unwrap()
        }
    }
}
