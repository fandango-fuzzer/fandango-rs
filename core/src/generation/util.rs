//! Utility generators, which perform some common generator routines.

use crate::generation::{Generated, Generator, GeneratorTuple, Sampler};
use crate::lang::{Operator, Symbol};
use crate::typing::{AsStaticNode, Structured};
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use hashbrown::{HashMap, HashSet};

type FandangoNode = crate::lang::FandangoNode<'static, 'static>;

/// Error variant which indicates that a [`Flattener`] could not be created for a given type, e.g.
/// because it contained a cycle.
#[derive(Debug)]
pub struct Unflattenable;

impl Display for Unflattenable {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("The structure could not be flattened.")
    }
}

impl Error for Unflattenable {}

/// Flattens nested alternatives (including via non-terminals) and makes their derivations of equal
/// weight. Considers [`FandangoNode::String`]s as points to consider as a "single" derivation.
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

#[derive(Debug, Clone)]
struct Flattened {
    children: BTreeMap<usize, FandangoNode>,
    total: usize,
}

fn flatten(
    nonterminals: &HashMap<FandangoNode, FandangoNode>,
    node: FandangoNode,
    visited: &mut Vec<FandangoNode>,
    flattened: &mut HashMap<FandangoNode, Flattened>,
) -> Result<usize, Unflattenable> {
    if visited.contains(&node) {
        return Err(Unflattenable);
    }
    visited.push(node);
    let mut inner = || match node {
        nonterminal @ FandangoNode::Nonterminal(_) => {
            let alt = nonterminals
                .get(&nonterminal)
                .expect("Nonterminal is used, so it must be defined");
            let total = flatten(nonterminals, *alt, visited, flattened)?;
            flattened.insert(
                node,
                flattened
                    .get(alt)
                    .expect("We just processed this alternative")
                    .clone(),
            );
            Ok(total)
        }
        FandangoNode::Alternative(alt) => {
            let mut children = BTreeMap::new();
            let mut total = 0;
            for (idx, concat) in alt.concatenations().iter().enumerate() {
                let child = FandangoNode::from(concat);
                total += flatten(nonterminals, child, visited, flattened)?;
                children.insert(idx, child);
            }
            flattened.insert(node, Flattened { children, total });
            Ok(total)
        }
        FandangoNode::Operator(Operator::Symbol(s)) => match s.inner() {
            Symbol::Nonterminal(n) => {
                flatten(nonterminals, FandangoNode::from(n), visited, flattened)
            }
            Symbol::String(s) => flatten(nonterminals, FandangoNode::from(s), visited, flattened),
            Symbol::Alternative(a) => {
                flatten(nonterminals, FandangoNode::from(a), visited, flattened)
            }
        },
        FandangoNode::Concatenation(concat) if concat.operators().len() == 1 => flatten(
            nonterminals,
            FandangoNode::from(FandangoNode::from(&concat.operators()[0])),
            visited,
            flattened,
        ),
        FandangoNode::String(_) => Ok(1),
        _ => Err(Unflattenable),
    };
    let result = inner();
    visited.pop();
    result
}

impl Flattener {
    /// Create a new [`Flattener`]. It will not flatten anything until you call
    /// [`Flattener::flatten`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the flattening list. If a node is not listed here, it will not be flattened.
    pub fn flatten<N>(mut self) -> Result<Self, Unflattenable>
    where
        N: Structured + AsStaticNode,
    {
        let nonterminals = N::ROOT.inner().nonterminals();

        let node = N::static_definition();
        flatten(&nonterminals, node, &mut vec![], &mut self.flattened)?;

        self.targets.insert(node);
        Ok(self)
    }
}

struct FlattenedSampler<'a, S> {
    choice: usize,
    flattened: &'a HashMap<FandangoNode, Flattened>,
    sampler: &'a mut S,
}

impl<S> FlattenedSampler<'_, S> {
    fn sample_alternative_from(&mut self, current: &Flattened) -> usize {
        match current.children.range(..=self.choice).enumerate().last() {
            None => unreachable!("Invalid choice while flattening"),
            Some((choice, (&lower, _))) => {
                self.choice -= lower;
                choice
            }
        }
    }
}

impl<N, S> Sampler<N> for FlattenedSampler<'_, S>
where
    N: AsStaticNode,
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
            .get(&N::static_definition())
            .expect("Attempted to generate something that wasn't in the graph");
        self.sample_alternative_from(current)
    }
}

impl<N, W, S> Generator<N, W, S> for Flattener
where
    N: AsStaticNode + for<'a> Generated<FlattenedSampler<'a, S>, W>,
    W: for<'a> GeneratorTuple<N, FlattenedSampler<'a, S>>,
    S: Sampler<N>,
{
    fn generate(&mut self, sampler: &mut S, with: &mut W) -> Option<N> {
        if self.targets.contains(&N::static_definition()) {
            let flattened = self.flattened.get(&N::static_definition()).unwrap();
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

#[cfg(feature = "dynamic")]
#[allow(deprecated)]
mod dynamic_impls {
    use crate::dynamic::{DynamicNode, HasDynamicSampler};
    use crate::generation::util::{
        flatten, FandangoNode, FlattenedSampler, Flattener, Unflattenable,
    };
    use crate::generation::{Generated, Generator, GeneratorTuple, Sampler};
    use crate::impl_has_dynamic_sampler;
    use alloc::vec;

    impl Flattener {
        /// Adds a node to the flattening list by root and definition.
        pub fn flatten_dynamic(
            mut self,
            root: FandangoNode,
            definition: FandangoNode,
        ) -> Result<Self, Unflattenable> {
            let nonterminals = match root {
                FandangoNode::Program(p) => p.nonterminals(),
                _ => panic!("Invalid root provided."),
            };

            flatten(&nonterminals, definition, &mut vec![], &mut self.flattened)?;

            self.targets.insert(definition);
            Ok(self)
        }
    }

    impl<S> Sampler<DynamicNode> for FlattenedSampler<'_, S>
    where
        S: Sampler<DynamicNode> + HasDynamicSampler,
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
                .get(&self.sampler.definition())
                .expect("Attempted to generate something that wasn't in the graph");
            self.sample_alternative_from(current)
        }
    }

    impl<W, S> Generator<DynamicNode, W, S> for Flattener
    where
        W: for<'a> GeneratorTuple<DynamicNode, FlattenedSampler<'a, S>>,
        S: Sampler<DynamicNode> + HasDynamicSampler,
    {
        fn generate(&mut self, sampler: &mut S, with: &mut W) -> Option<DynamicNode> {
            if self.targets.contains(&sampler.definition()) {
                let flattened = self.flattened.get(&sampler.definition()).unwrap();
                let choice = sampler.sample_alternative(flattened.total);
                let mut sampler = FlattenedSampler {
                    choice,
                    flattened: &self.flattened,
                    sampler,
                };
                Some(
                    with.generate(&mut sampler)
                        .unwrap_or_else(|| DynamicNode::generate(&mut sampler, with)),
                )
            } else {
                None
            }
        }
    }

    impl<S> HasDynamicSampler for FlattenedSampler<'_, S>
    where
        S: HasDynamicSampler,
    {
        impl_has_dynamic_sampler!(sampler);
    }
}
