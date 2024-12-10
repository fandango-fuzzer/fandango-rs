use crate::graph::FandangoNode;
use crate::typing::{AsNode, Node};
use crate::visitor::{VisitResult, VisitableChildren, Visitor};
use std::convert::Infallible;
use std::io;
use std::ops::ControlFlow;

pub struct WriteVisitor<W, const CACHE: bool> {
    output: W,
}

impl<W> WriteVisitor<W, true> {
    pub fn new(output: W) -> Self {
        Self { output }
    }

    pub fn cacheless(output: W) -> WriteVisitor<W, false> {
        WriteVisitor::<W, false> { output }
    }
}

impl<W, const CACHE: bool> WriteVisitor<W, CACHE> {
    pub fn output(self) -> W {
        self.output
    }
}

impl<W, T, const CACHE: bool> Visitor<T> for WriteVisitor<W, CACHE>
where
    W: io::Write,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = io::Error;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: VisitableChildren<'program, T> + Node + 'program,
        T: From<&'program mut N>,
    {
        if CACHE {
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
            _ => node.visit_each(self),
        }
    }
}
