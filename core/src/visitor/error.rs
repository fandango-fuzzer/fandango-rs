//! Visitor error flavours. Currently unused.

#![allow(missing_docs)]

use crate::typing::Node;
use crate::visitor::NodeTrace;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};

#[derive(Debug)]
pub struct VisitErrorTrace<E, T> {
    inner: E,
    trace: NodeTrace<T>,
}

impl<E, T> VisitErrorTrace<E, T> {
    pub fn extend<'node, N: Node<Type<'node> = T>>(mut self, node: &'node N, idx: usize) -> Self
    where
        N::Type<'node>: From<&'node N>,
    {
        self.trace.push((
            T::from(node),
            idx,
            node.span()
                .map(|s| (s.start_pos().line_col(), s.end_pos().line_col())),
        ));
        self
    }
}

impl<E, T> Display for VisitErrorTrace<E, T>
where
    E: Error,
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}, backtrace:\n", self.inner))?;
        let largest_idx = self
            .trace
            .iter()
            .map(|(_, idx, _)| idx)
            .max()
            .expect("Cannot construct error traces without a single entry");
        let max_padding = largest_idx.checked_ilog10().unwrap_or(0) as usize + 1;
        for (i, (node, idx, span)) in self.trace.iter().enumerate() {
            f.write_fmt(format_args!("  {}[{idx: >max_padding$}]: {node}", i + 1))?;
            if let Some(((l1, c1), (l2, c2))) = span {
                f.write_fmt(format_args!(" ({l1}:{c1}-{l2}:{c2})"))?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl<E, T> Error for VisitErrorTrace<E, T>
where
    E: Error,
    T: Debug + Display,
{
    fn cause(&self) -> Option<&dyn Error> {
        Some(&self.inner)
    }
}

impl<E, T> From<E> for VisitErrorTrace<E, T> {
    fn from(inner: E) -> Self {
        Self {
            inner,
            trace: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct InvalidPath;

impl Display for InvalidPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid path provided while traversing")
    }
}

impl Error for InvalidPath {}
