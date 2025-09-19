//! Benchmarking definitions for the XML grammar.

#![no_std]

extern crate alloc;

use common::DynamicBenchmarkSuite;
use core::convert::Infallible;
use fandango_runtime::operators::Checker;
use fandango_targets::xml;

/// The [`BenchmarkSuite`] definition for XML.
pub struct Benchmark(Infallible);

impl DynamicBenchmarkSuite for Benchmark {
    const NAME: &'static str = "xml";

    fn program() -> &'static fandango::lang::Program<'static> {
        xml::STRUCTURE.inner()
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
    use fandango_runtime::measurement::Violations;
    use fandango_runtime::operators::Checker;
    use fandango_targets::xml;

    impl BenchmarkSuite<StdSampler, StdGenerator> for Benchmark {
        type Start = xml::nonterminal_start;

        fn generate(sampler: &mut StdSampler, generator: &mut StdGenerator) -> Self::Start {
            xml::nonterminal_start::generate(sampler, generator, 0)
        }

        fn fix(item: &mut Self::Start, sampler: &mut StdSampler, generator: &mut StdGenerator) {
            xml::ConstraintFixer::evaluated(sampler, generator)
                .visit_mut(item, 0)
                .unwrap()
                .continue_value()
                .unwrap();
        }

        fn check(item: &Self::Start) -> Violations {
            xml::ConstraintVisitor::evaluated()
                .visit(item, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .violations()
        }
    }
}