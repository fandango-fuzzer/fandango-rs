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

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::hash::Hash;
use core::marker::PhantomData;
use fandango::generation::{Generated, Generator, GeneratorTuple, Sampler};
use fandango::graph::IntoGraph;
use fandango::lang::FandangoNode;
use fandango::typing::{AsStaticNode, Node, NodeTypes, StaticDiscriminable};
use hashbrown::HashMap;
use petgraph::data::DataMap;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::Direction;

pub struct DepthLimiter<SP> {
    max_depth: usize,
    shortest_path: SP,
}

impl DepthLimiter<ShortestPathImpl<HashMap<FandangoNode<'static, 'static>, Vec<usize>>>> {
    pub fn new<T>(mut max_depth: usize) -> Self
    where
        T: NodeTypes,
    {
        let (_nonterminals, graph) = T::ROOT.inner().into_graph();

        let mut depths = graph
            .node_references()
            .filter_map(|(idx, node)| {
                matches!(node, FandangoNode::String(_)).then_some((idx, 0usize))
            })
            .collect::<HashMap<_, _>>();
        let mut queue = VecDeque::from_iter(
            depths
                .keys()
                .copied()
                .flat_map(|term| graph.edges_directed(term, Direction::Incoming))
                .map(|e| e.source()),
        );
        let mut alternatives = HashMap::new();

        loop {
            let mut unchanged = true;
            let mut next_queue = VecDeque::new();

            for next in queue {
                if depths.contains_key(&next) {
                    continue;
                }
                let mut children = graph
                    .edges(next)
                    .map(|e| (e.weight().start(), depths.get(&e.target()).copied()))
                    .collect::<Vec<_>>();
                children.sort_by_key(|(s, _)| *s);

                let mut depth = None;
                match graph.node_weight(next).unwrap() {
                    FandangoNode::Alternative(_) => {
                        let (choices, alt_depth) = children.into_iter().enumerate().fold(
                            (Vec::new(), usize::MAX),
                            |(mut current, max_len), (idx, (_, len))| {
                                if let Some(len) = len {
                                    if len < max_len {
                                        current.clear();
                                        current.push(idx);
                                        return (current, len);
                                    } else if len == max_len {
                                        current.push(idx);
                                    }
                                }
                                (current, max_len)
                            },
                        );
                        alternatives.insert(next, choices);
                        depth = Some(alt_depth);
                    }
                    _ => {
                        if let Some(children) = children
                            .into_iter()
                            .map(|(_, c)| c)
                            .collect::<Option<Vec<_>>>()
                        {
                            depth = children.into_iter().min();
                        }
                    }
                }
                if let Some(depth) = depth {
                    next_queue.extend(
                        graph
                            .edges_directed(next, Direction::Incoming)
                            .map(|e| e.source()),
                    );
                    unchanged &= depths.insert(next, depth) == Some(depth);
                }
            }

            queue = next_queue;
            if unchanged {
                break;
            }
        }

        let shortest_paths = alternatives
            .into_iter()
            .map(|(idx, paths)| (*graph.node_weight(idx).unwrap(), paths))
            .collect();
        let shortest_path = ShortestPathImpl { shortest_paths };
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
}

pub struct ShortestPathImpl<SPS> {
    shortest_paths: SPS,
}

struct FirstStep<T>(T);
impl<T> FromIterator<T> for FirstStep<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        FirstStep(iter.into_iter().nth(1).unwrap())
    }
}

trait ShortestPath<N, S> {
    fn shortest_path(&self, sampler: &mut S) -> Option<usize>;
}

impl<N, S, SPS> ShortestPath<N, S> for ShortestPathImpl<SPS>
where
    SPS: ShortestPath<N, S>,
{
    fn shortest_path(&self, sampler: &mut S) -> Option<usize> {
        self.shortest_paths.shortest_path(sampler)
    }
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
            let options = &self.0 .1;
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
    N: AsStaticNode,
    S: Sampler<N>,
{
    #[inline(always)]
    fn shortest_path(&self, sampler: &mut S) -> Option<usize> {
        if let Some(options) = self.get(&FandangoNode::from(N::static_definition())) {
            return options.get(sampler.sample() % options.len()).copied();
        }
        None
    }
}
