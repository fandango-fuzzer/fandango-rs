use crate::evolvers::Evolver;
use crate::measurement::{FitnessMeasurer, HasFitness, HasViolations};
use crate::operators::crossover;
use crate::population::Individual;
use alloc::collections::BinaryHeap;
use alloc::vec::Vec;
use anyhow::Error;
use core::cmp::{Ordering, Reverse};
use core::iter;
use core::marker::PhantomData;
use fandango::generation::{Generated, InPlaceGenerated, Sampler};
use fandango::typing::Node;
use fandango::visitor::VisitorMut;
use fandango::visitor::navigation::{Advance, CountNodes, GoToMut};
use num_rational::Ratio;

pub struct BasicEvolver<H, M, N> {
    measurer: M,
    hooks: H,

    size: usize,
    elites: usize,
    replication: usize,

    crossover_rate: Ratio<usize>,
    phantom: PhantomData<N>,
}

impl<H, M> BasicEvolver<H, M, ()> {
    pub fn new<N>(
        measurer: M,
        hooks: H,
        size: usize,
        elites: usize,
        replication: usize,
        crossover_rate: Ratio<usize>,
    ) -> Option<BasicEvolver<H, M, N>> {
        if crossover_rate.numer() > crossover_rate.denom() || size <= elites || replication < size {
            return None;
        }
        Some(BasicEvolver::<H, M, N> {
            measurer,
            hooks,
            size,
            elites,
            replication,
            crossover_rate,
            phantom: PhantomData,
        })
    }
}

pub struct BasicIndividual<N, V> {
    node: N,
    measurement: V,
}

impl<N, V> BasicIndividual<N, V> {
    pub fn measurement(&self) -> &V {
        &self.measurement
    }
}

impl<N, V> Eq for BasicIndividual<N, V> where V: HasFitness {}

impl<N, V> PartialEq<Self> for BasicIndividual<N, V>
where
    V: HasFitness,
{
    fn eq(&self, other: &Self) -> bool {
        self.measurement.fitness().eq(&other.measurement.fitness())
    }
}

impl<N, V> PartialOrd<Self> for BasicIndividual<N, V>
where
    V: HasFitness,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<N, V> Ord for BasicIndividual<N, V>
where
    V: HasFitness,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.measurement.fitness().cmp(&other.measurement.fitness())
    }
}

impl<N, V> Individual for BasicIndividual<N, V>
where
    N: Node,
{
    type Node = N;

    fn node(&self) -> &Self::Node {
        &self.node
    }

    fn node_mut(&mut self) -> &mut Self::Node {
        &mut self.node
    }
}

impl<N, G, S, H, M, V> Evolver<BasicIndividual<N, V>, G, S> for BasicEvolver<H, M, N>
where
    H: BasicHook<N, G, S>,
    N: Node + Generated<S, G>,
    for<'a> N::TypeMut<'a>: InPlaceGenerated<S, G>,
    for<'a> M: FitnessMeasurer<'a, N, Error = Error, Value = V>,
    V: HasFitness + HasViolations,
    S: Sampler<N>,
{
    type Population = Vec<BasicIndividual<N, V>>;

    type Error = Error;

    fn initial(
        &mut self,
        generators: &mut G,
        sampler: &mut S,
    ) -> Result<Self::Population, Self::Error> {
        let mut population = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            let mut node = N::generate(sampler, generators, 0);
            self.hooks
                .individual_created(&mut node, generators, sampler)?;
            let violations = self.measurer.check(&node)?;
            let measurement = self.measurer.evaluate(&node, violations)?;
            let individual = BasicIndividual { node, measurement };
            population.push(individual);
        }
        population.sort_by_key(|n| Reverse(n.measurement.fitness()));
        Ok(population)
    }

    fn step(
        &mut self,
        generators: &mut G,
        sampler: &mut S,
        mut population: Self::Population,
    ) -> Result<Self::Population, Self::Error> {
        debug_assert_eq!(population.len(), self.size);
        debug_assert!(population.is_sorted_by_key(|n| Reverse(n.measurement.fitness())));

        let mut descendents = BinaryHeap::with_capacity(self.replication + self.size - self.elites);
        for _ in 0..self.replication {
            let parent = &population[sampler.sample() % population.len()];
            let mut child = parent.node().clone();
            let parent_violations = parent.measurement.violations().violations();
            let mut mutated = if parent_violations.is_empty() {
                let count = child.count_nodes();
                Advance::forward_ref(sampler.sample() % count)
                    .visit_mut(&mut child, 0)?
                    .break_value()
                    .unwrap()
            } else {
                let mut path =
                    parent_violations[sampler.sample() % parent_violations.len()].clone();
                let front = path.pop_front().unwrap();
                child.go_to_mut(front, path)?
            };
            if sampler.sample() % *self.crossover_rate.denom() < *self.crossover_rate.numer() {
                let mate = &population[sampler.sample() % population.len()];
                crossover(&mut mutated, mate.node(), sampler)?;
            } else {
                mutated.generate_in_place(sampler, generators, 0);
            }
            drop(mutated);

            self.hooks
                .individual_created(&mut child, generators, sampler)?;
            let violations = self.measurer.check(&child)?;
            let measurement = self.measurer.evaluate(&child, violations)?;
            descendents.push(BasicIndividual {
                node: child,
                measurement,
            });
        }

        // mix in the non-elites of the last population
        descendents.extend(population.drain(self.elites..));
        // take the top (size - #elites)
        population.extend(iter::from_fn(move || descendents.pop()).take(self.size - self.elites));
        // put into descending order
        population.sort_by_key(|n| Reverse(n.measurement.fitness()));
        Ok(population)
    }
}

pub trait BasicHook<N, G, S> {
    #[allow(unused)]
    fn individual_created(
        &mut self,
        node: &mut N,
        generators: &mut G,
        sampler: &mut S,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl<N, G, S> BasicHook<N, G, S> for () {}
