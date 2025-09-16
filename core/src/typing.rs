//! Type information used in generated FANDANGO grammars.

use crate::lang::FandangoNode;
use crate::lang::{Program, Tagged};
use crate::visitor::{VisitableChildren, VisitableChildrenMut};
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
    /// An enum which describes all possible nodes, and which may be visited with [`VisitWith`].
    type Type<'program>: From<&'program Self>
        + DiscriminantLookup
        + VisitableChildren<Self::Type<'program>>
        + PartialEq
        + PartialEq<Self::TypeMut<'program>>
        + Eq
        + AsNodeRef<Self::Repr>
        + Discriminable
    where
        Self: 'program;
    /// An enum which describes all possible mutable nodes, and which may be visited with either
    /// [`VisitWith`] or [`VisitWithMut`].
    type TypeMut<'program>: From<&'program mut Self>
        + AssignFrom<Self::Type<'program>>
        + DiscriminantLookup
        + VisitableChildren<Self::Type<'program>>
        + VisitableChildrenMut<Self::TypeMut<'program>>
        + PartialEq
        + PartialEq<Self::Type<'program>>
        + Eq
        + AsNodeMut<Self::Repr>
        + Discriminable
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

    /// The actual representative of this node
    ///
    /// Used to disambiguate over indirected nodes (e.g., via [`Box`]) so that we can constrain
    /// [`Node::Type`] and [`Node::TypeMut`] as strictly as possible. For generated nodes, [`Repr`]
    /// is `Self`.
    type Repr;

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
    for<'a> <T as Node>::Type<'a>: From<&'a Box<T>> + AsNodeRef<T>,
    for<'a> <T as Node>::TypeMut<'a>: From<&'a mut Box<T>> + AsNodeMut<T>,
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

    type Repr = T; // the underlying node

    fn children(&self) -> Self::ChildrenRef<'_> {
        (**self).children()
    }

    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {
        (**self).children_mut()
    }
}

/// Trait which simplifies copying between [`Node::Type`] and [`Node::TypeMut`].
pub trait AssignFrom<T> {
    /// Assigns from the other value, returning true if successful.
    fn assign_from(&mut self, other: T) -> bool;
}

/// Helper trait to access the opaque form of a given node.
pub trait Opaque {
    /// The opaque node (i.e., [`Node::Type`]).
    type Returned;

    /// Get the opaque version of this node.
    fn opaque(self) -> Self::Returned;
}

impl<'a, N> Opaque for &'a N
where
    N: Node,
    <N as Node>::Type<'a>: From<&'a N>,
{
    type Returned = <N as Node>::Type<'a>;

    fn opaque(self) -> Self::Returned {
        self.into()
    }
}

/// Helper trait to access the mutable opaque form of a given node.
pub trait OpaqueMut {
    /// The mutable opaque node (i.e. [`Node::TypeMut`])
    type Returned;

    /// Get the mutable opaque version of this node.
    fn opaque_mut(self) -> Self::Returned;
}

impl<'a, N> OpaqueMut for &'a mut N
where
    N: Node,
    <N as Node>::TypeMut<'a>: From<&'a mut N>,
{
    type Returned = <N as Node>::TypeMut<'a>;

    fn opaque_mut(self) -> Self::Returned {
        self.into()
    }
}

/// Trait automatically implemented for opaque nodes which allows for downcasting of immutable
/// references to concrete nodes.
///
/// Prefer [`Downcast`].
pub trait AsNodeRef<N> {
    /// Downcast this opaque node into a immutable concrete node reference, if this opaque node
    /// contains that node.
    fn as_node(&self) -> Option<&N>;
}

/// Trait automatically implemented for opaque nodes which allows for downcasting of mutable
/// references to concrete nodes.
///
/// Prefer [`DowncastMut`].
pub trait AsNodeMut<N>: AsNodeRef<N> {
    /// Downcast this opaque node into a mutable concrete node reference, if this opaque node
    /// contains that node.
    fn as_node_mut(&mut self) -> Option<&mut N>;
}

/// Downcast this opaque node into a concrete node, as possible.
pub trait Downcast {
    /// Perform the downcast.
    fn downcast<N>(&self) -> Option<&N>
    where
        Self: AsNodeRef<N>;
}

impl<T> Downcast for T {
    fn downcast<N>(&self) -> Option<&N>
    where
        Self: AsNodeRef<N>,
    {
        self.as_node()
    }
}

/// Downcast this opaque node into a mutable concrete node, as possible.
pub trait DowncastMut {
    /// Perform the downcast.
    fn downcast_mut<N>(&mut self) -> Option<&mut N>
    where
        Self: AsNodeMut<N>;
}

impl<T> DowncastMut for T {
    fn downcast_mut<N>(&mut self) -> Option<&mut N>
    where
        Self: AsNodeMut<N>,
    {
        self.as_node_mut()
    }
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

/// Accessor trait for children/variants of nodes.
///
/// You don't want to use this directly. Use [`Nth`] instead.
pub trait ChildAccessor<const N: usize>: Node {
    /// The (immutable) child reference to the [`N`]th child.
    type Child<'a>
    where
        Self: 'a;
    /// The (mutable) child reference to the [`N`]th child.
    type ChildMut<'a>
    where
        Self: 'a;

    /// Access the child node at position [`N`] immutably.
    fn child(&self) -> Self::Child<'_>;
    /// Access the child node at position [`N`] mutably.
    fn child_mut(&mut self) -> Self::ChildMut<'_>;
}

/// Accessor for valid [`N`]th child (or, for alternations, variant).
///
/// This trait is defined automatically for all nodes with children.
pub trait Nth {
    /// Access the [`N`]th child immutably -- if available.
    fn nth<const N: usize>(&self) -> Self::Child<'_>
    where
        Self: ChildAccessor<N>;

    /// Access the [`N`]th child mutably -- if available.
    fn nth_mut<const N: usize>(&mut self) -> Self::ChildMut<'_>
    where
        Self: ChildAccessor<N>;
}

impl<T> Nth for T
where
    T: Node,
{
    fn nth<const N: usize>(&self) -> <Self as ChildAccessor<N>>::Child<'_>
    where
        Self: ChildAccessor<N>,
    {
        self.child()
    }

    fn nth_mut<const N: usize>(&mut self) -> <Self as ChildAccessor<N>>::ChildMut<'_>
    where
        Self: ChildAccessor<N>,
    {
        self.child_mut()
    }
}
