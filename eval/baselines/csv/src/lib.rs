#![no_std]

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use common::{BenchmarkSuite, StdGenerator, StdSampler};
use core::convert::Infallible;
use fandango::generation::{Generated, Sampler};
use fandango::typing::Structured;
use fandango::visitor::Visitor;
use fandango_eval::operators::mutate;
use fandango_eval::{Checker, crossover, csv};

pub struct CsvBenchmark(Infallible);

#[allow(deprecated)]
impl BenchmarkSuite<StdSampler, StdGenerator> for CsvBenchmark {
    type Start = csv::nonterminal_start;

    const NAME: &'static str = "csv";

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
        crossover!(
            csv::nonterminal_csv_string_list,
            item,
            other,
            choices,
            sampler
        )
        .unwrap()
    }

    fn program() -> &'static fandango::lang::Program<'static> {
        csv::nonterminal_start::ROOT.inner()
    }
}
