//! Type information used in generated FANDANGO grammars.

use crate::graph::FandangoNode;
use crate::lang::{Program, Tagged};
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

/// Denotes that this type is structured in a tree shape with an associated [`FandangoNode`]. Only
/// to be implemented by generated code.
pub trait Structured {
    /// The type of [`FandangoNode`] which this node is structured for.
    type FandangoType: 'static;
    /// The concrete [`FandangoNode`] from the grammar definition, including source tagging.
    const STRUCTURE: &'static Tagged<'static, Self::FandangoType>;
    /// The root node of this structure.
    const ROOT: &'static Tagged<'static, Program<'static>>;
}

/// A type which has a corresponding [`FandangoNode`] definition.
pub trait AsNode {
    /// The root node for the grammar.
    fn root() -> FandangoNode<'static, 'static>;
    /// The definition of this node.
    fn definition() -> FandangoNode<'static, 'static>;
}

impl<N> AsNode for N
where
    N: Structured,
    FandangoNode<'static, 'static>: From<&'static Tagged<'static, N::FandangoType>>,
{
    fn root() -> FandangoNode<'static, 'static> {
        FandangoNode::Program(Self::ROOT.inner())
    }

    fn definition() -> FandangoNode<'static, 'static> {
        FandangoNode::from(Self::STRUCTURE)
    }
}

/// A discriminant for [`Node`]s which uniquely describes the type of this node. Not related to
/// [`std::mem::Discriminant`].
pub trait Discriminable {
    /// The discriminant value.
    const DISCRIMINANT: usize;

    /// The discriminant, accessible by reference.
    fn discriminant(&self) -> usize {
        Self::DISCRIMINANT
    }
}

/// A node representing an entry in a grammar or a derivation tree.
pub trait Node: Sized + AsNode + Discriminable {
    /// An enum which describes all possible nodes, and for which the following traits are
    /// implemented by generation (for `N::Type<'program>`):
    ///
    ///  - `From<&'program N>`
    ///  - `From<&'program Box<N>>`
    ///  - `From<&'program mut N>`
    ///  - `From<&'program mut Box<N>>`
    ///  - `From<N::TypeMut<'program>>`
    type Type<'program>
    where
        Self: 'program;
    /// An enum which describes all possible mutable nodes, and for which the following traits are
    /// implemented (for `N::TypeMut<'program>`):
    ///
    ///  - `From<&'program mut N>`
    ///  - `From<&'program mut Box<N>>`
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
    const ROOT: &'static Tagged<'static, Program<'static>> = T::ROOT;
}

impl<T> Discriminable for Box<T>
where
    T: Discriminable,
{
    const DISCRIMINANT: usize = T::DISCRIMINANT;
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
