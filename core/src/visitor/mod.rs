//! Visitors for type trees emitted by FANDANGO's `#[derive]` implementation.

pub mod error;
pub mod mutator;
pub mod navigation;
pub mod write;

use crate::typing::Node;
use either::Either;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{ControlFlow, Deref};

type NodeTrace<T> = Vec<(T, usize, Option<((usize, usize), (usize, usize))>)>;

/// Visitor pattern over generated nodes.
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

pub trait VisitWith<V>: Sized
where
    V: Visitor<Self>,
{
    fn visit_with(self, visitor: V, idx: usize) -> VisitResult<V, Self>;
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

#[derive(Debug)]
pub enum ChainError<C> {
    UnexpectedContinue(C),
}

impl<C> Display for ChainError<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainError::UnexpectedContinue(_) => {
                f.write_str("The first visitor continued when we expected a break")
            }
        }
    }
}

impl<C> Error for ChainError<C> where C: Debug {}

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
            ::std::ops::ControlFlow::Continue(c) => ::std::result::Result::Err(::fandango::visitor::ChainError::UnexpectedContinue(c))?,
            ::std::ops::ControlFlow::Break(b) => b
        };
        visitor_chain!(starting from next, $node, $idx, $($visitors),+)
    }};

    ($node:expr, $idx:expr, $visitor:expr, $($visitors:expr),+) => {{
        let idx = $idx;
        let mut next = match visitor_chain!($node, idx, $visitor) {
            ::std::ops::ControlFlow::Continue(c) => ::std::result::Result::Err(::fandango::visitor::ChainError::UnexpectedContinue(c))?,
            ::std::ops::ControlFlow::Break(b) => b
        };
        visitor_chain!(@ next, $node, idx, $($visitors),+)
    }};
}
