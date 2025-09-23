//! This example demonstrates how to use the NSGA2 evolver

use anyhow::Error;
use fandango::tuple_list::tuple_list;
use fandango::visitor::Visitor;
use fandango::visitor::write::WriteVisitor;
use fandango_runtime::evolvers::Evolver;
use fandango_runtime::evolvers::multi::{KPathDiversityHook, Nsga2Evolver};
use fandango_runtime::measurement::{HasFitness, ViolationFitness};
use fandango_runtime::measurement::{HasMeasurement, SizeFitness};
use fandango_runtime::operators::DepthLimiter;
use fandango_runtime::population::Individual;
use fandango_targets::xml;
use num_rational::Ratio;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::num::NonZeroUsize;

#[allow(deprecated)]
fn main() -> Result<(), Error> {
    let fitness = ViolationFitness::<xml::ConstraintVisitor>::new();
    let size = SizeFitness;
    // let fixer = XmlFixHook::evaluated();
    let fixer = ();
    let hook = KPathDiversityHook::new(fixer, NonZeroUsize::new(5).unwrap());
    let mut runtime = Nsga2Evolver::new::<xml::nonterminal_start>(
        tuple_list!(fitness, size),
        hook,
        100,
        1000,
        Ratio::new(80, 100),
    )
    .expect("Should be valid.");

    let generator = DepthLimiter::new(xml::STRUCTURE.inner(), 100);
    let mut generators = tuple_list!(generator);
    let mut sampler = StdRng::from_os_rng();

    let mut population = runtime.initial(&mut generators, &mut sampler)?;

    for i in 0..100 {
        let fitness = population
            .iter()
            .map(|i| i.measurement().fitness())
            .fold(0.0f64, |v, r| v + *r.0.numer() as f64 / *r.0.denom() as f64)
            / population.len() as f64;
        if fitness == 1.0 {
            println!("saturated fitness at generation {i}");
            break;
        }
        println!("average fitness at generation {i}: {fitness}");
        population = runtime.step(&mut generators, &mut sampler, population)?;
    }

    population.sort_by(|i1, i2| i1.node().cmp(i2.node()));
    population.dedup_by(|i1, i2| i1.node() == i2.node());

    println!("Population:");
    for (i, candidate) in population.into_iter().enumerate() {
        println!(
            "{i}: {}",
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
