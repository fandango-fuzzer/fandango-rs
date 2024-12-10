//! Type information used in generated FANDANGO grammars.

use crate::graph::FandangoNode;
use crate::lang::Tagged;
use crate::visitor::VisitableChildren;
use pest::Span;
use std::borrow::Cow;
use std::ops::Deref;
use std::rc::Rc;

/// Convert a maybe owned string span into a span. Only for use with generated code.
pub fn maybe_owned_span<'program>(
    source: &'program Option<(Rc<Cow<'_, str>>, usize, usize)>,
) -> Option<Span<'program>> {
    source
        .as_ref()
        .and_then(|(source, start, end)| Span::new(source, *start, *end))
}

pub trait Structured {
    type FandangoType: 'static;
    const STRUCTURE: &'static Tagged<'static, Self::FandangoType>;
}

pub trait AsNode {
    fn definition(&self) -> FandangoNode<'static, 'static>;
}

impl<N> AsNode for N
where
    N: Structured,
    FandangoNode<'static, 'static>: From<&'static Tagged<'static, N::FandangoType>>,
{
    fn definition(&self) -> FandangoNode<'static, 'static> {
        FandangoNode::from(Self::STRUCTURE)
    }
}

/// A node representing an entry in a grammar or a derivation tree.
pub trait Node: Sized + AsNode {
    type Type<'program>
    where
        Self: 'program;
    type TypeMut<'program>
    where
        Self: 'program;
    /// The type which references each child individually.
    type ChildrenRef<'program>
    where
        Self: 'program;
    /// The type which mutably references each child individually.
    type ChildrenRefMut<'program>
    where
        Self: 'program;

    /// The span, or [`None`] if the node was generated or mutated without concretisation.
    fn span(&self) -> Option<Span<'_>>;

    /// Immutable accessors to children nodes.
    fn children(&self) -> Self::ChildrenRef<'_>;
    /// Mutable accessors to children nodes.
    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_>;
}

impl<T> Structured for Box<T>
where
    T: Node + Structured,
{
    type FandangoType = T::FandangoType;
    const STRUCTURE: &'static Tagged<'static, Self::FandangoType> = T::STRUCTURE;
}

impl<T> Node for Box<T>
where
    Box<T>: AsNode,
    T: Node + Structured,
{
    type Type<'program>
        = T::Type<'program>
    where
        T: 'program;
    type TypeMut<'program>
        = T::TypeMut<'program>
    where
        T: 'program;
    type ChildrenRef<'program>
        = T::ChildrenRef<'program>
    where
        T: 'program;
    type ChildrenRefMut<'program>
        = T::ChildrenRefMut<'program>
    where
        T: 'program;

    fn span(&self) -> Option<Span<'_>> {
        self.deref().span()
    }

    fn children(&self) -> Self::ChildrenRef<'_> {
        (**self).children()
    }

    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {
        (**self).children_mut()
    }
}
