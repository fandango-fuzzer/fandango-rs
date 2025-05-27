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
use core::convert::Infallible;
use core::marker::PhantomData;
use core::ops::ControlFlow;
use fandango::generation::{Generated, Generator, GeneratorTuple, InPlaceGenerated, Sampler};
use fandango::graph::IntoGraph;
use fandango::lang::{FandangoNode, Program};
use fandango::typing::{AsNodeMut, AsStaticNode, Node, StaticDiscriminable};
use fandango::visitor::error::InvalidPath;
use fandango::visitor::navigation::GoTo;
use fandango::visitor::{VisitResult, VisitableChildren, Visitor};
use hashbrown::HashMap;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::Direction;

/// A generator which restricts the depth of the generated grammar.
///
/// This generator works by pre-calculating the shortest path(s) from each alternative, then forcing
/// alternative selection once a specified depth is reached.
pub struct DepthLimiter<SP> {
    max_depth: usize,
    shortest_path: SP,
}

impl<'program, 'source> DepthLimiter<HashMap<FandangoNode<'program, 'source>, Vec<usize>>> {
    /// Produce a new depth limiter for the given opaque type.
    pub fn new(program: &'program Program<'source>, max_depth: usize) -> Self {
        let (_nonterminals, graph) = program.into_graph();

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

        let shortest_path = alternatives
            .into_iter()
            .map(|(idx, paths)| (*graph.node_weight(idx).unwrap(), paths))
            .collect();
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
        if let Some(options) = self.get(&N::static_definition()) {
            return options.get(sampler.sample() % options.len()).copied();
        }
        None
    }
}

/// Mutate a node, FANDANGO-style.
pub fn mutate<'a, N, S, G>(
    mutated: &'a mut N,
    choices: &mut Vec<VecDeque<usize>>,
    sampler: &mut S,
    generator: &mut G,
) -> Result<Option<N::TypeMut<'a>>, InvalidPath>
where
    N: Node + for<'b> GoTo<'b, Value = N::TypeMut<'b>>,
    for<'b> N::TypeMut<'b>: InPlaceGenerated<S, G>,
    S: Sampler<N>,
{
    if choices.is_empty() {
        return Ok(None);
    }

    // pick any path for mutation
    let mut path = choices.swap_remove(sampler.sample() % choices.len());

    // prune paths which are invalidated; that is, removing paths which start with this one
    choices.retain(|v| v.len() < path.len() || v.iter().zip(&path).any(|(a, b)| a != b));

    let depth = path.len() - 1; // retain depth info
    let idx = path.pop_front().unwrap();
    let mut node = mutated.go_to(idx, path)?;

    node.generate_in_place(sampler, generator, depth);

    Ok(Some(node))
}

/// Scans a set of nodes for all instances of a given discriminant.
pub struct NodeScan {
    discriminant: usize,
    path: VecDeque<usize>,
    paths: Vec<VecDeque<usize>>,
}

impl NodeScan {
    /// Create a new scanner for the given node.
    pub fn new<O>() -> Self
    where
        O: StaticDiscriminable,
    {
        Self {
            discriminant: O::DISCRIMINANT,
            path: VecDeque::new(),
            paths: Vec::new(),
        }
    }

    /// Acquire the paths resulting from the search.
    pub fn paths(self) -> Vec<VecDeque<usize>> {
        self.paths
    }
}

impl<T> Visitor<T> for NodeScan
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        self.path.push_back(idx);
        if node.discriminant() == self.discriminant {
            self.paths.push(self.path.clone());
        }
        let mut result = T::from(node).visit_each(self);
        if let Ok(ControlFlow::Continue(visitor)) = &mut result {
            visitor.path.pop_back();
        }
        result
    }
}

/// Crossover a node, FANDANGO-style.
///
/// Due to limitations of the Rust type system, we have to keep this as a macro for now. :(
#[macro_export]
macro_rules! crossover {
    ($crossed:ty, $mutated:expr, $base:expr, $choices:ident, $sampler:expr) => {
        (|| {
            let mut base_choices = $crate::operators::NodeScan::new::<$crossed>()
                .visit($base, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .paths();
            if base_choices.is_empty() {
                return Result::<bool, ::fandango::visitor::error::InvalidPath>::Ok(false);
            }

            let mut filtered = ::alloc::vec::Vec::new();
            for path in $choices.iter() {
                let mut cloned = path.clone();
                let idx = cloned.pop_front().unwrap();
                let mut node = $mutated.go_to(idx, cloned)?;

                if ::fandango::typing::AsNodeMut::<$crossed>::as_node_mut(&mut node).is_some() {
                    filtered.push(path);
                }
            }
            if filtered.is_empty() {
                return Ok(false);
            }

            // pick any path for mutation
            let mut path = filtered
                .swap_remove(
                    ::fandango::generation::Sampler::<$crossed>::sample($sampler) % $choices.len(),
                )
                .clone();
            let mut base_path = base_choices
                .swap_remove(
                    ::fandango::generation::Sampler::<$crossed>::sample($sampler)
                        % base_choices.len(),
                )
                .clone();

            // prune paths which are invalidated; that is, removing paths which start with this one
            $choices.retain(|v| v.len() < path.len() || v.iter().zip(&path).any(|(a, b)| a != b));

            let idx = path.pop_front().unwrap();
            let mut node = $mutated.go_to(idx, path)?;

            let _ = base_path.pop_front().unwrap();
            let mut base = $base.go_to(idx, base_path)?;

            ::fandango::visitor::assignment::AssignmentVisitor(
                ::fandango::typing::AsNodeMut::<$crossed>::as_node_mut(&mut base)
                    .cloned()
                    .unwrap(),
            )
            .visit(
                ::fandango::typing::AsNodeMut::<$crossed>::as_node_mut(&mut node).unwrap(),
                idx,
            )
            .unwrap()
            .break_value()
            .unwrap();

            Ok(true)
        })()
    };
}
