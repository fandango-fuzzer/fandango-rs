//! Implements the operators as described in FANDANGO.
//!
//! 1. Mutator:
//!   - Find one node at random which fails constraint satisfaction
//!   - Regenerate that node
//! 2. Crossover:
//!   - Find one node in T1 at random which fails constraint satisfaction
//!   - Find another node of the same type in T2
//!   - Replace the matching node of T1 with that of T2
//! 3. Depth-limiting Generator/Sampler:
//!   - FANDANGO restricts the depth of produced derivation trees; we approximate this behavior

use alloc::vec::Vec;
use core::convert::Infallible;
use core::marker::PhantomData;
#[allow(deprecated)]
use fandango::dynamic::{DefinitionOf, HasDynamicSampler};
use fandango::generation::{Generated, Generator, GeneratorTuple, Sampler};
use fandango::graph::{IntoGraph, shortest_path};
use fandango::lang::{FandangoNode, Program};
use fandango::typing::{AsNodeRef, AssignFrom, Discriminable, Node, StaticDiscriminable};
use fandango::visitor::error::InvalidPath;
use fandango::visitor::{VisitResult, VisitableChildren, Visitor};
use fandango::{impl_definition_of, impl_has_dynamic_sampler};
use hashbrown::HashMap;

/// A generator which restricts the depth of the generated grammar.
///
/// This generator works by pre-calculating the shortest path(s) from each alternative, then forcing
/// alternative selection once a specified depth is reached.
#[derive(Clone, Debug)]
pub struct DepthLimiter<SP> {
    max_depth: usize,
    shortest_path: SP,
}

impl<'program, 'source> DepthLimiter<HashMap<FandangoNode<'program, 'source>, Vec<usize>>>
where
    'program: 'source,
{
    /// Produce a new depth limiter for the given opaque type.
    pub fn new(program: &'program Program<'source>, max_depth: usize) -> Self {
        let (_nonterminals, graph) = program.into_graph();

        let shortest_path = shortest_path(&graph);
        Self {
            max_depth,
            shortest_path,
        }
    }
}

impl<N, W, S, SP> Generator<N, W, S> for DepthLimiter<SP>
where
    W: GeneratorTuple<N, S>,
    N: Node + for<'a> Generated<ShortestPathSampler<'a, S, SP>, W>,
    SP: ShortestPath<N, S>,
{
    fn generate(&mut self, sampler: &mut S, with: &mut W, depth: usize) -> Option<N> {
        if depth < self.max_depth {
            None
        } else {
            let mut sp_sampler = ShortestPathSampler {
                inner: sampler,
                shortest_path: &mut self.shortest_path,
            };
            Some(N::generate(&mut sp_sampler, with, depth))
        }
    }
}

/// Sampler which picks the shortest path according to a given shortest path provider.
pub struct ShortestPathSampler<'a, S, SP> {
    inner: &'a mut S,
    shortest_path: &'a mut SP,
}

impl<N, S, SP> Sampler<N> for ShortestPathSampler<'_, S, SP>
where
    S: Sampler<N>,
    SP: ShortestPath<N, S>,
{
    fn sample_kleene(&mut self) -> usize {
        self.inner.sample_kleene()
    }

    fn sample_plus(&mut self) -> usize {
        self.inner.sample_plus()
    }

    fn sample_optional(&mut self) -> bool {
        self.inner.sample_optional()
    }

    fn sample_repetition(&mut self, lower: usize, upper: usize) -> usize {
        self.inner.sample_repetition(lower, upper)
    }

    fn sample_alternative(&mut self, count: usize) -> usize {
        self.shortest_path
            .shortest_path(self.inner)
            .unwrap_or_else(|| self.inner.sample_alternative(count))
    }

    fn sample(&mut self) -> usize {
        self.inner.sample()
    }

    fn reseed(&mut self, seed: u64) {
        self.inner.reseed(seed)
    }
}

impl<S, SP> HasDynamicSampler for ShortestPathSampler<'_, S, SP>
where
    S: HasDynamicSampler,
{
    impl_has_dynamic_sampler!(inner);
}

impl<N, S, SP> DefinitionOf<N> for ShortestPathSampler<'_, S, SP>
where
    S: DefinitionOf<N>,
{
    impl_definition_of!(inner);
}

trait ShortestPath<N, S> {
    fn shortest_path(&self, sampler: &mut S) -> Option<usize>;
}

impl<N, S> ShortestPath<N, S> for () {
    fn shortest_path(&self, _sampler: &mut S) -> Option<usize> {
        None
    }
}

// for the future... :)
impl<N, S, T, Tail> ShortestPath<N, S> for ((PhantomData<T>, Vec<usize>), Tail)
where
    N: Node + StaticDiscriminable,
    S: Sampler<N>,
    for<'a> T: Node<Type<'a> = N::Type<'a>> + StaticDiscriminable + 'a,
    Tail: ShortestPath<N, S>,
{
    #[inline(always)]
    fn shortest_path(&self, sampler: &mut S) -> Option<usize> {
        if N::DISCRIMINANT == T::DISCRIMINANT {
            let options = &self.0.1;
            if options.is_empty() {
                None
            } else {
                options.get(sampler.sample() % options.len()).copied()
            }
        } else {
            self.1.shortest_path(sampler)
        }
    }
}

impl<N, S> ShortestPath<N, S> for HashMap<FandangoNode<'static, 'static>, Vec<usize>>
where
    S: Sampler<N> + DefinitionOf<N>,
{
    #[inline(always)]
    fn shortest_path(&self, sampler: &mut S) -> Option<usize> {
        if let Some(options) = self.get(&sampler.definition_of()) {
            return options.get(sampler.sample() % options.len()).copied();
        }
        None
    }
}

/// Scans a set of nodes for all instances of a given discriminant.
pub struct NodeScan<T> {
    discriminant: usize,
    matches: Vec<T>,
}

impl<T> NodeScan<T> {
    /// Create a new scanner for the given node.
    pub fn new(discriminant: usize) -> Self {
        Self {
            discriminant,
            matches: Vec::new(),
        }
    }

    /// Acquire the paths resulting from the search.
    pub fn matches(self) -> Vec<T> {
        self.matches
    }
}

impl<T> Visitor<T> for NodeScan<T>
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        if node.discriminant() == self.discriminant {
            self.matches.push(T::from(node));
        }
        T::from(node).visit_each(self)
    }
}

/// Crossover operator, which selects a random (matching) subtree from `base` into `node`
pub fn crossover<'a, N, S>(
    node: &mut N::TypeMut<'a>,
    base: &'a N,
    sampler: &mut S,
) -> Result<bool, InvalidPath>
where
    N: Node,
    S: Sampler<()>,
{
    let discriminant = node.discriminant();

    let mut base_choices = NodeScan::new(discriminant)
        .visit(base, 0)
        .unwrap()
        .continue_value()
        .unwrap()
        .matches();
    if base_choices.is_empty() {
        return Ok(false);
    }

    let base = base_choices.swap_remove(sampler.sample() % base_choices.len());
    let success = node.assign_from(base);
    debug_assert!(success);

    Ok(true)
}

/// A simple visitor which counts nonterminals, for use in benchmarking against FANDANGO.
#[derive(Debug, Default)]
pub struct NonterminalVisitor {
    count: usize,
}

impl NonterminalVisitor {
    /// Collect the count associated with this visitor.
    pub fn count(self) -> usize {
        self.count
    }
}

impl<T> Visitor<T> for NonterminalVisitor
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        if matches!(node.definition(), FandangoNode::Nonterminal(_)) {
            self.count += 1;
        }
        T::from(node).visit_each(self)
    }
}
