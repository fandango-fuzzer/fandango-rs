//! Type information used in generated FANDANGO grammars.

use crate::lang::FandangoNode;
use crate::lang::{Program, Tagged};
use alloc::boxed::Box;
use core::ops::Deref;

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

/// A type which has a corresponding [`FandangoNode`] definition. Statically typed (i.e. generated)
/// nodes may implement [`AsStaticNode`] instead.
pub trait AsNode {
    /// The root node for the grammar.
    fn root(&self) -> FandangoNode<'static, 'static>;
    /// The definition of this node.
    fn definition(&self) -> FandangoNode<'static, 'static>;
}

/// A type which has a corresponding [`FandangoNode`] definition. Statically typed (i.e. generated)
/// nodes may implement [`AsStaticNode`] instead.
pub trait AsStaticNode {
    /// The root node for the grammar.
    fn static_root() -> FandangoNode<'static, 'static>;
    /// The definition of this node.
    fn static_definition() -> FandangoNode<'static, 'static>;
}

impl<N> AsStaticNode for N
where
    N: Structured,
    FandangoNode<'static, 'static>: From<&'static Tagged<'static, N::FandangoType>>,
{
    fn static_root() -> FandangoNode<'static, 'static> {
        FandangoNode::Program(Self::ROOT.inner())
    }

    fn static_definition() -> FandangoNode<'static, 'static> {
        FandangoNode::from(Self::STRUCTURE)
    }
}

impl<N> AsNode for N
where
    N: AsStaticNode,
{
    fn root(&self) -> FandangoNode<'static, 'static> {
        Self::static_root()
    }

    fn definition(&self) -> FandangoNode<'static, 'static> {
        Self::static_definition()
    }
}

/// A discriminant for [`Node`]s which uniquely describes the type of this node. Not related to
/// [`std::mem::Discriminant`].
pub trait Discriminable {
    /// The discriminant, accessible by reference.
    fn discriminant(&self) -> usize;
}

/// A [`Discriminable`] node type for which the discriminant is known at compile time, as opposed to
/// e.g. a [`crate::dynamic::DynamicNode`].
pub trait StaticDiscriminable {
    /// The discriminant value.
    const DISCRIMINANT: usize;
}

/// A node representing an entry in a grammar or a derivation tree.
pub trait Node: Sized + AsNode + Discriminable + Clone {
    /// An enum which describes all possible nodes, and for which the following traits are
    /// implemented by generation (for `N::Type<'program>`):
    ///  - `From<&'program N>`
    ///  - `From<&'program Box<N>>`
    ///  - `From<&'program mut N>`
    ///  - `From<&'program mut Box<N>>`
    ///  - `From<N::TypeMut<'program>>`
    ///  - `DiscriminantLookup`
    ///  - `NodeLookup`
    type Type<'program>
    where
        Self: 'program;
    /// An enum which describes all possible mutable nodes, and for which the following traits are
    /// implemented (for `N::TypeMut<'program>`):
    ///  - `From<&'program mut N>`
    ///  - `From<&'program mut Box<N>>`
    ///  - [`crate::visitor::VisitWith`]
    ///  - [`crate::generation::InPlaceGenerated`]
    ///  - `DiscriminantLookup`
    ///  - `NodeLookup`
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

    /// Immutable accessors to children nodes.
    fn children(&self) -> Self::ChildrenRef<'_>;
    /// Mutable accessors to children nodes.
    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_>;
}

impl<T> Structured for Box<T>
where
    T: Structured,
{
    type FandangoType = T::FandangoType;
    const STRUCTURE: &'static Tagged<'static, Self::FandangoType> = T::STRUCTURE;
    const ROOT: &'static Tagged<'static, Program<'static>> = T::ROOT;
}

impl<T> StaticDiscriminable for Box<T>
where
    T: StaticDiscriminable,
{
    const DISCRIMINANT: usize = T::DISCRIMINANT;
}

impl<T> Discriminable for Box<T>
where
    T: Discriminable,
{
    fn discriminant(&self) -> usize {
        self.deref().discriminant()
    }
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

    fn children(&self) -> Self::ChildrenRef<'_> {
        (**self).children()
    }

    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {
        (**self).children_mut()
    }
}

/// Trait automatically implemented for opaque nodes which allows for downcasting of immutable
/// references to concrete nodes.
pub trait AsNodeRef<N> {
    /// Downcast this opaque node into a immutable concrete node reference, if this opaque node
    /// contains that node.
    fn as_node(&self) -> Option<&N>;
}

/// Trait automatically implemented for opaque nodes which allows for downcasting of mutable
/// references to concrete nodes.
pub trait AsNodeMut<N> {
    /// Downcast this opaque node into a mutable concrete node reference, if this opaque node
    /// contains that node.
    fn as_node_mut(&mut self) -> Option<&mut N>;
}

/// Trait for opaque types to lookup the discriminant associated with the provided [`FandangoNode`].
pub trait DiscriminantLookup {
    /// Get the discriminant!
    fn lookup_discriminant(node: &FandangoNode<'static, 'static>) -> usize;
}

/// Trait for opaque types to look up the [`FandangoNode`] for the provided discriminant.
///
/// This is only implementable for static implementations at this time. [`DynamicNode`] can
/// implement [`DiscriminantLookup`], but not [`NodeLookup`]. Use wisely.
pub trait NodeLookup {
    /// Get the node!
    fn lookup_node(discriminant: usize) -> FandangoNode<'static, 'static>;
}
