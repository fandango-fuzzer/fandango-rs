//! Graph operations for lifting FANDANGO grammars to a graph.

use crate::lang::{
    Alternative, Concatenation, Nonterminal, Operator, Production, Program, Statement, Symbol,
};
use alloc::borrow::Cow;
use core::fmt::{Formatter, Write};
use petgraph::data::Build;
use petgraph::graphmap::{DiGraphMap, NodeTrait};
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Traverse this type's children, potentially recursively.
#[allow(unused_variables)]
pub trait Traverse: Sized {
    /// The node type for graph.
    type Node: NodeTrait + Traverse<Node = Self::Node> + From<Self>;

    /// Recurse the traversal! The order of calls to `consumer` are not guaranteed, but ultimately
    /// will invoke [`Traverse::traverse`] for each level.
    fn recurse<F>(self, mut consumer: F)
    where
        F: FnMut(Self::Node, Self::Node, usize),
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
        F: FnMut(Self::Node, Self::Node, usize),
    {
    }
}

/// Call `consumer` for each `i, child` in `children.enumerate()` with `consumer(parent, child, i)`.
pub fn traverse_children<'program, F, T>(
    parent: T,
    children: impl Iterator<Item = T::Node>,
    mut consumer: F,
) where
    F: FnMut(T::Node, T::Node, usize),
    T: Traverse,
{
    let node = parent.into();
    for (weight, child) in children.enumerate() {
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

    ($node:ty $(=> $from:tt)?, $current:expr, $field:tt, $($fields:tt),+) => {
        let next = $crate::chain_field_iter!($node $(=> $from)?, $current, $field);
        $crate::chain_field_iter!($node $(=> $from)?, next, $($fields),+)
    };
}

/// Internal macro for producing iterators over fields, for use in [`crate::impl_traverse`].
#[macro_export]
macro_rules! field_iter {
    ($node:ty $(=> $from:tt)?) => {
        ::core::iter::empty()
    };

    ($node:ty $(=> $from:tt)?, [ $field:tt ]) => {
        $($from.)? $field.iter().map(<$node>::from)
    };

    ($node:ty $(=> $from:tt)?, $field:tt) => {
        ::core::iter::once(<$node>::from(&$($from.)? $field))
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
                { $($iteration)+.chain(::core::iter::once(<$node>::from($next))) }
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
                { $($iteration)+.chain($next.iter().map(::core::convert::Into::into)) }
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
                { ::core::iter::once(<$node>::from($next)) }
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
                { $next.iter().map(<$node>::from) }
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

/// Macro which generates implementations of [`Traverse`] over fields of the provided struct or
/// variants of the provided enum.
///
/// The first four fields are, in order:
/// 1. The type for which [`Traverse`] is to be implemented.
/// 2. The name of the raw type (e.g., if providing the type behind a reference).
/// 3. The node type of the graph (e.g., [`FandangoNode`]).
/// 4. The generics/lifetimes required for the implementation (optional; where clauses not supported).
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
/// ```no_run
/// # use fandango::graph::FandangoNode;
/// # use fandango::lang::Program;
/// fandango::impl_traverse!(
///     &'program Program<'source>,
///     Program,
///     FandangoNode<'program, 'source>,
///     <'program, 'source>,
///     [statements]
/// );
/// ```
///
/// It is also possible to do this for enums across multiple variants:
/// ```no_run
/// # use fandango::graph::FandangoNode;
/// # use fandango::lang::Statement;
/// fandango::impl_traverse!(
///     &'program Statement<'source>,
///     Statement,
///     FandangoNode<'program, 'source>,
///     <'program, 'source>,
///     match { Production(prod), Constraint, Python }
/// );
/// ```
#[macro_export]
macro_rules! impl_traverse {
    ($target:ty, $name:ty, $node:ty, < $($generics:tt),* >, match { $($variants:tt)+ }) => {
        impl<$($generics),*> $crate::graph::Traverse for $target {
            type Node = $node;

            fn traverse<F>(self, consumer: F)
            where
                F: ::core::ops::FnMut(Self::Node, Self::Node, usize),
            {
                #![allow(unused_imports)]
                use $name::*;
                $crate::variant_traverse!(self, consumer, $node, { $($variants)+ })
            }
        }
    };

    ($target:ty, $name:ty, $node:ty, < $($generics:tt),* >, $($fields:tt),*) => {
        impl<$($generics),*> $crate::graph::Traverse for $target {
            type Node = $node;

            fn traverse<F>(self, consumer: F)
            where
                F: ::core::ops::FnMut(Self::Node, Self::Node, usize),
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
            <'program, 'source>,
            match { $($variants)+ }
        );
    };

    ($target:tt, $($fields:tt),*) => {
        impl_traverse!(
            &'program $target<'source>,
            $target,
            FandangoNode<'program, 'source>,
            <'program, 'source>,
            $($fields),*
        );
    };
}

/// Convert a type which implements [`Traverse`] into a [`DiGraphMap`].
pub trait IntoGraph: Traverse {
    /// Perform the conversion.
    fn into_graph(self) -> DiGraphMap<Self::Node, usize>;
}

impl<'program, 'source, T> IntoGraph for T
where
    T: Traverse<Node = FandangoNode<'program, 'source>>,
    'source: 'program,
{
    fn into_graph(self) -> DiGraphMap<Self::Node, usize> {
        let mut graph = DiGraphMap::new();
        let mut work = VecDeque::new();
        self.traverse(|n1, n2, w| work.push_back((n1, n2, w)));

        while let Some((n1, n2, w)) = work.pop_front() {
            match n1 {
                FandangoNode::Production(prod) => {
                    if let Some(w) = w.checked_sub(1) {
                        work.push_back((prod.nonterminal().into(), n2, w));
                    }
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
                        n2.traverse(|_, n2, _| work.push_back((n1, n2, w)))
                    }
                    FandangoNode::Concatenation(concats) if concats.operators().len() == 1 => {
                        n2.traverse(|_, n2, _| work.push_back((n1, n2, w)))
                    }
                    FandangoNode::Alternative(_)
                    | FandangoNode::Concatenation(_)
                    | FandangoNode::Operator(_)
                        if !matches!(n2, FandangoNode::Operator(Operator::Symbol(_))) =>
                    {
                        graph.update_edge(n1, n2, w);
                        n2.traverse(|n1, n2, w| work.push_back((n1, n2, w)))
                    }
                    FandangoNode::Nonterminal(_)
                    | FandangoNode::String(_)
                    | FandangoNode::Bytes(_) => {
                        graph.update_edge(n1, n2, w);
                    }
                    _ => n2.traverse(|_, n2, _| work.push_back((n1, n2, w))),
                },
                _ => n2.traverse(|n1, n2, w| work.push_back((n1, n2, w))),
            }
        }

        graph
    }
}

/// The node type used to represent the grammar's graph.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum FandangoNode<'program, 'source> {
    Program(&'program Program<'source>),
    Statement(&'program Statement<'source>),
    Production(&'program Production<'source>),
    Nonterminal(&'program Nonterminal<'source>),
    Alternative(&'program Alternative<'source>),
    Concatenation(&'program Concatenation<'source>),
    Operator(&'program Operator<'source>),
    Symbol(&'program Symbol<'source>),
    String(&'program Cow<'source, str>),
    Bytes(&'program Cow<'source, [u8]>),
}

impl fmt::Display for FandangoNode<'_, '_> {
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
                Operator::Repeat(_, range) => {
                    f.write_str(&format!("{{{},{}}}", range.start(), range.end()))
                }
                Operator::Symbol(_) => f.write_str("OP"),
            },
            FandangoNode::Symbol(_) => f.write_str("SYM"),
            FandangoNode::String(s) => fmt::Debug::fmt(s, f),
            FandangoNode::Bytes(b) => fmt::Debug::fmt(b, f),
        }
    }
}

impl fmt::Debug for FandangoNode<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FandangoNode::Program(_) => f
                .debug_struct("FandangoNode::Program")
                .finish_non_exhaustive(),
            FandangoNode::Statement(_) => f
                .debug_struct("FandangoNode::Statement")
                .finish_non_exhaustive(),
            FandangoNode::Production(_) => f
                .debug_struct("FandangoNode::Production")
                .finish_non_exhaustive(),
            FandangoNode::Nonterminal(nt) => f
                .debug_struct("FandangoNode::Nonterminal")
                .field("name", nt.name())
                .finish(),
            FandangoNode::Alternative(_) => f
                .debug_struct("FandangoNode::Alternative")
                .finish_non_exhaustive(),
            FandangoNode::Concatenation(_) => f
                .debug_struct("FandangoNode::Concatenation")
                .finish_non_exhaustive(),
            FandangoNode::Operator(op) => {
                let mut debug = f.debug_struct("FandangoNode::Operator");
                debug.field(
                    "variant",
                    &match op {
                        Operator::Kleene(_) => "Kleene",
                        Operator::Plus(_) => "Plus",
                        Operator::Option(_) => "Option",
                        Operator::Repeat(_, _) => "Repeat",
                        Operator::Symbol(_) => "Symbol",
                    },
                );
                if let Operator::Repeat(_, range) = op {
                    debug.field("count", &format!("{{{},{}}}", range.start(), range.end()));
                }
                debug.finish_non_exhaustive()
            }
            FandangoNode::Symbol(_) => f
                .debug_struct("FandangoNode::Symbol")
                .finish_non_exhaustive(),
            FandangoNode::String(s) => f
                .debug_struct("FandangoNode::String")
                .field("content", s)
                .finish(),
            FandangoNode::Bytes(b) => f
                .debug_struct("FandangoNode::Bytes")
                .field("content", b)
                .finish(),
        }
    }
}

impl Traverse for FandangoNode<'_, '_> {
    type Node = Self;

    fn traverse<F>(self, consumer: F)
    where
        F: FnMut(Self::Node, Self::Node, usize),
    {
        match self {
            FandangoNode::Program(s) => s.traverse(consumer),
            FandangoNode::Statement(s) => s.traverse(consumer),
            FandangoNode::Production(s) => s.traverse(consumer),
            FandangoNode::Alternative(s) => s.traverse(consumer),
            FandangoNode::Concatenation(s) => s.traverse(consumer),
            FandangoNode::Operator(s) => s.traverse(consumer),
            FandangoNode::Symbol(s) => s.traverse(consumer),
            FandangoNode::Nonterminal(_) | FandangoNode::String(_) | FandangoNode::Bytes(_) => {} // nothing to do
        }
    }
}

macro_rules! impl_node_from {
    ($node:tt, $actual:tt, $($rest:tt),+) => {
        impl<'program, 'source> From<&'program $actual<'source, $($rest),+>> for FandangoNode<'program, 'source> {
            fn from(value: &'program $actual<'source, $($rest),+>) -> Self {
                Self::$node(value)
            }
        }
    };

    ($node:tt) => {
        impl<'program, 'source> From<&'program $node<'source>> for FandangoNode<'program, 'source> {
            fn from(value: &'program $node<'source>) -> Self {
                Self::$node(value)
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
impl_node_from!(String, Cow, str);
impl_node_from!(Bytes, Cow, [u8]);

impl FandangoNode<'_, '_> {
    fn discriminant(self) -> usize {
        match self {
            FandangoNode::Program(_) => 0,
            FandangoNode::Statement(_) => 1,
            FandangoNode::Production(_) => 2,
            FandangoNode::Nonterminal(_) => 3,
            FandangoNode::Alternative(_) => 4,
            FandangoNode::Concatenation(_) => 5,
            FandangoNode::Operator(_) => 6,
            FandangoNode::Symbol(_) => 7,
            FandangoNode::String(_) => 8,
            FandangoNode::Bytes(_) => 9,
        }
    }

    fn ptr(self) -> usize {
        match self {
            FandangoNode::Program(s) => s as *const _ as usize,
            FandangoNode::Statement(s) => s as *const _ as usize,
            FandangoNode::Production(s) => s as *const _ as usize,
            FandangoNode::Nonterminal(s) => s as *const _ as usize,
            FandangoNode::Alternative(s) => s as *const _ as usize,
            FandangoNode::Concatenation(s) => s as *const _ as usize,
            FandangoNode::Operator(s) => s as *const _ as usize,
            FandangoNode::Symbol(s) => s as *const _ as usize,
            FandangoNode::String(s) => s as *const _ as usize,
            FandangoNode::Bytes(s) => s as *const _ as usize,
        }
    }
}

impl Eq for FandangoNode<'_, '_> {}

impl PartialEq<Self> for FandangoNode<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd<Self> for FandangoNode<'_, '_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FandangoNode<'_, '_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.discriminant()
            .cmp(&other.discriminant())
            .then_with(|| {
                if let (FandangoNode::Nonterminal(n1), FandangoNode::Nonterminal(n2)) =
                    (self, other)
                {
                    n1.cmp(n2)
                } else {
                    self.ptr().cmp(&other.ptr())
                }
            })
    }
}

impl Hash for FandangoNode<'_, '_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.discriminant().hash(state);
        if let FandangoNode::Nonterminal(nt) = self {
            nt.name().hash(state);
        } else {
            self.ptr().hash(state);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::graph::IntoGraph;
    use crate::lang::test::SIMPLE_GRAMMAR;
    use crate::lang::Program;
    use petgraph::dot::{Config, Dot};
    use std::error::Error;

    // this doesn't really test anything, just produces a graph in GraphViz format
    #[test]
    fn test_graph() -> Result<(), Box<dyn Error>> {
        let program = Program::try_from(SIMPLE_GRAMMAR)?;

        let graph = (&program).into_graph();

        let rendered = Dot::with_attr_getters(
            &graph,
            &[Config::NodeNoLabel, Config::EdgeNoLabel],
            &|_, (_, _, weight)| format!("label = {:?}", format!("{}", weight)),
            &|_, (_, node)| format!("label = {:?}", format!("{}", node)),
        );

        println!("{rendered}");

        Ok(())
    }
}
