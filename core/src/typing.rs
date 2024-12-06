//! Type information used in generated FANDANGO grammars.

use crate::graph::FandangoNode;
use crate::lang::Tagged;
use pest::Span;
use std::borrow::Cow;
use std::rc::Rc;

/// Convert a maybe owned string span into a span. Only for use with generated code.
pub fn maybe_owned_span<'node, 'source>(
    source: &'node Option<(Rc<Cow<'source, str>>, usize, usize)>,
) -> Option<Span<'node>> {
    source
        .as_ref()
        .and_then(|(source, start, end)| Span::new(source, *start, *end))
}

pub trait Structured
where
    FandangoNode<'static, 'static>: From<&'static Self::FandangoType>,
{
    type FandangoType: 'static;
    const STRUCTURE: &'static Tagged<'static, Self::FandangoType>;

    fn as_node(&self) -> FandangoNode<'static, 'static> {
        FandangoNode::from(Self::STRUCTURE.inner())
    }
}

/// A node representing an entry in a grammar or a derivation tree.
pub trait Node: Sized {
    /// The span, or [`None`] if the node was generated or mutated without concretisation.
    fn span(&self) -> Option<Span<'_>>;
}

/// Denotes that the provided node has direct children. Alternates must first be unwrapped to their
/// concrete variants.
pub trait Children: Node {
    /// The type which references each child individually.
    type ChildrenRef<'program>
    where
        Self: 'program;
    /// The type which mutably references each child individually.
    type ChildrenRefMut<'program>
    where
        Self: 'program;

    /// Immutable accessors to children nodes.
    fn children(&self) -> Self::ChildrenRef<'_>;
    /// Mutable accessors to children nodes.
    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_>;
}
