pub mod navigation;
pub mod write;

use crate::typing::Node;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::ControlFlow;

type NodeTrace<T> = Vec<(T, usize, Option<((usize, usize), (usize, usize))>)>;

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

pub trait Visitor<T> {
    type Continue;
    type Break;
    type Error;

    fn visit<'program, N>(self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: VisitableChildren<'program, T> + Node + 'program,
        T: From<&'program mut N>;
}

pub type VisitResult<V, T>
where
    V: Visitor<T>,
= Result<ControlFlow<V::Break, V::Continue>, V::Error>;

pub type MaybeVisitResult<V, T>
where
    V: Visitor<T>,
= Result<Result<ControlFlow<V::Break, V::Continue>, V::Error>, V>;

pub trait VisitableChildren<'program, T> {
    fn visit_each<V>(&'program mut self, visitor: V) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>;

    fn visit_nth<V>(&'program mut self, visitor: V, idx: usize) -> MaybeVisitResult<V, T>
    where
        V: Visitor<T>;
}

impl<'program, N, T> VisitableChildren<'program, T> for Box<N>
where
    N: VisitableChildren<'program, T>,
{
    fn visit_each<V>(&'program mut self, visitor: V) -> VisitResult<V, T>
    where
        V: Visitor<T, Continue = V>,
    {
        (**self).visit_each(visitor)
    }

    fn visit_nth<V>(&'program mut self, visitor: V, idx: usize) -> MaybeVisitResult<V, T>
    where
        V: Visitor<T>,
    {
        (**self).visit_nth(visitor, idx)
    }
}
