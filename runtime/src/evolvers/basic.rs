use crate::evolvers::Evolver;
use crate::measurement::{FitnessMeasurer, HasFitness, HasMeasurement, HasViolations};
use crate::operators::{MutatorVisitor, crossover};
use crate::population::Individual;
use alloc::collections::BinaryHeap;
use alloc::vec::Vec;
use anyhow::Error;
use core::cmp::{Ordering, Reverse};
use core::iter;
use core::marker::PhantomData;
use fandango::generation::{Generated, InPlaceGenerated, Sampler};
use fandango::typing::Node;
use fandango::visitor::navigation::{Advance, CountNodes, GoToMut};
use fandango::visitor::{VisitWithMut, VisitableChildrenMut, VisitorMut};
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
        if crossover_rate.numer() > crossover_rate.denom() || size <= elites {
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
    pub fn new(node: N, measurement: V) -> Self {
        Self { node, measurement }
    }
}

impl<N, V> Eq for BasicIndividual<N, V>
where
    V: HasFitness,
    V::Fitness: Eq,
{
}

impl<N, V> PartialEq<Self> for BasicIndividual<N, V>
where
    V: HasFitness,
    V::Fitness: PartialEq<V::Fitness>,
{
    fn eq(&self, other: &Self) -> bool {
        self.measurement.fitness().eq(&other.measurement.fitness())
    }
}

impl<N, V> PartialOrd<Self> for BasicIndividual<N, V>
where
    V: HasFitness,
    V::Fitness: PartialOrd<V::Fitness>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.measurement
            .fitness()
            .partial_cmp(other.measurement.fitness())
    }
}

impl<N, V> Ord for BasicIndividual<N, V>
where
    V: HasFitness,
    V::Fitness: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.measurement.fitness().cmp(&other.measurement.fitness())
    }
}

impl<N, V> HasMeasurement for BasicIndividual<N, V>
where
    N: Node,
{
    type Measurement = V;

    fn measurement(&self) -> &Self::Measurement {
        &self.measurement
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
    for<'a, 'b, 'c> N::TypeMut<'a>: VisitWithMut<MutatorVisitor<'b, S, G>>
        + InPlaceGenerated<S, G>
        + VisitableChildrenMut<N::TypeMut<'a>>,
    for<'a> M: FitnessMeasurer<'a, N, Error = Error, Value = V>,
    V: HasFitness + HasViolations,
    V::Fitness: Ord,
    S: Sampler<N>,
{
    type Population = Vec<BasicIndividual<N, V>>;

    type Error = Error;

    fn initial(
        &mut self,
        generators: &mut G,
        sampler: &mut S,
    ) -> Result<Self::Population, Self::Error> {
        let mut population = BinaryHeap::with_capacity(self.size + self.replication);
        for _ in 0..(self.size + self.replication) {
            let mut node = N::generate(sampler, generators, 0);
            self.hooks
                .individual_created(&mut node, generators, sampler)?;
            let measurement = self.measurer.evaluate(&node)?;
            let individual = BasicIndividual::new(node, measurement);
            population.push(individual);
        }
        let population = Vec::from_iter(iter::from_fn(move || population.pop()).take(self.size));
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
                drop(mutated);
            } else {
                let mutator = MutatorVisitor::new(sampler, generators);
                mutated
                    .visit_with_mut(mutator, 0)?
                    .continue_value()
                    .unwrap();
            }

            self.hooks
                .individual_created(&mut child, generators, sampler)?;
            let measurement = self.measurer.evaluate(&child)?;
            descendents.push(BasicIndividual::new(child, measurement));
        }

        // mix in the non-elites of the last population
        descendents.extend(population.drain(self.elites..));
        // take the top (size - #elites)
        population.extend(iter::from_fn(move || descendents.pop()).take(self.size - self.elites));
        // put into descending order
        population.sort_by(|n1, n2| n2.cmp(n1));
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
