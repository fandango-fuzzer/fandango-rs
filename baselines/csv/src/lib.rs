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
#[expect(deprecated)]
mod static_defs {
    use crate::Benchmark;
    use common::{BenchmarkSuite, StdGenerator, StdSampler};
    use fandango::generation::Generated;
    use fandango::visitor::{Visitor, VisitorMut};
    use fandango_runtime::measurement::Violations;
    use fandango_runtime::operators::Checker;
    use fandango_targets::csv;

    impl BenchmarkSuite<StdSampler, StdGenerator> for Benchmark {
        type Start = csv::nonterminal_start;

        fn generate(sampler: &mut StdSampler, generator: &mut StdGenerator) -> Self::Start {
            csv::nonterminal_start::generate(sampler, generator, 0)
        }

        fn fix(item: &mut Self::Start, sampler: &mut StdSampler, generator: &mut StdGenerator) {
            csv::ConstraintFixer::evaluated(sampler, generator)
                .visit_mut(item, 0)
                .unwrap()
                .continue_value()
                .unwrap();
        }

        fn check(item: &Self::Start) -> Violations {
            csv::ConstraintVisitor::evaluated()
                .visit(item, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .violations()
        }
    }
}
