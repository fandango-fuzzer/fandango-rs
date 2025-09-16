//! Benchmarking definitions for the ScriptSizeC grammar.

#![no_std]

extern crate alloc;

use common::DynamicBenchmarkSuite;
use core::convert::Infallible;
use fandango_targets::scriptsizec;

/// The [`BenchmarkSuite`] definition for ScriptSizeC.
pub struct Benchmark(Infallible);

impl DynamicBenchmarkSuite for Benchmark {
    const NAME: &'static str = "scriptsizec";

    fn program() -> &'static fandango::lang::Program<'static> {
        scriptsizec::STRUCTURE.inner()
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
    use fandango_targets::{Checker, scriptsizec};

    impl BenchmarkSuite<StdSampler, StdGenerator> for Benchmark {
        type Start = scriptsizec::nonterminal_start;

        fn generate(sampler: &mut StdSampler, generator: &mut StdGenerator) -> Self::Start {
            scriptsizec::nonterminal_start::generate(sampler, generator, 0)
        }

        fn fix(item: &mut Self::Start, _sampler: &mut StdSampler, _generator: &mut StdGenerator) {
            scriptsizec::ConstraintFixer::evaluated()
                .visit(item, 0)
                .unwrap()
                .continue_value()
                .unwrap();
        }

        fn check(item: &Self::Start) -> Vec<VecDeque<usize>> {
            scriptsizec::ConstraintVisitor::evaluated()
                .visit(item, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .violations()
        }
    }
}
