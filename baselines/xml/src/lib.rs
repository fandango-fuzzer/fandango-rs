//! Benchmarking definitions for the XML grammar.

#![no_std]

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use common::{BenchmarkSuite, StdGenerator, StdSampler};
use core::convert::Infallible;
use fandango::dynamic::{DynamicNode, DynamicSampler};
use fandango::generation::Generated;
use fandango::typing::{AsStaticNode, Structured};
use fandango::visitor::Visitor;
use fandango_targets::operators::mutate;
use fandango_targets::{Checker, crossover, xml};

/// The [`BenchmarkSuite`] definition for XML.
pub struct Benchmark(Infallible);

impl BenchmarkSuite<StdSampler, StdGenerator> for Benchmark {
    type Start = xml::nonterminal_start;

    const NAME: &'static str = "xml";

    fn generate(sampler: &mut StdSampler, generator: &mut StdGenerator) -> Self::Start {
        xml::nonterminal_start::generate(sampler, generator, 0)
    }

    fn fix(item: &mut Self::Start, sampler: &mut StdSampler, generator: &mut StdGenerator) {
        xml::ConstraintFixer::evaluated(sampler, generator)
            .visit(item, 0)
            .unwrap()
            .continue_value()
            .unwrap();
    }

    fn check(item: &mut Self::Start) -> Vec<VecDeque<usize>> {
        xml::ConstraintVisitor::evaluated()
            .visit(item, 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .violations()
    }

    fn mutate(
        item: &mut Self::Start,
        choices: &mut Vec<VecDeque<usize>>,
        sampler: &mut StdSampler,
        generator: &mut StdGenerator,
    ) -> bool {
        mutate(item, choices, sampler, generator).unwrap().is_some()
    }

    fn crossover(
        item: &mut Self::Start,
        other: &mut Self::Start,
        choices: &mut Vec<VecDeque<usize>>,
        sampler: &mut StdSampler,
    ) -> bool {
        crossover!(xml::nonterminal_id, item, other, choices, sampler).unwrap()
    }

    fn crossover_dynamic(
        item: &mut DynamicNode,
        other: &mut DynamicNode,
        choices: &mut Vec<VecDeque<usize>>,
        sampler: &mut DynamicSampler<StdSampler>,
    ) -> bool {
        crossover!(
            dynamic xml::nonterminal_id::static_definition(),
            item,
            other,
            choices,
            sampler
        )
        .unwrap()
    }

    fn program() -> &'static fandango::lang::Program<'static> {
        xml::nonterminal_start::ROOT.inner()
    }
}
