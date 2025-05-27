//! Benchmarking definitions for the ScriptSizeC grammar.

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
use fandango_targets::{Checker, crossover, scriptsizec};

/// The [`BenchmarkSuite`] definition for ScriptSizeC.
pub struct Benchmark(Infallible);

impl BenchmarkSuite<StdSampler, StdGenerator> for Benchmark {
    type Start = scriptsizec::nonterminal_start;

    const NAME: &'static str = "scriptsizec";

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

    fn check(item: &mut Self::Start) -> Vec<VecDeque<usize>> {
        scriptsizec::ConstraintVisitor::evaluated()
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
        crossover!(scriptsizec::nonterminal_id, item, other, choices, sampler).unwrap()
    }

    fn crossover_dynamic(
        item: &mut DynamicNode,
        other: &mut DynamicNode,
        choices: &mut Vec<VecDeque<usize>>,
        sampler: &mut DynamicSampler<StdSampler>,
    ) -> bool {
        crossover!(
            dynamic scriptsizec::nonterminal_id::static_definition(),
            item,
            other,
            choices,
            sampler
        )
        .unwrap()
    }

    fn program() -> &'static fandango::lang::Program<'static> {
        scriptsizec::nonterminal_start::ROOT.inner()
    }
}
