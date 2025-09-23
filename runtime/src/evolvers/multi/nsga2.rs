use crate::evolvers::Evolver;
use crate::evolvers::basic::{BasicHook, BasicIndividual};
use crate::evolvers::multi::{Dom, Multiobjective};
use crate::measurement::{FitnessMeasurer, HasFitness, HasMeasurement, HasViolations};
use crate::operators::{MutatorVisitor, crossover};
use crate::population::Individual;
use alloc::collections::{BTreeSet, BinaryHeap};
use alloc::vec;
use alloc::vec::Vec;
use anyhow::Error;
use core::cmp::Ordering;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ops::Sub;
use fandango::generation::{Generated, InPlaceGenerated, Sampler};
use fandango::lang::FandangoNode;
use fandango::typing::{AsNode, Node};
use fandango::visitor::kpath::{KPathUpdate, KPaths};
use fandango::visitor::navigation::{Advance, CountNodes, GoToMut};
use fandango::visitor::{VisitWithMut, VisitableChildrenMut, Visitor, VisitorMut};
use num_rational::Ratio;

fn fast_non_dominated_sort<I>(mut population: Vec<I>, survivors: usize) -> Vec<Vec<I>>
where
    I: Dom<I>,
{
    let mut s = vec![Vec::new(); population.len()];
    let mut n = vec![0usize; population.len()];

    let mut front = Vec::with_capacity(population.len());
    for (p, first) in population.iter().enumerate() {
        for (q, other) in population.iter().enumerate().skip(p + 1) {
            let (dominator, dominated) = match first.dominates(other).unwrap_or(Ordering::Equal) {
                Ordering::Greater => (p, q),
                Ordering::Equal => continue,
                Ordering::Less => (q, p),
            };
            s[dominator].push(dominated);
            n[dominated] += 1;
        }
        if n[p] == 0 {
            front.push(p);
        }
    }

    let mut remaining = population.len() - front.len();
    let mut fronts = vec![front];
    for i in 0.. {
        if population.len() - remaining > survivors || remaining == 0 {
            break;
        }

        let front = &fronts[i];
        let mut next_front = Vec::with_capacity(remaining);
        for &p in front {
            for &q in &s[p] {
                n[q] -= 1;
                if n[q] == 0 {
                    next_front.push(q);
                }
            }
        }

        remaining -= next_front.len();
        fronts.push(next_front);
    }

    // we extract the population members here in reverse index order
    let mut returned = fronts
        .iter()
        .map(|f| Vec::with_capacity(f.len()))
        .collect::<Vec<_>>();
    let mut extracted = BinaryHeap::from_iter(
        fronts
            .into_iter()
            .enumerate()
            .flat_map(|(idx, front)| front.into_iter().map(move |i| (i, idx))),
    );
    while let Some((extracted, front)) = extracted.pop() {
        returned[front].push(population.swap_remove(extracted));
    }
    returned
}

impl<I> Dom<I> for I
where
    I: Individual,
    <I as HasMeasurement>::Measurement: HasFitness,
    <<I as HasMeasurement>::Measurement as HasFitness>::Fitness:
        Dom<<<I as HasMeasurement>::Measurement as HasFitness>::Fitness>,
{
    fn dominates(&self, other: &I) -> Option<Ordering> {
        self.measurement()
            .fitness()
            .dominates(other.measurement().fitness())
    }
}

pub struct Nsga2Evolver<H, M, N> {
    measurer: M,
    hooks: H,

    size: usize,
    replication: usize,

    crossover_rate: Ratio<usize>,
    phantom: PhantomData<N>,
}

impl<H, MT> Nsga2Evolver<H, Multiobjective<MT>, ()> {
    pub fn new<N>(
        measurers: MT,
        hooks: H,
        size: usize,
        replication: usize,
        crossover_rate: Ratio<usize>,
    ) -> Option<Nsga2Evolver<H, Multiobjective<MT>, N>> {
        if crossover_rate.numer() > crossover_rate.denom() {
            return None;
        }
        Some(Nsga2Evolver::<H, Multiobjective<MT>, N> {
            measurer: Multiobjective::new(measurers),
            hooks,
            size,
            replication,
            crossover_rate,
            phantom: PhantomData,
        })
    }
}

impl<N, G, S, H, M, V> Evolver<BasicIndividual<N, V>, G, S> for Nsga2Evolver<H, M, N>
where
    H: Nsga2Hook<BasicIndividual<N, V>, G, S>,
    N: Node + Generated<S, G>,
    for<'a, 'b, 'c> N::TypeMut<'a>: VisitWithMut<MutatorVisitor<'b, S, G>>
        + InPlaceGenerated<S, G>
        + VisitableChildrenMut<N::TypeMut<'a>>,
    for<'a> M: FitnessMeasurer<'a, N, Error = Error, Value = V>,
    V: HasFitness + HasViolations,
    <V as HasFitness>::Fitness: Dom<<V as HasFitness>::Fitness>,
    S: Sampler<N>,
{
    type Population = Vec<BasicIndividual<N, V>>;
    type Error = Error;

    fn initial(
        &mut self,
        generators: &mut G,
        sampler: &mut S,
    ) -> Result<Self::Population, Self::Error> {
        let mut population = Vec::with_capacity(self.size + self.replication);
        for _ in 0..(self.size + self.replication) {
            let mut node = N::generate(sampler, generators, 0);
            self.hooks
                .individual_created(&mut node, generators, sampler)?;
            let measurement = self.measurer.evaluate(&node)?;
            let individual = BasicIndividual::new(node, measurement);
            population.push(individual);
        }

        let mut fronts = fast_non_dominated_sort(population, self.size);
        // only need to diversity sort the last chunk
        self.hooks.diversity_sort(fronts.last_mut().unwrap());

        Ok(fronts.into_iter().flatten().take(self.size).collect())
    }

    fn step(
        &mut self,
        generators: &mut G,
        sampler: &mut S,
        mut population: Self::Population,
    ) -> Result<Self::Population, Self::Error> {
        debug_assert_eq!(population.len(), self.size);

        let mut descendents = Vec::with_capacity(self.replication + self.size);
        for _ in 0..self.replication {
            let parent = &population[sampler.sample() % population.len()];
            let mut child = parent.node().clone();
            let parent_violations = parent.measurement().violations().violations();
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

        descendents.append(&mut population);

        let mut fronts = fast_non_dominated_sort(descendents, self.size);
        // only need to diversity sort the last chunk
        self.hooks.diversity_sort(fronts.last_mut().unwrap());

        // memory reuse
        population.extend(fronts.into_iter().flatten().take(self.size));

        Ok(population)
    }
}

pub trait Nsga2Hook<I, G, S>: BasicHook<I::Node, G, S>
where
    I: Individual,
{
    #[allow(unused)]
    fn diversity_sort(&mut self, individuals: &mut [I]) {}
}

impl<I, G, S> Nsga2Hook<I, G, S> for () where I: Individual {}

pub struct KPathDiversityHook<H> {
    k: NonZeroUsize,
    inner: H,
}

impl<H> KPathDiversityHook<H> {
    pub fn new(inner: H, k: NonZeroUsize) -> Self {
        Self { k, inner }
    }
}

impl<H, N, G, S> BasicHook<N, G, S> for KPathDiversityHook<H>
where
    H: BasicHook<N, G, S>,
{
    fn individual_created(
        &mut self,
        node: &mut N,
        generators: &mut G,
        sampler: &mut S,
    ) -> Result<(), Error> {
        self.inner.individual_created(node, generators, sampler)
    }
}

impl<H, I, G, S> Nsga2Hook<I, G, S> for KPathDiversityHook<H>
where
    H: BasicHook<I::Node, G, S>,
    I: Individual,
    I::Node: Node + AsNode,
{
    fn diversity_sort(&mut self, individuals: &mut [I]) {
        let FandangoNode::Program(program) = individuals[0].node().root() else {
            panic!("The root node wasn't a program node!")
        };
        let mut covered = Vec::with_capacity(individuals.len());
        let mut kpaths = KPaths::new::<<I::Node as Node>::Type<'_>>(self.k, program);
        for individual in individuals.iter() {
            let _ = KPathUpdate::inserting(&mut kpaths).visit(individual.node(), 0);
            covered.push(
                kpaths
                    .lookup()
                    .iter()
                    .filter_map(|(k, v)| (*v != 0).then_some(k))
                    .cloned()
                    .collect::<BTreeSet<_>>(),
            );
            // we could use clear, but for large k, the number of paths likely exceeds the number
            // of paths we actually visit!
            let _ = KPathUpdate::removing(&mut kpaths).visit(individual.node(), 0);
        }
        // greedy set cover
        for i in 0..individuals.len() {
            let (best_individual, best_set) = covered
                .iter()
                .enumerate()
                .max_by_key(|(_i, set)| set.len())
                .unwrap();
            if best_set.is_empty() {
                break;
            }
            covered.swap(i, best_individual);
            individuals.swap(i, best_individual);
            let (seen, unevaluated) = covered.split_at_mut(i + 1);
            let best_set = &seen[i];
            for set in unevaluated {
                *set = set.sub(best_set);
            }
        }
    }
}
