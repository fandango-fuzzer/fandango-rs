//! Visitors for emitting FANDANGO-generated inputs.

use crate::graph::FandangoNode;
use crate::typing::Node;
use crate::visitor::{VisitResult, VisitableChildren, Visitor};
use std::convert::Infallible;
use std::io;
use std::ops::ControlFlow;

/// A visitor which emits the string representation of the tree, optionally using the existing
/// string representation in memory.
pub struct WriteVisitor<W, const CACHE: bool> {
    from: Vec<usize>,
    output: W,
}

/// A [`WriteVisitor`] which uses the existing strings.
pub type CachingWriteVisitor<W> = WriteVisitor<W, true>;
/// A [`WriteVisitor`] which doesn't use the existing strings.
pub type CachelessWriteVisitor<W> = WriteVisitor<W, false>;

impl<W, const CACHE: bool> From<(Vec<usize>, W)> for WriteVisitor<W, CACHE> {
    fn from((from, output): (Vec<usize>, W)) -> Self {
        Self::new_from(output, from)
    }
}

impl<W, const CACHE: bool> WriteVisitor<W, CACHE> {
    fn new(output: W) -> Self {
        Self::new_from(output, Vec::new())
    }

    fn new_from(output: W, from: Vec<usize>) -> Self {
        Self { output, from }
    }
}

impl<W> WriteVisitor<W, true> {
    /// Create a caching [`WriteVisitor`].
    pub fn caching(output: W) -> Self {
        Self::new(output)
    }

    /// Create a caching [`WriteVisitor`] starting at a specific point in the tree.
    pub fn caching_from(output: W, from: Vec<usize>) -> Self {
        Self::new_from(output, from)
    }
}
impl<W> WriteVisitor<W, false> {
    /// Create a non-caching [`WriteVisitor`].
    pub fn cacheless(output: W) -> Self {
        Self::new(output)
    }

    /// Create a non-caching [`WriteVisitor`] starting at a specific point in the tree.
    pub fn cacheless_from(output: W, from: Vec<usize>) -> Self {
        Self::new_from(output, from)
    }
}

impl<W, const CACHE: bool> WriteVisitor<W, CACHE> {
    /// Consume the visitor and collect the output.
    pub fn output(self) -> W {
        self.output
    }
}

impl<W, T, const CACHE: bool> Visitor<T> for WriteVisitor<W, CACHE>
where
    T: VisitableChildren<T>,
    W: io::Write,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = io::Error;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        if let Some(i) = self.from.pop() {
            assert_eq!(i, idx);
        }
        if CACHE && self.from.is_empty() {
            if let Some(span) = node.span() {
                self.output.write_all(span.as_str().as_bytes())?;
                return Ok(ControlFlow::Continue(self));
            }
        }
        match node.definition() {
            FandangoNode::String(s) => {
                self.output.write_all(s.as_bytes())?;
                Ok(ControlFlow::Continue(self))
            }
            _ => {
                if let Some(&from) = self.from.last() {
                    T::from(node).visit_each_from(self, from)
                } else {
                    T::from(node).visit_each(self)
                }
            }
        }
    }
}
