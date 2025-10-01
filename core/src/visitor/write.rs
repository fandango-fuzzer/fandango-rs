//! Visitors for emitting FANDANGO-generated inputs.

use crate::lang::FandangoNode;
use crate::typing::{AsNodeRef, Node, Opaque};
use crate::visitor::{VisitResult, VisitableChildren, Visitor};
use core::convert::Infallible;
use core::ops::ControlFlow;
use embedded_io as io;

/// A visitor which emits the string representation of the tree.
pub struct WriteVisitor<W> {
    output: W,
}

impl<W> WriteVisitor<W> {
    /// Create a [`WriteVisitor`].
    pub fn new(output: W) -> Self {
        Self { output }
    }
}

impl<W> WriteVisitor<W> {
    /// Consume the visitor and collect the output.
    pub fn output(self) -> W {
        self.output
    }
}

impl<W, T> Visitor<T> for WriteVisitor<W>
where
    T: VisitableChildren<T>,
    W: io::Write,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = <W as io::ErrorType>::Error;

    fn visit<'program, N>(mut self, node: &'program N, _: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        match node.definition() {
            FandangoNode::String(s) => {
                self.output.write_all(s.inner())?;
                Ok(ControlFlow::Continue(self))
            }
            _ => node.opaque().visit_each(self),
        }
    }
}
