//! Benchmarking definitions for the lang grammar.

#![no_std]

extern crate alloc;

use common::DynamicBenchmarkSuite;
use core::convert::Infallible;
use fandango_targets::lualang as lang;

/// The [`BenchmarkSuite`] definition for lang.
pub struct Benchmark(Infallible);

impl DynamicBenchmarkSuite for Benchmark {
    const NAME: &'static str = "lualang";

    fn program() -> &'static fandango::lualang::Program<'static> {
        lang::STRUCTURE.inner()
    }
}

#[cfg(feature = "static_defs")]
mod static_defs {
    use crate::Benchmark;
    use common::{BenchmarkSuite, StdGenerator, StdSampler};
    use fandango::generation::Generated;
    use fandango::visitor::{Visitor, VisitorMut};
    use fandango_runtime::measurement::Violations;
    use fandango_runtime::operators::Checker;
    use fandango_targets::lualang as lang;

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

        fn check(item: &Self::Start) -> Violations {
            lang::ConstraintVisitorDefUse::default()
                .visit(item, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .violations()
        }
    }
}
