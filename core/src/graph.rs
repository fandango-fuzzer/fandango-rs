//! Graph operations for lifting FANDANGO grammars to a graph.

use crate::lang::constraints::{
    Atom, BaseSelection, Comparison, Conjunction, Constraint, ConstraintOperator, Disjunction,
    Expr, Implies, Inversion, Quantifier, QuantifierSpecification, RsPair, RsPairs, RsSlice,
    RsSlices, Selection, Selector, SelectorLength,
};
use crate::lang::{
    Alternative, Concatenation, Nonterminal, Operator, Production, Program, Statement, Symbol,
    Tagged,
};
use core::fmt::{Formatter, Write};
use pest::Span;
use petgraph::graph;
use petgraph::graph::DiGraph;
use petgraph::graphmap::NodeTrait;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fmt::Display;
use std::hash::Hash;

/// Traverse this type's children, potentially recursively, for use with grammar graph creation.
#[allow(unused_variables)]
pub trait GraphTraverse<'program>: Sized {
    /// The node type for graph.
    type Node: NodeTrait + GraphTraverse<'program, Node = Self::Node> + From<Self>;

    /// Recurse the traversal! The order of calls to `consumer` are not guaranteed, but ultimately
    /// will invoke [`GraphTraverse::traverse`] for each level.
    fn recurse<F>(self, mut consumer: F)
    where
        F: FnMut(Self::Node, Self::Node, Span<'program>),
    {
        self.traverse(|n1, n2, w| {
            consumer(n1, n2, w);
            n2.traverse(&mut consumer)
        })
    }

    /// Traverse a single level from this node. The `consumer` function should accept two nodes, the
    /// parent and the child, as well as an unsigned integer denoting the index of the children.
    fn traverse<F>(self, consumer: F)
    where
        F: FnMut(Self::Node, Self::Node, Span<'program>),
    {
    }
}

/// Call `consumer` for each `i, child` in `children.enumerate()` with `consumer(parent, child, i)`.
pub fn traverse_children<'program, 'source, F, T>(
    parent: T,
    children: impl Iterator<Item = (T::Node, Span<'source>)>,
    mut consumer: F,
) where
    F: FnMut(T::Node, T::Node, Span<'source>),
    T: GraphTraverse<'program>,
    'source: 'program,
{
    let node = parent.into();
    for (child, weight) in children {
        consumer(node, child, weight);
    }
}

/// Internal macro for compiling iterator chains, for use in [`crate::impl_traverse`].
#[macro_export]
macro_rules! chain_field_iter {
    ($node:ty $(=> $from:tt)?, $current:expr) => {
        $current
    };

    ($node:ty $(=> $from:tt)?, $current:expr, $field:tt) => {
        $current.chain($crate::field_iter!($node $(=> $from)?, $field))
    };

    ($node:ty $(=> $from:tt)?, $current:expr, $field:tt, $($fields:tt),+) => {{
        let next = $crate::chain_field_iter!($node $(=> $from)?, $current, $field);
        $crate::chain_field_iter!($node $(=> $from)?, next, $($fields),+)
    }};
}

/// Internal macro for producing iterators over fields, for use in [`crate::impl_traverse`].
#[macro_export]
macro_rules! field_iter {
    ($node:ty $(=> $from:tt)?) => {
        ::core::iter::empty()
    };

    ($node:ty $(=> $from:tt)?, [ $field:tt ]) => {
        $($from.)? $field.iter().map(::core::convert::From::from)
    };

    ($node:ty $(=> $from:tt)?, $field:tt) => {
        ::core::iter::once((&$($from.)? $field)).map(::core::convert::From::from)
    };

    ($node:ty $(=> $from:tt)?, $field:tt, $($fields:tt),+) => {{
        let next = $crate::field_iter!($node $(=> $from)?, $field);
        $crate::chain_field_iter!($node $(=> $from)?, next, $($fields),+)
    }};
}

/// Internal macro for compiling iterator chains, for use in [`crate::impl_traverse`], over enums.
#[macro_export]
macro_rules! variant_traverse {
    ($from:tt, $consumer:tt, $node:ty, @($variant:tt { $($bindings:tt),+ } { $($iteration:tt)+ } { } $(, $($variants:tt)+)?), $($emitted:tt)*) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @($($($variants)+)?),
            $($emitted)*
            $variant($($bindings),+) => { $crate::graph::traverse_children($from, $($iteration)+, $consumer) }
        )
    };

    ($from:tt, $consumer:tt, $node:ty, @($variant:tt { $($bindings:tt),+ } { $($iteration:tt)+ } { _ $(, $($remaining:tt),+)? } $(, $($variants:tt)+)?), $($emitted:tt)*) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @(
                $variant
                { $($bindings),+, _ }
                { $($iteration)+ }
                { $($($remaining),+)? }
                $(, $($variants)+)?
            ),
            $($emitted)*
        )
    };

    ($from:tt, $consumer:tt, $node:ty, @($variant:tt { $($bindings:tt),+ } { $($iteration:tt)+ } { $next:tt $(, $($remaining:tt),+)? } $(, $($variants:tt)+)?), $($emitted:tt)*) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @(
                $variant
                { $($bindings),+, $next }
                { $($iteration)+.chain(::core::iter::once($next).map(::core::convert::From::from)) }
                { $($($remaining),+)? }
                $(, $($variants)+)?
            ),
            $($emitted)*
        )
    };

    ($from:tt, $consumer:tt, $node:ty, @($variant:tt { $($bindings:tt),+ } { $($iteration:tt)+ } { [ $next:tt ] $(, $($remaining:tt),+)? } $(, $($variants:tt)+)?), $($emitted:tt)*) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @(
                $variant
                { $($bindings),+, $next }
                { $($iteration)+.chain($next.iter().map(::core::convert::From::from)) }
                { $($($remaining),+)? }
                $(, $($variants)+)?
            ),
            $($emitted)*
        )
    };

    ($from:tt, $consumer:tt, $node:ty, @($variant:tt { } { } { $next:tt $(, $($remaining:tt),+)? } $(, $($variants:tt)+)?), $($emitted:tt)*) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @(
                $variant
                { $next }
                { ::core::iter::once($next).map(::core::convert::From::from) }
                { $($($remaining),+)? }
                $(, $($variants)+)?
            ),
            $($emitted)*
        )
    };

    ($from:tt, $consumer:tt, $node:ty, @($variant:tt { } { } { _ $(, $($remaining:tt),+)? } $(, $($variants:tt)+)?), $($emitted:tt)*) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @(
                $variant
                { }
                { }
                { $($($remaining),+)? }
                $(, $($variants)+)?
            ),
            $($emitted)*
        )
    };

    ($from:tt, $consumer:tt, $node:ty, @($variant:tt { } { } { [ $next:tt ] $(, $($remaining:tt),+)? } $(, $($variants:tt)+)?), $($emitted:tt)*) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @(
                $variant
                { $next }
                { $next.iter().map(::core::convert::From::from) }
                { $($($remaining),+)? }
                $(, $($variants)+)?
            ),
            $($emitted)*
        )
    };

    ($from:tt, $consumer:tt, $node:ty, @($variant:tt $(, $($variants:tt)+)?), $($emitted:tt)*) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @($($($variants)+)?),
            $($emitted)*
            $variant => {  }
        )
    };

    ($from:tt, $consumer:tt, $node:ty, @($variant:tt ( $($options:tt),+ ) $(, $($variants:tt)+)?), $($emitted:tt)*) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @(
                $variant
                { }
                { }
                { $($options),+ }
                $(, $($variants)+)?
            ),
            $($emitted)*
        )
    };

    ($from:tt, $consumer:tt, $node:ty, { $($variants:tt)+ }) => {
        $crate::variant_traverse!(
            $from,
            $consumer,
            $node,
            @($($variants)+),
        )
    };

    ($from:tt, $consumer:tt, $node:ty, @(), $($emitted:tt)*) => {
        match $from {
            $($emitted)*
        }
    };
}

/// Macro which generates implementations of [`GraphTraverse`] over fields of the provided struct or
/// variants of the provided enum. The lifetime `'source` is already within the lifetime list and
/// corresponds to the lifetime of the source code.
///
/// The first four fields are, in order:
/// 1. The type for which [`GraphTraverse`] is to be implemented.
/// 2. The name of the raw type (e.g., if providing the type behind a reference).
/// 3. The node type of the graph (e.g., [`FandangoNode`]).
/// 4. The generics/lifetimes required for the implementation (optional).
///
/// The remaining argument(s) are a variadic list of fields or a list of match-like enum pattern
/// bindings without the `=> { ... }` clause, surrounded by `match { }`. Fields or enum bindings
/// which are surrounded by `[]` will be interpreted as iterables and those without will be
/// considered as something which can be immediately [`Into::into`]'d into the corresponding node
/// type. The order of variables will be preserved, and the enumeration will take place over the
/// combined iterator.
///
/// This enum can simplify the implementation of traversal for structures with a variety of layouts.
/// For example, `statements` here has an `iter` method, the items for which implement [`Into`] for
/// the node type [`FandangoNode`].
/// ```rust,ignore
/// # use fandango::graph::FandangoNode;
/// # use fandango::lang::Program;
/// fandango::impl_traverse!(
///     &'program Program<'source>,
///     Program,
///     FandangoNode<'program, 'source>,
///     <'source: 'program>,
///     [statements]
/// );
/// ```
///
/// It is also possible to do this for enums across multiple variants:
/// ```rust,ignore
/// # use fandango::graph::FandangoNode;
/// # use fandango::lang::Statement;
/// fandango::impl_traverse!(
///     &'program Statement<'source>,
///     Statement,
///     FandangoNode<'program, 'source>,
///     <'source: 'program>,
///     match { Production(prod), Constraint, Python }
/// );
/// ```
#[macro_export]
macro_rules! impl_traverse {
    ($target:ty, $name:ty, $node:ty, < $($generics:tt $(: $constraints:tt)?),* >, match { $($variants:tt)+ }) => {
        impl<'program, $($generics $(: $constraints)?),*> $crate::graph::GraphTraverse<'program> for $target {
            type Node = $node;

            fn traverse<F>(self, consumer: F)
            where
                F: ::core::ops::FnMut(Self::Node, Self::Node, $crate::lang::Span<'program>),
            {
                #![allow(unused_imports)]
                use $name::*;
                $crate::variant_traverse!(self, consumer, $node, { $($variants)+ })
            }
        }
    };

    ($target:ty, $name:ty, $node:ty, < $($generics:tt $(: $constraints:tt)?),* >, $($fields:tt),*) => {
        impl<'program, $($generics $(: $constraints)?),*> $crate::graph::GraphTraverse<'program> for $target {
            type Node = $node;

            fn traverse<F>(self, consumer: F)
            where
                F: ::core::ops::FnMut(Self::Node, Self::Node,  $crate::lang::Span<'program>),
            {
                $crate::graph::traverse_children(self, $crate::field_iter!($node => self, $($fields),*), consumer);
            }
        }
    };

    ($target:ty, $name:ty, $node:ty, match { $($variants:tt)+ }) => {
        $crate::impl_traverse!($target, $name, $node, <>, { $($variants)+ })
    };

    ($target:ty, $name:ty, $node:ty, $($fields:tt),*) => {
        $crate::impl_traverse!($target, $name, $node, <>, $($fields),*)
    };
}

macro_rules! impl_fandango_traverse {
    ($target:tt, match { $($variants:tt)+ }) => {
        impl_traverse!(
            &'program $target<'source>,
            $target,
            FandangoNode<'program, 'source>,
            <'source: 'program>,
            match { $($variants)+ }
        );
    };

    ($target:tt, $($fields:tt),*) => {
        impl_traverse!(
            &'program $target<'source>,
            $target,
            FandangoNode<'program, 'source>,
            <'source: 'program>,
            $($fields),*
        );
    };
}

/// Convert a type which implements [`GraphTraverse`] into a [`DiGraph`].
pub trait IntoGraph<'program>: GraphTraverse<'program> {
    /// Perform the conversion.
    fn into_graph(
        self,
    ) -> (
        HashMap<Self::Node, graph::NodeIndex>,
        DiGraph<Self::Node, Span<'program>>,
    );
}

impl<'program, 'source, T> IntoGraph<'program> for T
where
    T: GraphTraverse<'program, Node = FandangoNode<'program, 'source>>,
    'source: 'program,
{
    fn into_graph(
        self,
    ) -> (
        HashMap<FandangoNode<'program, 'source>, graph::NodeIndex>,
        DiGraph<Self::Node, Span<'program>>,
    ) {
        let mut graph = DiGraph::new();
        let mut work = VecDeque::new();
        self.traverse(|n1, n2, w| work.push_back((n1, n2, w)));

        let mut node_indices: HashMap<FandangoNode<'program, 'source>, graph::NodeIndex> =
            HashMap::new();
        let mut idx = |g: &mut DiGraph<FandangoNode<'program, 'source>, _>, n| {
            *node_indices.entry(n).or_insert_with(|| g.add_node(n))
        };

        while let Some((n1, n2, w)) = work.pop_front() {
            match n1 {
                FandangoNode::Production(_) if matches!(n2, FandangoNode::Nonterminal(_)) => {
                    let n1 = idx(&mut graph, n1);
                    let n2 = idx(&mut graph, n2);
                    graph.add_edge(n1, n2, w);
                }
                FandangoNode::Production(prod) => {
                    work.push_back((prod.nonterminal().into(), n2, w));
                }
                FandangoNode::Alternative(alt) if alt.concatenations().len() == 1 => {
                    n2.traverse(|n1, n2, w| work.push_back((n1, n2, w)))
                }
                FandangoNode::Concatenation(concats) if concats.operators().len() == 1 => {
                    n2.traverse(|n1, n2, w| work.push_back((n1, n2, w)))
                }
                FandangoNode::Nonterminal(_)
                | FandangoNode::Alternative(_)
                | FandangoNode::Concatenation(_)
                | FandangoNode::Operator(_) => match n2 {
                    FandangoNode::Alternative(alt) if alt.concatenations().len() == 1 => {
                        n2.traverse(|_, n2, w| work.push_back((n1, n2, w)))
                    }
                    FandangoNode::Concatenation(concats) if concats.operators().len() == 1 => {
                        n2.traverse(|_, n2, w| work.push_back((n1, n2, w)))
                    }
                    FandangoNode::Alternative(_)
                    | FandangoNode::Concatenation(_)
                    | FandangoNode::Operator(_)
                        if !matches!(n2, FandangoNode::Operator(Operator::Symbol(_))) =>
                    {
                        {
                            let n1 = idx(&mut graph, n1);
                            let n2 = idx(&mut graph, n2);
                            graph.add_edge(n1, n2, w);
                        }
                        n2.traverse(|n1, n2, w| work.push_back((n1, n2, w)))
                    }
                    FandangoNode::Nonterminal(_) | FandangoNode::String(_) => {
                        let n1 = idx(&mut graph, n1);
                        let n2 = idx(&mut graph, n2);
                        graph.add_edge(n1, n2, w);
                    }
                    _ => n2.traverse(|_, n2, w| work.push_back((n1, n2, w))),
                },
                _ => n2.traverse(|n1, n2, w| work.push_back((n1, n2, w))),
            }
        }

        (node_indices, graph)
    }
}

/// The node type used to represent the grammar's graph.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(missing_docs)]
pub enum FandangoNode<'program, 'source> {
    // Core grammar components
    Program(&'program Program<'source>),
    Statement(&'program Statement<'source>),
    Production(&'program Production<'source>),
    Nonterminal(&'program Nonterminal<'source>),
    Alternative(&'program Alternative<'source>),
    Concatenation(&'program Concatenation<'source>),
    Operator(&'program Operator<'source>),
    Symbol(&'program Symbol<'source>),
    String(&'program Tagged<'source, Cow<'source, str>>),
    // Constraint components
    Constraint(&'program Constraint<'source>),
    Implies(&'program Implies<'source>),
    Quantifier(&'program Quantifier<'source>),
    QuantifierSpecification(&'program QuantifierSpecification<'source>),
    Disjunction(&'program Disjunction<'source>),
    Conjunction(&'program Conjunction<'source>),
    Atom(&'program Atom<'source>),
    Comparison(&'program Comparison<'source>),
    ConstraintOperator(&'program Tagged<'source, ConstraintOperator>),
    Expr(&'program Expr<'source>),
    SelectorLength(&'program SelectorLength<'source>),
    Selection(&'program Selection<'source>),
    Selector(&'program Selector<'source>),
    BaseSelection(&'program BaseSelection<'source>),
    RsPairs(&'program RsPairs<'source>),
    RsPair(&'program RsPair<'source>),
    RsSlices(&'program RsSlices<'source>),
    RsSlice(&'program Tagged<'source, RsSlice>),
    Inversion(&'program Inversion<'source>),
}

impl Display for FandangoNode<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FandangoNode::Program(_) => f.write_str("PROG"),
            FandangoNode::Statement(_) => f.write_str("STMT"),
            FandangoNode::Production(_) => f.write_str("PROD"),
            FandangoNode::Nonterminal(nt) => {
                f.write_char('<')?;
                f.write_str(nt.name())?;
                f.write_char('>')
            }
            FandangoNode::Alternative(_) => f.write_char('|'),
            FandangoNode::Concatenation(_) => f.write_char('~'),
            FandangoNode::Operator(op) => match op {
                Operator::Kleene(_) => f.write_char('*'),
                Operator::Plus(_) => f.write_char('+'),
                Operator::Option(_) => f.write_char('?'),
                Operator::Repeat(_, start, end) => f.write_str(&format!("{{{},{}}}", start, end)),
                Operator::Symbol(_) => f.write_str("OP"),
            },
            FandangoNode::Symbol(_) => f.write_str("SYM"),
            FandangoNode::String(s) => fmt::Debug::fmt(s, f),
            FandangoNode::Constraint(_) => f.write_str("CNST"),
            FandangoNode::Implies(_) => f.write_str("=>"),
            FandangoNode::Quantifier(Quantifier::Forall(_)) => f.write_char('∀'),
            FandangoNode::Quantifier(Quantifier::Exists(_)) => f.write_char('∃'),
            FandangoNode::Quantifier(Quantifier::Disjunction(_)) => f.write_str("QNTF"),
            FandangoNode::QuantifierSpecification(_) => f.write_str("QSPEC"),
            FandangoNode::Disjunction(_) => f.write_str("DISJ"),
            FandangoNode::Conjunction(_) => f.write_str("CONJ"),
            FandangoNode::Atom(_) => f.write_str("ATOM"),
            FandangoNode::Comparison(_) => f.write_str("CMP"),
            FandangoNode::ConstraintOperator(op) => match op.inner() {
                ConstraintOperator::Neq => f.write_str("!="),
                ConstraintOperator::Lt => f.write_char('<'),
                ConstraintOperator::LtEq => f.write_str("<="),
                ConstraintOperator::Eq => f.write_str("=="),
                ConstraintOperator::GtEq => f.write_str(">="),
                ConstraintOperator::Gt => f.write_char('>'),
            },
            FandangoNode::Expr(_) => f.write_str("EXPR"),
            FandangoNode::SelectorLength(SelectorLength::WithLength(_)) => f.write_str("COUNT"),
            FandangoNode::SelectorLength(SelectorLength::NoLength(_)) => f.write_str("SELECT"),
            FandangoNode::Selector(_) => f.write_str("SELECTOR"),
            FandangoNode::Selection(Selection::OverSlices(_, _))
            | FandangoNode::Selection(Selection::OverPairs(_, _)) => f.write_str("OVER"),
            FandangoNode::Selection(Selection::Basic(_)) => f.write_str("DIRECT"),
            FandangoNode::BaseSelection(BaseSelection::Nonterminal(_)) => f.write_str("EXACT"),
            FandangoNode::BaseSelection(BaseSelection::Selector(_)) => f.write_str("SELECTED"),
            FandangoNode::RsPairs(_) => f.write_str("PAIRS"),
            FandangoNode::RsPair(_) => f.write_str("PAIR"),
            FandangoNode::RsSlices(_) => f.write_str("SLICES"),
            FandangoNode::RsSlice(s) => Display::fmt(s.inner(), f),
            FandangoNode::Inversion(Inversion::Selector(_)) => f.write_str("SELECTED"),
            FandangoNode::Inversion(Inversion::Stringified(_)) => f.write_str("STRINGIFIED"),
        }
    }
}

impl<'program, 'source> GraphTraverse<'program> for FandangoNode<'program, 'source>
where
    'source: 'program,
{
    type Node = Self;

    fn traverse<F>(self, consumer: F)
    where
        F: FnMut(Self::Node, Self::Node, Span<'program>),
    {
        match self {
            FandangoNode::Program(s) => s.traverse(consumer),
            FandangoNode::Statement(s) => s.traverse(consumer),
            FandangoNode::Production(s) => s.traverse(consumer),
            FandangoNode::Alternative(s) => s.traverse(consumer),
            FandangoNode::Concatenation(s) => s.traverse(consumer),
            FandangoNode::Operator(s) => s.traverse(consumer),
            FandangoNode::Symbol(s) => s.traverse(consumer),
            FandangoNode::Nonterminal(s) => s.traverse(consumer),
            FandangoNode::Constraint(s) => s.traverse(consumer),
            FandangoNode::Implies(s) => s.traverse(consumer),
            FandangoNode::Quantifier(s) => s.traverse(consumer),
            FandangoNode::QuantifierSpecification(s) => s.traverse(consumer),
            FandangoNode::Disjunction(s) => s.traverse(consumer),
            FandangoNode::Conjunction(s) => s.traverse(consumer),
            FandangoNode::Atom(s) => s.traverse(consumer),
            FandangoNode::Comparison(s) => s.traverse(consumer),
            FandangoNode::Expr(s) => s.traverse(consumer),
            FandangoNode::SelectorLength(s) => s.traverse(consumer),
            FandangoNode::Selection(s) => s.traverse(consumer),
            FandangoNode::Selector(s) => s.traverse(consumer),
            FandangoNode::BaseSelection(s) => s.traverse(consumer),
            FandangoNode::RsPairs(s) => s.traverse(consumer),
            FandangoNode::RsPair(s) => s.traverse(consumer),
            FandangoNode::RsSlices(s) => s.traverse(consumer),
            FandangoNode::Inversion(s) => s.traverse(consumer),
            // nothing to do in these cases; they are terminals
            FandangoNode::String(_) => {}
            FandangoNode::ConstraintOperator(_) => {}
            FandangoNode::RsSlice(_) => {}
        }
    }
}

/// Transforms a full grammar tree into a node describing only the head.
pub trait IntoNode {
    /// The node type which this tree transforms to.
    type Node;

    /// Perform the conversion!
    fn into_node(self) -> Self::Node;
}

macro_rules! impl_node_from {
    ($node:tt) => {
        impl<'program, 'source> From<&'program $node<'source>> for FandangoNode<'program, 'source> {
            fn from(value: &'program $node<'source>) -> Self {
                Self::$node(value)
            }
        }

        impl<'program, 'source> From<&'program Tagged<'source, $node<'source>>>
            for FandangoNode<'program, 'source>
        {
            fn from(value: &'program Tagged<'source, $node<'source>>) -> Self {
                Self::$node(value.inner())
            }
        }
    };
}

impl_node_from!(Program);
impl_node_from!(Statement);
impl_node_from!(Production);
impl_node_from!(Nonterminal);
impl_node_from!(Alternative);
impl_node_from!(Concatenation);
impl_node_from!(Operator);
impl_node_from!(Symbol);
impl_node_from!(Constraint);
impl_node_from!(Implies);
impl_node_from!(Quantifier);
impl_node_from!(QuantifierSpecification);
impl_node_from!(Disjunction);
impl_node_from!(Conjunction);
impl_node_from!(Atom);
impl_node_from!(Comparison);
impl_node_from!(Expr);
impl_node_from!(SelectorLength);
impl_node_from!(Selector);
impl_node_from!(Selection);
impl_node_from!(BaseSelection);
impl_node_from!(RsPairs);
impl_node_from!(RsPair);
impl_node_from!(RsSlices);
impl_node_from!(Inversion);

impl<'program, 'source> From<&'program Tagged<'source, Cow<'source, str>>>
    for FandangoNode<'program, 'source>
{
    fn from(value: &'program Tagged<'source, Cow<'source, str>>) -> Self {
        Self::String(value)
    }
}

impl<'program, 'source> From<&'program Tagged<'source, ConstraintOperator>>
    for FandangoNode<'program, 'source>
{
    fn from(value: &'program Tagged<'source, ConstraintOperator>) -> Self {
        Self::ConstraintOperator(value)
    }
}

impl<'program, 'source> From<&'program Tagged<'source, RsSlice>>
    for FandangoNode<'program, 'source>
{
    fn from(value: &'program Tagged<'source, RsSlice>) -> Self {
        Self::RsSlice(value)
    }
}

#[cfg(test)]
mod test {
    use crate::graph::{FandangoNode, IntoGraph};
    use crate::lang::test::SIMPLE_GRAMMAR;
    use crate::lang::Program;
    use petgraph::data::{Element, FromElements};
    use petgraph::dot::{Config, Dot};
    use petgraph::graph::DiGraph;
    use std::error::Error;

    // this doesn't really test anything, just produces a graph in GraphViz format
    #[test]
    fn test_graph() -> Result<(), Box<dyn Error>> {
        let program = Program::try_from(SIMPLE_GRAMMAR)?;

        let (_, graph) = (&program).into_graph();

        let renderable = DiGraph::<_, _>::from_elements(
            graph
                .raw_nodes()
                .iter()
                .map(|n| Element::Node { weight: n.weight })
                .chain(graph.raw_edges().iter().map(|e| {
                    let (start_line, start_col) = e.weight.start_pos().line_col();
                    let (end_line, end_col) = e.weight.end_pos().line_col();
                    let rendered = if start_line == end_line {
                        format!("{start_line}:{start_col}-{end_col}")
                    } else {
                        format!("{start_line}:{start_col}-{end_line}:{end_col}")
                    };
                    Element::Edge {
                        source: e.source().index(),
                        target: e.target().index(),
                        weight: rendered,
                    }
                })),
        );

        let rendered = Dot::with_attr_getters(
            &renderable,
            &[Config::NodeNoLabel, Config::EdgeNoLabel],
            &|_, e| format!("label = {:?}", e.weight()),
            &|_, (_, node)| {
                format!(
                    "label = {:?}",
                    match node {
                        FandangoNode::String(s) => format!("{}", s.inner()),
                        _ => format!("{}", node),
                    }
                )
            },
        );

        println!("{rendered}");

        Ok(())
    }
}
