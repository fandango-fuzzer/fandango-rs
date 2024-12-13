//! Utility generators, which perform some common generator routines.

use crate::generation::{Generated, Generator, GeneratorTuple, Sampler};
use crate::graph::IntoGraph;
use crate::typing::AsNode;
use pest::Span;
use petgraph::graphmap::DiGraphMap;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

type FandangoNode = crate::graph::FandangoNode<'static, 'static>;

/// Error variant which indicates that a [`Flattener`] could not be created for a given type, e.g.
/// because it contained a cycle.
#[derive(Debug)]
pub struct Unflattenable;

impl Display for Unflattenable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("The structure could not be flattened.")
    }
}

impl Error for Unflattenable {}

/// Flattens nested alternatives (including via non-terminals) and makes their derivations of equal
/// weight. Considers [`FandangoNode::Concatenation`], [`FandangoNode::Operator`], and
/// [`FandangoNode::String`] as points at which to consider as a "single" derivation.
///
/// For example, consider the following grammar snippet:
/// ```text,ignore
/// <non_zero> ::=
///               "1"
///             | "2"
///             | "3"
///             | "4"
///             | "5"
///             | "6"
///             | "7"
///             | "8"
///             | "9"
///             ;
/// <digit> ::= "0" | <non_zero>;
/// ```
///
/// When generating `digit` without flattening, approximately 50% of the generated digits will be 0.
#[derive(Debug, Default)]
pub struct Flattener {
    targets: HashSet<FandangoNode>,
    flattened: HashMap<FandangoNode, Flattened>,
}

#[derive(Debug)]
struct Flattened {
    children: BTreeMap<usize, FandangoNode>,
    total: usize,
}

fn flatten(
    node: FandangoNode,
    graph: &DiGraphMap<FandangoNode, Span<'static>>,
    stack: &mut HashSet<FandangoNode>,
    collected: &mut HashMap<FandangoNode, Flattened>,
) -> Result<usize, Unflattenable> {
    match node {
        FandangoNode::Nonterminal(_) | FandangoNode::Alternative(_) => {
            if stack.insert(node) {
                let mut children = BTreeMap::new();
                let mut total = 0usize;
                assert!(graph.contains_node(node));
                for child in graph.edges(node).map(|(_, c, _)| c) {
                    let count = flatten(child, graph, stack, collected)?;
                    children.insert(total, child);
                    total += count;
                }
                collected.insert(node, Flattened { children, total });
                stack.remove(&node);
                Ok(total)
            } else {
                Err(Unflattenable)
            }
        }
        FandangoNode::Concatenation(_) | FandangoNode::Operator(_) | FandangoNode::String(_) => {
            collected.insert(node, Flattened {
                children: BTreeMap::new(),
                total: 1,
            });
            Ok(1)
        }
        _ => unreachable!("Encountered a node which should never be a descendant in a graph"),
    }
}

impl Flattener {
    /// Create a new [`Flattener`]. It will not flatten anything until you call
    /// [`Flattener::flatten`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the flattening list. If a node is not listed here, it will not be flattened.
    pub fn flatten<N: AsNode>(mut self) -> Result<Self, Unflattenable> {
        let graph = N::root().into_graph();

        flatten(
            N::definition(),
            &graph,
            &mut HashSet::new(),
            &mut self.flattened,
        )?;
        self.targets.insert(N::definition());

        Ok(self)
    }
}

struct FlattenedSampler<'a, S> {
    choice: usize,
    flattened: &'a HashMap<FandangoNode, Flattened>,
    sampler: &'a mut S,
}

impl<N, S> Sampler<N> for FlattenedSampler<'_, S>
where
    N: AsNode,
    S: Sampler<N>,
{
    fn sample_kleene(&mut self) -> usize {
        self.sampler.sample_kleene()
    }

    fn sample_plus(&mut self) -> usize {
        self.sampler.sample_plus()
    }

    fn sample_optional(&mut self) -> bool {
        self.sampler.sample_optional()
    }

    fn sample_repetition(&mut self, lower: usize, upper: usize) -> usize {
        self.sampler.sample_repetition(lower, upper)
    }

    fn sample_alternative(&mut self, _: usize) -> usize {
        // we blithely ignore count here; this was already computed!
        let current = self
            .flattened
            .get(&N::definition())
            .expect("Attempted to generate something that wasn't in the graph");
        match current.children.range(..=self.choice).enumerate().last() {
            None => unreachable!("Invalid choice while flattening"),
            Some((choice, (&lower, _))) => {
                self.choice -= lower;
                choice
            }
        }
    }
}

impl<N, W, S> Generator<N, W, S> for Flattener
where
    N: AsNode + for<'a> Generated<FlattenedSampler<'a, S>, W>,
    W: for<'a> GeneratorTuple<N, FlattenedSampler<'a, S>>,
    S: Sampler<N>,
{
    fn generate(&mut self, sampler: &mut S, with: &mut W) -> Option<N> {
        if self.targets.contains(&N::definition()) {
            let flattened = self.flattened.get(&N::definition()).unwrap();
            let choice = sampler.sample_alternative(flattened.total);
            let mut sampler = FlattenedSampler {
                choice,
                flattened: &self.flattened,
                sampler,
            };
            Some(
                with.generate(&mut sampler)
                    .unwrap_or_else(|| N::generate(&mut sampler, with)),
            )
        } else {
            None
        }
    }
}
