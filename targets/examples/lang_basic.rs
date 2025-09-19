//! This is documentation.

use anyhow::Error;
use fandango::tuple_list::tuple_list;
use fandango::visitor::Visitor;
use fandango::visitor::write::WriteVisitor;
use fandango_runtime::evolvers::Evolver;
use fandango_runtime::evolvers::basic::{BasicEvolver, BasicIndividual};
use fandango_runtime::measurement::{HasFitness, ViolationFitness};
use fandango_runtime::operators::DepthLimiter;
use fandango_runtime::population::Individual;
use fandango_targets::lang;
use num_rational::Ratio;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::time::Instant;

#[allow(deprecated)]
fn main() -> Result<(), Error> {
    let fitness = ViolationFitness::<lang::ConstraintVisitorAtLeastOneVarAlsoDefUse>::new();

    let fixer = ();
    let mut runtime = BasicEvolver::new(fitness, fixer, 1000, 10, 1000, Ratio::new(80, 100))
        .expect("Should be valid.");

    let generator = DepthLimiter::new(lang::STRUCTURE.inner(), 100);
    let mut generators = tuple_list!(generator);
    let mut sampler = StdRng::from_os_rng();

    let mut population: Vec<BasicIndividual<lang::nonterminal_start, _>> =
        runtime.initial(&mut generators, &mut sampler)?;

    let start = Instant::now();
    for i in 0..100 {
        let fitness = population
            .iter()
            .map(|i| i.measurement().fitness())
            .fold(0.0f64, |v, r| v + *r.numer() as f64 / *r.denom() as f64)
            / population.len() as f64;
        if fitness == 1.0 {
            break;
        }
        println!("average fitness at generation {i}: {fitness}");
        population = runtime.step(&mut generators, &mut sampler, population)?;
    }
    let elapsed = start.elapsed();

    println!("Population after {:.2}:", elapsed.as_secs_f64());
    for (i, candidate) in population.into_iter().enumerate() {
        println!(
            "{i}~~~~~~~~~~~~:\n{}",
            String::from_utf8(
                WriteVisitor::new(Vec::new())
                    .visit(candidate.node(), 0)?
                    .continue_value()
                    .unwrap()
                    .output()
            )?
        );
    }

    Ok(())
}
