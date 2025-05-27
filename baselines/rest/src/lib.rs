//! Benchmarking definitions for the REST grammar.

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
use fandango_targets::{Checker, crossover, rest};

/// The [`BenchmarkSuite`] definition for REST.
pub struct Benchmark(Infallible);

impl BenchmarkSuite<StdSampler, StdGenerator> for Benchmark {
    type Start = rest::nonterminal_start;

    const NAME: &'static str = "rest";

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

    fn check(item: &mut Self::Start) -> Vec<VecDeque<usize>> {
        rest::ConstraintVisitor::evaluated()
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
        crossover!(rest::nonterminal_id, item, other, choices, sampler).unwrap()
            || crossover!(rest::nonterminal_underline, item, other, choices, sampler).unwrap()
    }

    fn crossover_dynamic(
        item: &mut DynamicNode,
        other: &mut DynamicNode,
        choices: &mut Vec<VecDeque<usize>>,
        sampler: &mut DynamicSampler<StdSampler>,
    ) -> bool {
        crossover!(
            dynamic rest::nonterminal_id::static_definition(),
            item,
            other,
            choices,
            sampler
        )
        .unwrap()
            || crossover!(dynamic rest::nonterminal_underline::static_definition(), item, other, choices, sampler).unwrap()
    }

    fn program() -> &'static fandango::lang::Program<'static> {
        rest::nonterminal_start::ROOT.inner()
    }
}
