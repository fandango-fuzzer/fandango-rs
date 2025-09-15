//! Utility visitors for navigating type trees.

use crate::typing::{AsNodeMut, AsNodeRef, Node};
use crate::visitor::error::InvalidPath;
use crate::visitor::{
    VisitMutResult, VisitResult, VisitWith, VisitableChildren, VisitableChildrenMut, Visitor,
    VisitorMut,
};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::ControlFlow;

/// Trait which enables chaining of visitors by returning paths between them (e.g. with
/// [`crate::visitor_chain`]). One would choose this over simply traversing a given node if the
/// intended visitor needs to be able to traverse the full subtree, not just the provided node.
/// The path here is expected to be provided without the first node index.
///
/// If you're intending to just visit a specific node's subtree, use [`GoTo::go_to`] instead.
pub trait StartingFrom {
    /// The visitor, but with the provided path (in case of a builder pattern).
    type WithPath;

    /// Provide the intended starting path.
    fn starting_from(self, from: VecDeque<usize>) -> Self::WithPath;
}

/// Find a given node, by DFS or BFS.
#[derive(Clone, Debug)]
pub struct FindVisitor<const DFS: bool> {
    reference: usize,
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
            reference: target as *const N as usize,
            discriminant: target.discriminant(),
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

    fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        let actual_ptr = node as *const N as usize;
        if node.discriminant() == self.discriminant && actual_ptr == self.reference {
            let mut path = VecDeque::new();
            path.push_front(idx);
            Ok(ControlFlow::Break(path))
        } else {
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
}

impl<T> Visitor<T> for FindVisitor<false>
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = VecDeque<usize>;
    type Error = Infallible;

    fn visit<'program, N>(self, node: &'program N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        let mut stack = Vec::new();

        let mut work = VecDeque::new();
        work.push_back((usize::MAX, idx, T::from(node)));

        struct ChildCollector<'a, T> {
            reference: usize,
            discriminant: usize,
            parent: usize,
            work: &'a mut VecDeque<(usize, usize, T)>,
        }

        impl<T> Visitor<T> for ChildCollector<'_, T> {
            type Continue = Self;
            type Break = usize;
            type Error = Infallible;

            fn visit<'program, N>(self, node: &'program N, idx: usize) -> VisitResult<Self, T>
            where
                N: Node,
                T: From<&'program N> + AsNodeRef<N>,
            {
                let actual_ptr = node as *const N as usize;
                let discriminant = node.discriminant();
                let t = T::from(node);
                if discriminant == self.discriminant && actual_ptr == self.reference {
                    Ok(ControlFlow::Break(idx))
                } else {
                    self.work.push_back((self.parent, idx, t));
                    Ok(ControlFlow::Continue(self))
                }
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

/// Advance in the tree in pre-order traversal, forwards or backwards, and return the `n`th node. If
/// the tree has fewer nodes than `n`, the number of nodes which were traversed.
#[derive(Debug, Clone)]
pub struct Advance<const FORWARD: bool, const REF: bool> {
    count: usize,
    target: usize,
    from: VecDeque<usize>,
}

impl<const FORWARD: bool, const REF: bool> Advance<FORWARD, REF> {
    fn new(target: usize) -> Self {
        Self {
            count: 0,
            target,
            from: VecDeque::new(),
        }
    }
}

impl<const FORWARD: bool, const REF: bool> StartingFrom for Advance<FORWARD, REF> {
    type WithPath = Self;

    fn starting_from(mut self, from: VecDeque<usize>) -> Self::WithPath {
        self.from = from;
        self
    }
}

impl Advance<true, false> {
    /// Advance forwards within the tree and return a path.
    pub fn forward(target: usize) -> Self {
        Self::new(target)
    }
}

impl Advance<true, true> {
    /// Advance forwards within the tree and return a reference.
    pub fn forward_ref(target: usize) -> Self {
        Self::new(target)
    }
}

impl Advance<false, false> {
    /// Advance backwards within the tree and return a path.
    pub fn backwards(target: usize) -> Self {
        Self::new(target)
    }
}

impl Advance<false, true> {
    /// Advance backwards within the tree and return a tree.
    pub fn backwards_ref(target: usize) -> Self {
        Self::new(target)
    }
}

impl<T, const FORWARD: bool> Visitor<T> for Advance<FORWARD, false>
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = VecDeque<usize>;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        let mut traversal = if let Some(starting_at) = self.from.pop_front() {
            if FORWARD {
                T::from(node).visit_each_from(self, starting_at)
            } else {
                T::from(node).visit_each_reverse_from(self, starting_at)
            }
        } else if self.count == self.target {
            Ok(ControlFlow::Break(VecDeque::new()))
        } else {
            self.count += 1;
            if FORWARD {
                T::from(node).visit_each(self)
            } else {
                T::from(node).visit_each_reverse(self)
            }
        };
        if let Ok(ControlFlow::Break(trace)) = &mut traversal {
            trace.push_front(idx);
        }
        traversal
    }
}

impl<T, const FORWARD: bool> Visitor<T> for Advance<FORWARD, true>
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = T;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        if let Some(starting_at) = self.from.pop_front() {
            if FORWARD {
                T::from(node).visit_each_from(self, starting_at)
            } else {
                T::from(node).visit_each_reverse_from(self, starting_at)
            }
        } else if self.count == self.target {
            Ok(ControlFlow::Break(T::from(node)))
        } else {
            self.count += 1;
            if FORWARD {
                T::from(node).visit_each(self)
            } else {
                T::from(node).visit_each_reverse(self)
            }
        }
    }
}

/// A visitor which goes to a provided path and fetches the node.
#[derive(Debug, Default)]
pub struct GoToVisitor {
    to: VecDeque<usize>,
}

impl GoToVisitor {
    /// Create the visitor with the intended destination (not including the start node; see
    /// [`StartingFrom`] and [`crate::visitor_chain`] for details.
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

    fn visit<'program, N>(mut self, node: &'program N, _: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
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

impl<T> VisitorMut<T> for GoToVisitor
where
    T: VisitableChildrenMut<T>,
{
    type Continue = Infallible;
    type Break = T;
    type Error = InvalidPath;

    fn visit_mut<'program, N>(mut self, node: &'program mut N, _: usize) -> VisitMutResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        if let Some(next) = self.to.pop_front() {
            T::from(node)
                .visit_nth_mut(self, next)
                .map_err(|_| InvalidPath)?
        } else {
            Ok(ControlFlow::Break(T::from(node)))
        }
    }
}

/// Helper trait to use [`GoToVisitor`] as a method on the derivation tree from the provided node.
pub trait GoTo {
    /// The type which is returned (see [`VisitWith`] for why this is necessary).
    type Value;

    /// Perform the traversal!
    fn go_to(self, idx: usize, path: VecDeque<usize>) -> Result<Self::Value, InvalidPath>;
}

impl<'a, N> GoTo for &'a N
where
    N: Node + 'a,
    GoToVisitor: Visitor<N::Type<'a>, Break = N::Type<'a>, Error = InvalidPath>,
    N::Type<'a>: From<&'a N> + AsNodeRef<N>,
{
    type Value = N::Type<'a>;

    fn go_to(self, idx: usize, path: VecDeque<usize>) -> Result<N::Type<'a>, InvalidPath> {
        Ok(GoToVisitor::new(path)
            .visit(self, idx)?
            .break_value()
            .unwrap())
    }
}

/// Helper trait to use [`GoToVisitor`] as a method on the derivation tree from the provided node
/// while accessing the resulting node mutably.
pub trait GoToMut {
    /// The type which is returned (see [`super::VisitWithMut`] for why this is necessary).
    type Value;

    /// Perform the traversal!
    fn go_to_mut(self, idx: usize, path: VecDeque<usize>) -> Result<Self::Value, InvalidPath>;
}

impl<'a, N> GoToMut for &'a mut N
where
    N: Node + 'a,
    GoToVisitor: VisitorMut<N::TypeMut<'a>, Break = N::TypeMut<'a>, Error = InvalidPath>,
    N::TypeMut<'a>: From<&'a mut N> + AsNodeMut<N>,
{
    type Value = N::TypeMut<'a>;

    fn go_to_mut(self, idx: usize, path: VecDeque<usize>) -> Result<N::TypeMut<'a>, InvalidPath> {
        Ok(GoToVisitor::new(path)
            .visit_mut(self, idx)?
            .break_value()
            .unwrap())
    }
}

/// Count the number of nodes in the derivation tree underneath a given node.
#[derive(Debug, Default)]
pub struct NodeCountVisitor {
    count: usize,
    from: VecDeque<usize>,
}

impl NodeCountVisitor {
    /// Create a new counter.
    pub fn new() -> Self {
        Self {
            count: 0,
            from: VecDeque::new(),
        }
    }

    /// Consume the visitor and acquire the final count.
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

    fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        self.count += 1;
        GoToVisitor::new(core::mem::take(&mut self.from))
            .visit(node, idx)?
            .break_value()
            .unwrap()
            .visit_each(self)
    }
}

/// Helper trait to use [`NodeCountVisitor`] as a method on the derivation tree from the provided
/// node.
pub trait CountNodes<'a> {
    /// Perform the count!
    fn count_nodes(&'a self) -> usize;
}

/// Helper trait to use [`NodeCountVisitor`] as a method on the derivation tree from the provided
/// opaque node.
pub trait CountNodesWith<'a> {
    /// Perform the count!
    fn count_nodes(&'a self) -> usize;
}

impl<'a, N> CountNodes<'a> for N
where
    N: Node + 'a,
    NodeCountVisitor: Visitor<N::Type<'a>, Continue = NodeCountVisitor, Error = InvalidPath>,
    N::Type<'a>: From<&'a N> + AsNodeRef<N>,
{
    fn count_nodes(&'a self) -> usize {
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
    fn count_nodes(&'a self) -> usize {
        self.visit_with(NodeCountVisitor::new(), 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .count()
    }
}

/// Count the number of bytes in the derivation tree underneath a given node.
#[derive(Debug, Default)]
pub struct ByteCountVisitor {
    count: usize,
    from: VecDeque<usize>,
}

impl ByteCountVisitor {
    /// Create a new counter.
    pub fn new() -> Self {
        Self {
            count: 0,
            from: VecDeque::new(),
        }
    }

    /// Consume the visitor and acquire the final count.
    pub fn count(self) -> usize {
        self.count
    }
}

impl StartingFrom for ByteCountVisitor {
    type WithPath = Self;

    fn starting_from(mut self, from: VecDeque<usize>) -> Self::WithPath {
        self.from = from;
        self
    }
}

impl<T> Visitor<T> for ByteCountVisitor
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = InvalidPath;

    fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        self.count += 1;
        GoToVisitor::new(core::mem::take(&mut self.from))
            .visit(node, idx)?
            .break_value()
            .unwrap()
            .visit_each(self)
    }
}

/// Helper trait to use [`ByteCountVisitor`] as a method on the derivation tree from the provided
/// node.
pub trait CountBytes<'a> {
    /// Perform the count!
    fn count_bytes(&'a mut self) -> usize;
}

/// Helper trait to use [`ByteCountVisitor`] as a method on the derivation tree from the provided
/// opaque node.
pub trait CountBytesWith<'a> {
    /// Perform the count!
    fn count_bytes(&'a mut self) -> usize;
}

impl<'a, N> CountBytes<'a> for N
where
    N: Node + 'a,
    ByteCountVisitor: Visitor<N::Type<'a>, Continue = ByteCountVisitor, Error = InvalidPath>,
    N::Type<'a>: From<&'a N> + AsNodeRef<N>,
{
    fn count_bytes(&'a mut self) -> usize {
        ByteCountVisitor::new()
            .visit(self, 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .count()
    }
}

impl<'a, T> CountBytesWith<'a> for T
where
    T: VisitWith<'a, ByteCountVisitor>,
    T::Visited: VisitableChildren<T::Visited>,
{
    fn count_bytes(&'a mut self) -> usize {
        self.visit_with(ByteCountVisitor::new(), 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .count()
    }
}
