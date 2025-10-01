use crate::evolvers::Evolver;
use crate::evolvers::basic::{BasicHook, BasicIndividual};
use crate::evolvers::multi::{Dom, Multiobjective};
use crate::measurement::{FitnessMeasurer, HasFitness, HasMeasurement, HasViolations};
use crate::operators::{MutatorVisitor, crossover};
use crate::population::Individual;
<<<<<<< HEAD
use alloc::collections::BinaryHeap;
=======
use alloc::collections::{BTreeSet, BinaryHeap};
>>>>>>> cf4b4e0 (multiobjective)
use alloc::vec;
use alloc::vec::Vec;
use anyhow::Error;
use core::cmp::Ordering;
<<<<<<< HEAD
use core::convert::Infallible;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ops::ControlFlow;
use fandango::generation::{Generated, InPlaceGenerated, Sampler};
use fandango::lang::FandangoNode;
use fandango::typing::{AsNode, Node, NodeLookup};
use fandango::visitor::kpath::{KPathUpdate, KPathVisit, KPathVisitor, KPaths};
use fandango::visitor::navigation::{Advance, CountNodes, GoToMut};
use fandango::visitor::{VisitWithMut, VisitableChildrenMut, Visitor, VisitorMut};
use mappable_rc::Mrc;
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
=======
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

fn fast_non_dominated_sort<I>(mut popululation: Vec<I>, survivors: usize) -> Vec<Vec<I>>
where
    I: Dom<I>,
{
    let mut s = vec![Vec::new(); popululation.len()];
    let mut n = vec![0usize; popululation.len()];

    let mut front = Vec::with_capacity(popululation.len());
    for (p, first) in popululation.iter().enumerate() {
        for q in (p + 1)..popululation.len() {
            let other = &popululation[q];
>>>>>>> cf4b4e0 (multiobjective)
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

<<<<<<< HEAD
    let mut remaining = population.len() - front.len();
    let mut fronts = vec![front];
    for i in 0.. {
        if population.len() - remaining > survivors || remaining == 0 {
=======
    let mut remaining = popululation.len() - front.len();
    let mut fronts = vec![front];
    for i in 0.. {
        if popululation.len() - remaining > survivors || remaining == 0 {
>>>>>>> cf4b4e0 (multiobjective)
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
<<<<<<< HEAD
        returned[front].push(population.swap_remove(extracted));
=======
        returned[front].push(popululation.swap_remove(extracted));
>>>>>>> cf4b4e0 (multiobjective)
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

<<<<<<< HEAD
/// An evolver which implements [NSGA-II], but where diversity is handled by a hook
///
/// [NSGA-II]: https://web.njit.edu/~horacio/Math451H/download/2002-6-2-DEB-NSGA-II.pdf
pub struct Nsga2Evolver<H, M, N> {
    measurer: M,
    hook: H,
=======
pub struct Nsga2Evolver<H, M, N> {
    measurer: M,
    hooks: H,
>>>>>>> cf4b4e0 (multiobjective)

    size: usize,
    replication: usize,

    crossover_rate: Ratio<usize>,
    phantom: PhantomData<N>,
}

impl<H, MT> Nsga2Evolver<H, Multiobjective<MT>, ()> {
<<<<<<< HEAD
    /// Create a new [`Nsga2Evolver`]
    ///
    /// - `measurers` specifies the tuple list of [`FitnessMeasurer`]s for this evolver.
    /// - `hook` specifies the [`Nsga2Evolver`] that will be called at various hook points.
    /// - `size` specifies the target size of the population.
    /// - `replication` specifies the number of individuals which will be produced by mutation and
    ///   crossover.
    /// - `crossover_rate` specifies the rate at which crossover is performed instead of mutation.
    ///   The function returns [`None`] if this is greater than 1.
    pub fn new<N>(
        measurers: MT,
        hook: H,
=======
    pub fn new<N>(
        measurers: MT,
        hooks: H,
>>>>>>> cf4b4e0 (multiobjective)
        size: usize,
        replication: usize,
        crossover_rate: Ratio<usize>,
    ) -> Option<Nsga2Evolver<H, Multiobjective<MT>, N>> {
        if crossover_rate.numer() > crossover_rate.denom() {
            return None;
        }
        Some(Nsga2Evolver::<H, Multiobjective<MT>, N> {
            measurer: Multiobjective::new(measurers),
<<<<<<< HEAD
            hook,
=======
            hooks,
>>>>>>> cf4b4e0 (multiobjective)
            size,
            replication,
            crossover_rate,
            phantom: PhantomData,
        })
    }
}

<<<<<<< HEAD
impl<N, G, S, H, M, V> Evolver<G, S> for Nsga2Evolver<H, M, N>
=======
impl<N, G, S, H, M, V> Evolver<BasicIndividual<N, V>, G, S> for Nsga2Evolver<H, M, N>
>>>>>>> cf4b4e0 (multiobjective)
where
    H: Nsga2Hook<BasicIndividual<N, V>, G, S>,
    N: Node + Generated<S, G>,
    for<'a, 'b, 'c> N::TypeMut<'a>: VisitWithMut<MutatorVisitor<'b, S, G>>
        + InPlaceGenerated<S, G>
        + VisitableChildrenMut<N::TypeMut<'a>>,
<<<<<<< HEAD
    for<'a> M: FitnessMeasurer<'a, N, Error = Error, Measurement = V>,
=======
    for<'a> M: FitnessMeasurer<'a, N, Error = Error, Value = V>,
>>>>>>> cf4b4e0 (multiobjective)
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
<<<<<<< HEAD
            self.hook
=======
            self.hooks
>>>>>>> cf4b4e0 (multiobjective)
                .individual_created(&mut node, generators, sampler)?;
            let measurement = self.measurer.evaluate(&node)?;
            let individual = BasicIndividual::new(node, measurement);
            population.push(individual);
        }

        let mut fronts = fast_non_dominated_sort(population, self.size);
        // only need to diversity sort the last chunk
<<<<<<< HEAD
        self.hook.diversity_sort(fronts.last_mut().unwrap());
=======
        self.hooks.diversity_sort(fronts.last_mut().unwrap());
>>>>>>> cf4b4e0 (multiobjective)

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
                let (&idx, path) = path
                    .make_contiguous()
                    .split_first()
                    .expect("Must be non-empty");
                child.go_to_mut(idx, path)?
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

<<<<<<< HEAD
            self.hook
=======
            self.hooks
>>>>>>> cf4b4e0 (multiobjective)
                .individual_created(&mut child, generators, sampler)?;
            let measurement = self.measurer.evaluate(&child)?;
            descendents.push(BasicIndividual::new(child, measurement));
        }

<<<<<<< HEAD
        descendents.append(&mut population);

        let mut fronts = fast_non_dominated_sort(descendents, self.size);
        // only need to diversity sort the last chunk
        self.hook.diversity_sort(fronts.last_mut().unwrap());
=======
        descendents.extend(population.drain(..));

        let mut fronts = fast_non_dominated_sort(descendents, self.size);
        // only need to diversity sort the last chunk
        self.hooks.diversity_sort(fronts.last_mut().unwrap());
>>>>>>> cf4b4e0 (multiobjective)

        // memory reuse
        population.extend(fronts.into_iter().flatten().take(self.size));

        Ok(population)
    }
}

<<<<<<< HEAD
/// A hook for [`Nsga2Evolver`]s
=======
>>>>>>> cf4b4e0 (multiobjective)
pub trait Nsga2Hook<I, G, S>: BasicHook<I::Node, G, S>
where
    I: Individual,
{
<<<<<<< HEAD
    /// Perform the diversity-based sorting on the last Pareto front
=======
>>>>>>> cf4b4e0 (multiobjective)
    #[allow(unused)]
    fn diversity_sort(&mut self, individuals: &mut [I]) {}
}

impl<I, G, S> Nsga2Hook<I, G, S> for () where I: Individual {}

<<<<<<< HEAD
/// A [`Nsga2Hook`] which wraps a [`BasicHook`] and optimizes diversity by rare k-paths
=======
>>>>>>> cf4b4e0 (multiobjective)
pub struct KPathDiversityHook<H> {
    k: NonZeroUsize,
    inner: H,
}

impl<H> KPathDiversityHook<H> {
<<<<<<< HEAD
    /// Create the hook over the provided [`BasicHook`] and with some `k`
=======
>>>>>>> cf4b4e0 (multiobjective)
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
<<<<<<< HEAD
    for<'a> <I::Node as Node>::Type<'a>: NodeLookup,
=======
>>>>>>> cf4b4e0 (multiobjective)
{
    fn diversity_sort(&mut self, individuals: &mut [I]) {
        let FandangoNode::Program(program) = individuals[0].node().root() else {
            panic!("The root node wasn't a program node!")
        };
<<<<<<< HEAD
        let mut kpaths = KPaths::new::<<I::Node as Node>::Type<'_>>(self.k, program);
        let mut update = KPathUpdate::inserting(&mut kpaths);
        for individual in individuals.iter() {
            update = update
                .visit(individual.node(), 0)
                .unwrap()
                .continue_value()
                .unwrap();
        }

        struct MinObserved<T> {
            fewest: usize,
            phantom: PhantomData<T>,
        }

        impl<T> MinObserved<T> {
            pub fn new() -> Self {
                Self {
                    fewest: usize::MAX,
                    phantom: PhantomData,
                }
            }
        }

        impl<T> KPathVisitor for MinObserved<T>
        where
            T: NodeLookup,
        {
            type Value = usize;
            type Break = Infallible;
            type Error = Infallible;

            fn visit_path(
                &mut self,
                count: usize,
                path: &Mrc<[usize]>,
            ) -> Result<ControlFlow<Self::Break>, Self::Error> {
                if !matches!(
                    T::lookup_node(*path.last().unwrap()),
                    FandangoNode::String(_)
                ) {
                    self.fewest = self.fewest.min(count);
                }
                Ok(ControlFlow::Continue(()))
            }

            fn value(self) -> Self::Value {
                self.fewest
            }
        }

        individuals.sort_by_cached_key(|individual| {
            KPathVisit::new(
                update.kpaths(),
                MinObserved::<<I::Node as Node>::Type<'_>>::new(),
            )
            .visit(individual.node(), 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .value()
        });
=======
        let mut covered = Vec::with_capacity(individuals.len());
        let mut kpaths = KPaths::new::<<I::Node as Node>::Type<'_>>(self.k, program);
        for individual in individuals.iter() {
            let _ = KPathUpdate::inserting(&mut kpaths).visit(individual.node(), 0);
            covered.push(
                kpaths
                    .lookup()
                    .iter()
                    .filter_map(|(k, v)| (*v != 0).then(|| k.clone()))
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
>>>>>>> cf4b4e0 (multiobjective)
    }
}
