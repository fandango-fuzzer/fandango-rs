//! Visitors for type trees emitted by FANDANGO's `#[derive]` implementation.

pub mod assignment;
pub mod error;
pub mod navigation;
pub mod write;

use crate::typing::Node;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use core::ops::ControlFlow;
use either::Either;

type NodeTrace<T> = Vec<(T, usize, Option<((usize, usize), (usize, usize))>)>;

/// Visitor pattern over nodes.
pub trait Visitor<T> {
    /// The type which is returned when the visitation may continue.
    type Continue;
    /// The type which is returned when the visitation is complete.
    type Break;
    /// The error which is returned upon failure.
    type Error;

    /// Visit a provided node. You are responsible for recursion.
    fn visit<'program, N>(self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>;
}

/// Visits an opaque node with the provided visitor.
pub trait VisitWith<'a, V>: Sized {
    /// This type is an intermediary which represents what type *actually* gets visited by the
    /// visitor. This is necessary because implementors of [`VisitWith`] look something like this:
    ///
    /// ```
    /// # #![allow(non_camel_case_types)]
    /// # struct start<'source>(std::marker::PhantomData<&'source ()>);
    /// #
    /// pub enum TypeMut<'program, 'source> {
    ///     start(&'program mut start<'source>),
    ///     // other variants...
    /// }
    /// ```
    ///
    /// If we use the visitor directly, this will consume the `'program` lifetime -- and thus
    /// prevent further modification until we create a new node of this type. By specifying what is
    /// indeed visited, we can first reborrow the type and then perform the visit:
    ///
    /// ```
    /// # #![allow(non_camel_case_types)]
    /// # use std::ops::ControlFlow;
    /// # use std::marker::PhantomData;
    /// # use std::convert::Infallible;
    /// # use fandango_core::lang::FandangoNode;
    /// # use fandango_core::typing::{AsNode, Discriminable, Node};
    /// # use fandango_core::visitor::{MaybeVisitResult, VisitResult, VisitWith, VisitableChildren, Visitor};
    /// #
    /// # pub struct start<'source>(PhantomData<&'source ()>);
    /// # impl Discriminable for start<'_> {
    /// #     const DISCRIMINANT: usize = 0;
    /// # }
    /// #
    /// # impl AsNode for start<'_> {
    /// #     fn root() -> FandangoNode<'static, 'static> {
    /// #         unimplemented!()
    /// #     }
    /// #
    /// #     fn definition() -> FandangoNode<'static, 'static> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// # impl<'source> Node for start<'source> {
    /// #     type Type<'program> = () where Self: 'program;
    /// #     type TypeMut<'program> = TypeMut<'program, 'source> where Self: 'program;
    /// #     type ChildrenRef<'program> = () where Self: 'program;
    /// #     type ChildrenRefMut<'program> = () where Self: 'program;
    /// #
    /// #     fn span(&self) -> Option<pest::Span<'_>> {
    /// #         unimplemented!()
    /// #     }
    /// #
    /// #     fn clear_span(&mut self) {
    /// #         unimplemented!()
    /// #     }
    /// #
    /// #     fn children(&self) -> Self::ChildrenRef<'_> {
    /// #         unimplemented!()
    /// #     }
    /// #
    /// #     fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// pub enum TypeMut<'program, 'source> {
    ///     start(&'program mut start<'source>),
    ///     // other children...
    /// }
    ///
    /// impl<'program, 'source> TypeMut<'program, 'source> {
    ///     fn reborrow<'a>(&'a mut self) -> TypeMut<'a, 'source> where 'source: 'a {
    ///         match self {
    ///             TypeMut::start(n) => TypeMut::start(&mut *n),
    ///             // other children...
    ///         }
    ///     }
    /// }
    /// #
    /// # impl<'program, 'source> VisitableChildren<TypeMut<'program, 'source>> for TypeMut<'program, 'source> {
    /// #     fn visit_each<V>(self, visitor: V) -> VisitResult<V, TypeMut<'program, 'source>> where V: Visitor<TypeMut<'program, 'source>, Continue=V> {
    /// #         Ok(ControlFlow::Continue(visitor))
    /// #     }
    /// #
    /// #     fn visit_each_reverse<V>(self, visitor: V) -> VisitResult<V, TypeMut<'program, 'source>> where V: Visitor<TypeMut<'program, 'source>, Continue=V> {
    /// #         unimplemented!()
    /// #     }
    /// #
    /// #     fn visit_each_from<V>(self, visitor: V, idx: usize) -> VisitResult<V, TypeMut<'program, 'source>> where V: Visitor<TypeMut<'program, 'source>, Continue=V> {
    /// #         unimplemented!()
    /// #     }
    /// #
    /// #     fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> VisitResult<V, TypeMut<'program, 'source>> where V: Visitor<TypeMut<'program, 'source>, Continue=V> {
    /// #         unimplemented!()
    /// #     }
    /// #
    /// #     fn visit_nth<V>(self, visitor: V, idx: usize) -> MaybeVisitResult<V, TypeMut<'program, 'source>> where V: Visitor<TypeMut<'program, 'source>> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// # impl<'program, 'source> From<&'program mut start<'source>> for TypeMut<'program, 'source> {
    /// #     fn from(value: &'program mut start<'source>) -> Self {
    /// #         Self::start(value)
    /// #     }
    /// # }
    /// #
    /// impl<'a, 'program, 'source, V> VisitWith<'a, V> for TypeMut<'program, 'source>
    /// where
    ///     'program: 'a,
    ///     'source: 'program,
    /// {
    ///     type Visited = TypeMut<'a, 'source>;
    ///
    ///     fn visit_with(&'a mut self, visitor: V, idx: usize) -> VisitResult<V, Self::Visited>
    ///     where
    ///         V: Visitor<TypeMut<'a, 'source>> {
    ///         match self.reborrow() {
    ///             TypeMut::start(n) => visitor.visit(n, idx),
    ///             // other children...
    ///         }
    ///     }
    /// }
    ///
    /// struct ToyVisitor;
    ///
    /// impl<T> Visitor<T> for ToyVisitor
    /// where
    ///     T: VisitableChildren<T>,
    /// {
    ///     type Continue = Self;
    ///     type Break = Infallible;
    ///     type Error = Infallible;
    ///
    ///     fn visit<'program, N>(self, node: &'program mut N, _: usize) -> VisitResult<Self, T>
    ///     where
    ///         N: Node<TypeMut<'program> = T>,
    ///         T: From<&'program mut N>
    ///     {
    ///         T::from(node).visit_each(self)
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // using later...
    /// # let node = start(PhantomData);
    /// let mut node: start<'static> = node; // node from generated source
    /// let mut t = TypeMut::from(&mut node);
    /// // we can now perform the visitation multiple times
    /// t.visit_with(ToyVisitor, 0)?;
    /// t.visit_with(ToyVisitor, 0)?;
    /// t.visit_with(ToyVisitor, 0)?;
    /// t.visit_with(ToyVisitor, 0)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// This thereby informs the compiler that the visitor will not consume the node's reference.
    type Visited;

    /// Perform the visit on the opaque node.
    fn visit_with(&'a mut self, visitor: V, idx: usize) -> VisitResult<V, Self::Visited>
    where
        V: Visitor<Self::Visited>;
}

/// The result type returned by visitors.
pub type VisitResult<V, T>
where
    V: Visitor<T>,
= Result<ControlFlow<V::Break, V::Continue>, V::Error>;

/// The result type returned when visiting a node which may not exist.
pub type MaybeVisitResult<V, T>
where
    V: Visitor<T>,
= Result<Result<ControlFlow<V::Break, V::Continue>, V::Error>, V>;

/// Denotes that a node (and its children) may be visited by a given visitor.
pub trait VisitableChildren<T> {
    /// Visit each of the children of this node, in order.
    fn visit_each<V>(self, visitor: V) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>;

    /// Visit each of the children of this node, in reverse order.
    fn visit_each_reverse<V>(self, visitor: V) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>;

    /// Visit each of the children of this node, in order, starting at a given node (inclusive).
    fn visit_each_from<V>(self, visitor: V, idx: usize) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>;

    /// Visit each of the children of this node, in reverse order, starting at a given node (inclusive).
    fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>;

    /// Visit just the nth child, or return the visitor if the nth child did not exist.
    fn visit_nth<V>(self, visitor: V, idx: usize) -> MaybeVisitResult<V, T>
    where
        V: Visitor<T>;
}

impl<'program, N, T> VisitableChildren<T> for &'program mut Box<N>
where
    &'program mut N: VisitableChildren<T>,
{
    fn visit_each<V>(self, visitor: V) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>,
    {
        (**self).visit_each(visitor)
    }

    fn visit_each_reverse<V>(self, visitor: V) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>,
    {
        (**self).visit_each_reverse(visitor)
    }

    fn visit_each_from<V>(self, visitor: V, idx: usize) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>,
    {
        (**self).visit_each_from(visitor, idx)
    }

    fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>,
    {
        (**self).visit_each_reverse_from(visitor, idx)
    }

    fn visit_nth<V>(self, visitor: V, idx: usize) -> MaybeVisitResult<V, T>
    where
        V: Visitor<T>,
    {
        (**self).visit_nth(visitor, idx)
    }
}

impl<V1, V2, T> Visitor<T> for Either<V1, V2>
where
    V1: Visitor<T>,
    V2: Visitor<T>,
{
    type Continue = Either<V1::Continue, V2::Continue>;
    type Break = Either<V1::Break, V2::Break>;
    type Error = Either<V1::Error, V2::Error>;

    fn visit<'program, N>(self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        Ok(match self {
            Either::Left(visitor) => match visitor.visit(node, idx).map_err(Either::Left)? {
                ControlFlow::Continue(c) => ControlFlow::Continue(Either::Left(c)),
                ControlFlow::Break(c) => ControlFlow::Break(Either::Left(c)),
            },
            Either::Right(visitor) => match visitor.visit(node, idx).map_err(Either::Right)? {
                ControlFlow::Continue(c) => ControlFlow::Continue(Either::Right(c)),
                ControlFlow::Break(c) => ControlFlow::Break(Either::Right(c)),
            },
        })
    }
}

/// Denotes that an error was encountered during the chaining of visitors with [`crate::visitor_chain`].
#[derive(Debug)]
pub enum ChainError<C> {
    /// While chaining, [`ControlFlow::Continue`] was encountered instead of [`ControlFlow::Break`].
    UnexpectedContinue(C),
}

impl<C> Display for ChainError<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ChainError::UnexpectedContinue(_) => {
                f.write_str("The first visitor continued when we expected a break")
            }
        }
    }
}

impl<C> Error for ChainError<C> where C: Debug {}

/// Perform chaining of visitors. Useful for when your visitors perform their own traversal, and may
/// need exclusive mutable access to parent nodes of the target. If you just need mutable access to
/// a particular subtree, use [`navigation::GoTo::go_to`] or [`navigation::GoToWith::go_to`].
///
/// The following invariants must hold:
///  - For all but the last visitor, the `Break` type must be [`VecDeque`]`<usize>`.
///    - The `Break` type represents the full path taken to the intended target node, *including the
///      index of the first node*.
///    - The first node's index *MUST* be exactly the index provided originally.
///  - All visitors but the first must implement [`navigation::StartingFrom`].
///    - The visitor accepts the path *not including the index of the first node* (this is instead
///      provided as an argument to [`Visitor::visit`]).
#[macro_export]
macro_rules! visitor_chain {
    ($node:expr, $idx:ident, $visitor:expr) => {{
        ::fandango::visitor::Visitor::visit($visitor, $node, $idx)?
    }};

    (@ $next:expr, $node:expr, $idx:ident, $visitor:expr) => {{
        assert_eq!($idx, $next.pop_front().unwrap());
        ::fandango::visitor::Visitor::visit(::fandango::visitor::navigation::StartingFrom::starting_from($visitor, $next), $node, $idx)?
    }};

    (@ $next:expr, $node:expr, $idx:ident, $visitor:expr, $($visitors:expr),+) => {{
        assert_eq!($idx, $next.pop_front().unwrap());
        let mut next = match visitor_chain!(@ $next, $node, $idx, $visitor) {
            ::core::ops::ControlFlow::Continue(c) => ::core::result::Result::Err(::fandango::visitor::ChainError::UnexpectedContinue(c))?,
            ::core::ops::ControlFlow::Break(b) => b
        };
        visitor_chain!(starting from next, $node, $idx, $($visitors),+)
    }};

    ($node:expr, $idx:expr, $visitor:expr, $($visitors:expr),+) => {{
        let idx = $idx;
        let mut next = match visitor_chain!($node, idx, $visitor) {
            ::core::ops::ControlFlow::Continue(c) => ::core::result::Result::Err(::fandango::visitor::ChainError::UnexpectedContinue(c))?,
            ::core::ops::ControlFlow::Break(b) => b
        };
        visitor_chain!(@ next, $node, idx, $($visitors),+)
    }};
}
