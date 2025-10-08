//! This example demonstrates how to use the NSGA2 evolver and defining one's own fitness

use anyhow::Error;
use fandango::tuple_list::tuple_list;
use fandango::typing::{Node, StaticDiscriminable};
use fandango::visitor::Visitor;
use fandango::visitor::navigation::{CountNodes};
use fandango::visitor::write::WriteVisitor;
use fandango_runtime::evolvers::Evolver;
use fandango_runtime::evolvers::multi::{KPathDiversityHook, Nsga2Evolver};
use fandango_runtime::measurement::HasMeasurement;
use fandango_runtime::measurement::{FitnessMeasurer, HasFitness, ViolationFitness};
use fandango_runtime::operators::{DepthLimiter, NodeScan};
use fandango_runtime::population::Individual;
use fandango_targets::clang::{self};
use num_rational::Ratio;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::cmp::Reverse;
use std::convert::Infallible;
use std::num::NonZeroUsize;

struct NodeGoal {
    n: usize,
}

impl<'a, N> FitnessMeasurer<'a, N> for NodeGoal
where
    N: Node,
{
    type Measurement = Reverse<usize>;
    type Error = Infallible;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurement, Self::Error> {
        Ok(Reverse(self.n.abs_diff(node.count_nodes())))
    }
}

// Up to n struct definitions
struct StructGoal {
    n: usize,
}

impl<'a, N> FitnessMeasurer<'a, N> for StructGoal
where
    N: Node,
{
    type Measurement = Reverse<usize>;
    type Error = Infallible;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurement, Self::Error> {
        Ok(Reverse(NodeScan::new(clang::nonterminal_struct_def::DISCRIMINANT as usize).visit(node, 0).unwrap().continue_value().unwrap().matches().len().saturating_sub(self.n)))
    }
}

// Up to n struct fields
struct StructFieldGoal {
    n: usize,
}

impl<'a, N> FitnessMeasurer<'a, N> for StructFieldGoal
where
    N: Node,
{
    type Measurement = Reverse<usize>;
    type Error = Infallible;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurement, Self::Error> {
        Ok(Reverse(NodeScan::new(clang::nonterminal_field_name::DISCRIMINANT as usize).visit(node, 0).unwrap().continue_value().unwrap().matches().len().saturating_sub(self.n)))
    }
}

// Up to n function definitions
struct FnGoal {
    n: usize,
}

impl<'a, N> FitnessMeasurer<'a, N> for FnGoal
where
    N: Node,
{
    type Measurement = Reverse<usize>;
    type Error = Infallible;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurement, Self::Error> {
        Ok(Reverse(NodeScan::new(clang::nonterminal_fn_def::DISCRIMINANT as usize).visit(node, 0).unwrap().continue_value().unwrap().matches().len().saturating_sub(self.n)))
    }
}

// More than n expressions
struct ExprGoal {
    n: usize,
}

impl<'a, N> FitnessMeasurer<'a, N> for ExprGoal
where
    N: Node,
{
    type Measurement = Reverse<usize>;
    type Error = Infallible;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurement, Self::Error> {
        Ok(Reverse(self.n.saturating_sub(NodeScan::new(clang::nonterminal_expr::DISCRIMINANT as usize).visit(node, 0).unwrap().continue_value().unwrap().matches().len())))
    }
}

fn run_once() -> Result<(), Error> {
    let fitness = ViolationFitness::<clang::CombinedConstraintVisitor>::new();
    let nodes = NodeGoal { n: 1000 };
    let structs = StructGoal { n: 1 };
    let fns = FnGoal { n: 1 };
    let fields = StructFieldGoal { n: 5 };
    let exprs = ExprGoal { n: 20 };
    // let fixer = XmlFixHook::evaluated();
    let fixer = ();
    let hook = KPathDiversityHook::new(fixer, NonZeroUsize::new(10).unwrap());
    let mut runtime = Nsga2Evolver::new::<clang::nonterminal_start>(
        tuple_list!(fitness, /* nodes, */ structs, fields, fns, exprs),
        hook,
        100,
        1000,
        Ratio::new(80, 100),
    )
    .expect("Should be valid.");

    let generator = DepthLimiter::new(clang::STRUCTURE.inner(), 100);
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
        println!("Candidate #{i} ===============================================");
        println!("{}", String::from_utf8(
            WriteVisitor::new(Vec::new())
                .visit(candidate.node(), 0)?
                .continue_value()
                .unwrap()
                .output()
        )?);
        println!("Fitness: {:?}", candidate.measurement().fitness());
        println!("Size: {}", candidate.node().count_nodes());
        // Checking if the NodeScan are correct:
        println!("Structs: {}", NodeScan::new(clang::nonterminal_struct_def::DISCRIMINANT as usize).visit(candidate.node(), 0).unwrap().continue_value().unwrap().matches().len());
    }

    Ok(())
}

#[allow(deprecated)]
fn main() -> Result<(), Error> {
    // Run multiple times to see different results.
    for _ in 0..10 {
        run_once()?;
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        println!("~                                                           ~");
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }
    Ok(())
}
