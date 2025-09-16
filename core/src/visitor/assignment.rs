//! Utility visitors for manipulating the value of nodes directly.

use crate::typing::{AsNodeMut, Discriminable, Node};
use crate::visitor::{VisitMutResult, VisitorMut};
use core::convert::Infallible;
use core::ops::ControlFlow;

/// Simple visitor which just replaces a node with the contained value. Returns itself if the type
/// of the node is incorrect.
#[derive(Debug, Copy, Clone)]
pub struct AssignmentVisitor<N>(pub N);

impl<U, T> VisitorMut<T> for AssignmentVisitor<U>
where
    U: Discriminable,
    T: AsNodeMut<U>,
{
    type Continue = Infallible;
    type Break = ();
    type Error = Self;

    fn visit_mut<'program, N>(self, node: &'program mut N, _idx: usize) -> VisitMutResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        let mut visited = T::from(node);
        match <T as AsNodeMut<U>>::as_node_mut(&mut visited) {
            Some(inner) if inner.discriminant() == self.0.discriminant() => {
                *inner = self.0;
                Ok(ControlFlow::Break(()))
            }
            _ => Err(self),
        }
    }
}

/// Swaps a visited node with this opaque node.
pub struct SwapVisitor<T> {
    replacement: T,
}

impl<T> SwapVisitor<T> {
    /// Swap this node with that node!
    pub fn new(replacement: T) -> Self {
        Self { replacement }
    }
}

impl<T> VisitorMut<T> for SwapVisitor<T> {
    type Continue = Infallible;
    type Break = T;
    type Error = T;

    fn visit_mut<'program, N>(
        mut self,
        node: &'program mut N,
        _idx: usize,
    ) -> VisitMutResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        match self.replacement.as_node_mut() {
            Some(replacement) if replacement.discriminant() == node.discriminant() => {
                core::mem::swap(node, replacement);
                Ok(ControlFlow::Break(self.replacement))
            }
            _ => Err(self.replacement),
        }
    }
}
