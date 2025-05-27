//! Utility visitors for manipulating the value of nodes directly.

use crate::typing::{AsNodeMut, Node};
use crate::visitor::{VisitResult, Visitor};
use core::convert::Infallible;
use core::ops::ControlFlow;

/// Simple visitor which just replaces a node with the contained value. Returns itself if the type
/// of the node is incorrect.
#[derive(Debug, Copy, Clone)]
pub struct AssignmentVisitor<N>(pub N);

impl<U, T> Visitor<T> for AssignmentVisitor<U>
where
    T: AsNodeMut<U>,
{
    type Continue = Infallible;
    type Break = ();
    type Error = Self;

    fn visit<'program, N>(self, node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        match T::from(node).as_node_mut() {
            None => Err(self),
            Some(node) => {
                *node = self.0;
                Ok(ControlFlow::Break(()))
            }
        }
    }
}
