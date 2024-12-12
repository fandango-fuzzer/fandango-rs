//! Utility visitors for navigating type trees.

use crate::typing::Node;
use crate::visitor::{VisitResult, VisitableChildren, Visitor};
use either::Either;
use std::collections::VecDeque;
use std::convert::Infallible;
use std::ops::ControlFlow;

/// Find a given node, by DFS or BFS.
#[derive(Clone, Debug)]
pub struct FindVisitor<const DFS: bool> {
    reference: Either<(usize, usize), usize>,
    discriminant: usize,
    path: Vec<usize>,
    from: Vec<usize>,
}

impl<const DFS: bool> FindVisitor<DFS> {
    fn new<N>(target: &N) -> Self
    where
        N: Node,
    {
        Self::new_from(target, Vec::new())
    }

    fn new_from<N>(target: &N, from: Vec<usize>) -> Self
    where
        N: Node,
    {
        Self {
            reference: match target.span() {
                None => Either::Right(target as *const N as usize),
                Some(s) => Either::Left((s.start(), s.end())),
            },
            discriminant: N::DISCRIMINANT,
            path: Vec::new(),
            from,
        }
    }
}

impl FindVisitor<true> {
    /// Search for a node by DFS.
    pub fn dfs<N>(target: &N) -> Self
    where
        N: Node,
    {
        Self::new(target)
    }

    /// Search for a node by DFS, starting from a given position.
    pub fn dfs_from<N>(target: &N, from: Vec<usize>) -> Self
    where
        N: Node,
    {
        Self::new_from(target, from)
    }
}

impl FindVisitor<false> {
    /// Search for a node by BFS. A `_from` variant is not provided at this time.
    pub fn bfs<N>(target: &N) -> Self
    where
        N: Node,
    {
        Self::new(target)
    }
}

impl<T> Visitor<T> for FindVisitor<true>
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Vec<usize>;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        if let Some(i) = self.from.pop() {
            assert_eq!(i, idx);
        }
        let span = node.span().map(|s| (s.start(), s.end()));
        let actual_ptr = node as *const N as usize;
        if N::DISCRIMINANT == self.discriminant {
            match (&self.reference, span) {
                (&Either::Left((s1, e1)), Some((s2, e2))) if s1 == s2 && e1 == e2 => {
                    self.path.push(idx);
                    return Ok(ControlFlow::Break(self.path));
                }
                (&Either::Right(ptr), _) if ptr == actual_ptr => {
                    self.path.push(idx);
                    return Ok(ControlFlow::Break(self.path));
                }
                _ => {}
            }
        }
        match {
            if let Some(&from) = self.from.last() {
                T::from(node).visit_each_from(self, from)
            } else {
                T::from(node).visit_each(self)
            }
        }? {
            ControlFlow::Break(mut path) => {
                path.push(idx);
                Ok(ControlFlow::Break(path))
            }
            c => Ok(c),
        }
    }
}

impl<T> Visitor<T> for FindVisitor<false>
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Vec<usize>;
    type Error = Infallible;

    fn visit<'program, N>(self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        let mut stack = Vec::new();

        let mut work = VecDeque::new();
        work.push_back((usize::MAX, idx, T::from(node)));

        let mut visitor = self;

        struct ChildCollector<'a, T> {
            reference: Either<(usize, usize), usize>,
            discriminant: usize,
            parent: usize,
            work: &'a mut VecDeque<(usize, usize, T)>,
        }

        impl<'a, T> Visitor<T> for ChildCollector<'a, T> {
            type Continue = Self;
            type Break = usize;
            type Error = Infallible;

            fn visit<'program, N>(self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
            where
                N: Node,
                T: From<&'program mut N>,
            {
                let span = node.span().map(|s| (s.start(), s.end()));
                let actual_ptr = node as *const N as usize;
                let t = T::from(node);
                if N::DISCRIMINANT == self.discriminant {
                    match (&self.reference, span) {
                        (&Either::Left((s1, e1)), Some((s2, e2))) if s1 == s2 && e1 == e2 => {
                            return Ok(ControlFlow::Break(idx));
                        }
                        (&Either::Right(ptr), _) if ptr == actual_ptr => {
                            return Ok(ControlFlow::Break(idx));
                        }
                        _ => {}
                    }
                }
                self.work.push_back((self.parent, idx, t));
                Ok(ControlFlow::Continue(self))
            }
        }

        while let Some((parent, idx, next)) = work.pop_front() {
            let next_parent = stack.len();
            stack.push((parent, idx));

            let collector = ChildCollector {
                reference: visitor.reference,
                discriminant: visitor.discriminant,
                parent: next_parent,
                work: &mut work,
            };

            match next.visit_each(collector)? {
                ControlFlow::Continue(_) => {}
                ControlFlow::Break(c) => {
                    let mut parent = next_parent;
                    visitor.path.push(c);
                    while parent != 0 {
                        let (next_parent, idx) = stack[parent];
                        visitor.path.push(idx);
                        parent = next_parent;
                    }
                    visitor.path.push(stack[0].1);
                    return Ok(ControlFlow::Break(visitor.path));
                }
            }
        }

        Ok(ControlFlow::Continue(visitor))
    }
}
