//! Utility visitors for navigating type trees.

use crate::typing::Node;
use crate::visitor::error::InvalidPath;
use crate::visitor::{VisitResult, VisitWith, VisitableChildren, Visitor};
use either::Either;
use std::collections::VecDeque;
use std::convert::Infallible;
use std::ops::ControlFlow;

pub trait StartingFrom {
    type WithPath;

    fn starting_from(self, from: VecDeque<usize>) -> Self::WithPath;
}

/// Find a given node, by DFS or BFS.
#[derive(Clone, Debug)]
pub struct FindVisitor<const DFS: bool> {
    reference: Either<(usize, usize), usize>,
    discriminant: usize,
    from: VecDeque<usize>,
}

impl<const DFS: bool> FindVisitor<DFS> {
    fn new<N>(target: &N) -> Self
    where
        N: Node,
    {
        Self::new_from(target, VecDeque::new())
    }

    fn new_from<N>(target: &N, from: VecDeque<usize>) -> Self
    where
        N: Node,
    {
        Self {
            reference: match target.span() {
                None => Either::Right(target as *const N as usize),
                Some(s) => Either::Left((s.start(), s.end())),
            },
            discriminant: N::DISCRIMINANT,
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
    pub fn dfs_from<N>(target: &N, from: VecDeque<usize>) -> Self
    where
        N: Node,
    {
        Self::new_from(target, from)
    }
}

impl StartingFrom for FindVisitor<true> {
    type WithPath = Self;

    fn starting_from(self, from: VecDeque<usize>) -> Self::WithPath {
        Self::WithPath {
            reference: self.reference,
            discriminant: self.discriminant,
            from,
        }
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
    type Break = VecDeque<usize>;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        let span = node.span().map(|s| (s.start(), s.end()));
        let actual_ptr = node as *const N as usize;
        if N::DISCRIMINANT == self.discriminant {
            match (&self.reference, span) {
                (&Either::Left((s1, e1)), Some((s2, e2))) if s1 == s2 && e1 == e2 => {
                    let mut path = VecDeque::new();
                    path.push_front(idx);
                    return Ok(ControlFlow::Break(path));
                }
                (&Either::Right(ptr), _) if ptr == actual_ptr => {
                    let mut path = VecDeque::new();
                    path.push_front(idx);
                    return Ok(ControlFlow::Break(path));
                }
                _ => {}
            }
        }
        match {
            let from = self.from.pop_front().unwrap_or(0);
            T::from(node).visit_each_from(self, from)
        }? {
            ControlFlow::Break(mut path) => {
                path.push_front(idx);
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
    type Break = VecDeque<usize>;
    type Error = Infallible;

    fn visit<'program, N>(self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        let mut stack = Vec::new();

        let mut work = VecDeque::new();
        work.push_back((usize::MAX, idx, T::from(node)));

        struct ChildCollector<'a, T> {
            reference: Either<(usize, usize), usize>,
            discriminant: usize,
            parent: usize,
            work: &'a mut VecDeque<(usize, usize, T)>,
        }

        impl<T> Visitor<T> for ChildCollector<'_, T> {
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
                reference: self.reference,
                discriminant: self.discriminant,
                parent: next_parent,
                work: &mut work,
            };

            match next.visit_each(collector)? {
                ControlFlow::Continue(_) => {}
                ControlFlow::Break(c) => {
                    let mut path = VecDeque::new();
                    let mut parent = next_parent;
                    path.push_front(c);
                    while parent != 0 {
                        let (next_parent, idx) = stack[parent];
                        path.push_front(idx);
                        parent = next_parent;
                    }
                    path.push_front(stack[0].1);
                    return Ok(ControlFlow::Break(path));
                }
            }
        }

        Ok(ControlFlow::Continue(self))
    }
}

#[derive(Debug, Clone)]
pub struct Advance<const FORWARD: bool> {
    count: usize,
    target: usize,
    from: VecDeque<usize>,
}

impl<const FORWARD: bool> Advance<FORWARD> {
    fn new(target: usize) -> Self {
        Self {
            count: 0,
            target,
            from: VecDeque::new(),
        }
    }
}

impl<const FORWARD: bool> StartingFrom for Advance<FORWARD> {
    type WithPath = Self;

    fn starting_from(mut self, from: VecDeque<usize>) -> Self::WithPath {
        self.from = from;
        self
    }
}

impl Advance<true> {
    pub fn forward(target: usize) -> Self {
        Self::new(target)
    }
}

impl<T> Visitor<T> for Advance<true>
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = T;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        if self.count == self.target {
            return Ok(ControlFlow::Break(T::from(node)));
        }
        self.count += 1;
        let starting_at = self.from.pop_front().unwrap_or(0);
        match T::from(node).visit_each_from(self, starting_at)? {
            ControlFlow::Continue(visitor) => Ok(ControlFlow::Continue(visitor)),
            b => Ok(b),
        }
    }
}

#[derive(Debug, Default)]
pub struct GoToVisitor {
    to: VecDeque<usize>,
}

impl GoToVisitor {
    pub fn new(to: VecDeque<usize>) -> Self {
        Self { to }
    }
}

impl<T> Visitor<T> for GoToVisitor
where
    T: VisitableChildren<T>,
{
    type Continue = Infallible;
    type Break = T;
    type Error = InvalidPath;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        if let Some(next) = self.to.pop_front() {
            T::from(node)
                .visit_nth(self, next)
                .map_err(|_| InvalidPath)?
        } else {
            Ok(ControlFlow::Break(T::from(node)))
        }
    }
}

pub trait GoTo<'a> {
    type Returned;

    fn go_to(
        &'a mut self,
        idx: usize,
        path: VecDeque<usize>,
    ) -> Result<Self::Returned, InvalidPath>;
}

pub trait GoToWith<'a>: Sized {
    type Value;

    fn go_to(&'a mut self, idx: usize, path: VecDeque<usize>) -> Result<Self::Value, InvalidPath>;
}

impl<'a, N> GoTo<'a> for N
where
    N: Node + 'a,
    GoToVisitor: Visitor<N::TypeMut<'a>, Break = N::TypeMut<'a>, Error = InvalidPath>,
    N::TypeMut<'a>: From<&'a mut N>,
{
    type Returned = N::TypeMut<'a>;

    fn go_to(
        &'a mut self,
        idx: usize,
        path: VecDeque<usize>,
    ) -> Result<N::TypeMut<'a>, InvalidPath> {
        Ok(GoToVisitor::new(path)
            .visit(self, idx)?
            .break_value()
            .unwrap())
    }
}

impl<'a, T> GoToWith<'a> for T
where
    T: VisitWith<'a, GoToVisitor>,
    T::Visited: VisitableChildren<T::Visited>,
{
    type Value = T::Visited;

    fn go_to(&'a mut self, idx: usize, path: VecDeque<usize>) -> Result<T::Visited, InvalidPath> {
        Ok(self
            .visit_with(GoToVisitor::new(path), idx)?
            .break_value()
            .unwrap())
    }
}

#[derive(Debug, Default)]
pub struct NodeCountVisitor {
    count: usize,
    from: VecDeque<usize>,
}

impl NodeCountVisitor {
    pub fn new() -> Self {
        Self {
            count: 0,
            from: VecDeque::new(),
        }
    }

    pub fn count(self) -> usize {
        self.count
    }
}

impl StartingFrom for NodeCountVisitor {
    type WithPath = Self;

    fn starting_from(mut self, from: VecDeque<usize>) -> Self::WithPath {
        self.from = from;
        self
    }
}

impl<T> Visitor<T> for NodeCountVisitor
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = InvalidPath;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        self.count += 1;
        GoToVisitor::new(std::mem::take(&mut self.from))
            .visit(node, idx)?
            .break_value()
            .unwrap()
            .visit_each(self)
    }
}

pub trait CountNodes<'a> {
    fn count_nodes(&'a mut self) -> usize;
}

pub trait CountNodesWith<'a> {
    fn count_nodes(&'a mut self) -> usize;
}

impl<'a, N> CountNodes<'a> for N
where
    N: Node + 'a,
    NodeCountVisitor: Visitor<N::TypeMut<'a>, Continue = NodeCountVisitor, Error = InvalidPath>,
    N::TypeMut<'a>: From<&'a mut N>,
{
    fn count_nodes(&'a mut self) -> usize {
        NodeCountVisitor::new()
            .visit(self, 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .count()
    }
}

impl<'a, T> CountNodesWith<'a> for T
where
    T: VisitWith<'a, NodeCountVisitor>,
    T::Visited: VisitableChildren<T::Visited>,
{
    fn count_nodes(&'a mut self) -> usize {
        self.visit_with(NodeCountVisitor::new(), 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .count()
    }
}
