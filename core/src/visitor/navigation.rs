//! Utility visitors for navigating type trees.

use crate::typing::{AsNodeMut, AsNodeRef, Node, Opaque, OpaqueMut};
use crate::visitor::error::InvalidPath;
use crate::visitor::{
    VisitMutResult, VisitResult, VisitWith, VisitableChildren, VisitableChildrenMut, Visitor,
    VisitorMut,
};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::ControlFlow;

/// Find a given node, by DFS or BFS.
#[derive(Clone, Debug)]
pub struct FindVisitor<T, const DFS: bool> {
    reference: T,
}

impl<T, const DFS: bool> FindVisitor<T, DFS> {
    fn new<'a, N>(target: &'a N) -> Self
    where
        N: Node<Type<'a> = T>,
        T: From<&'a N>,
    {
        Self {
            reference: target.opaque(),
        }
    }
}

impl<T> FindVisitor<T, true> {
    /// Search for a node by DFS.
    pub fn dfs<'a, N>(target: &'a N) -> Self
    where
        N: Node<Type<'a> = T>,
        T: From<&'a N>,
    {
        Self::new(target)
    }
}

impl<T> FindVisitor<T, false> {
    /// Search for a node by BFS.
    pub fn bfs<'a, N>(target: &'a N) -> Self
    where
        N: Node<Type<'a> = T>,
        T: From<&'a N>,
    {
        Self::new(target)
    }
}

impl<T, U> Visitor<T> for FindVisitor<U, true>
where
    T: VisitableChildren<T> + PartialEq<U>,
{
    type Continue = Self;
    type Break = VecDeque<usize>;
    type Error = Infallible;

    fn visit<'program, N>(self, node: &'program N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        let opaque = node.opaque();
        if opaque == self.reference {
            let mut path = VecDeque::new();
            path.push_front(idx);
            Ok(ControlFlow::Break(path))
        } else {
            match opaque.visit_each(self)? {
                ControlFlow::Break(mut path) => {
                    path.push_front(idx);
                    Ok(ControlFlow::Break(path))
                }
                c => Ok(c),
            }
        }
    }
}

impl<T, U> Visitor<T> for FindVisitor<U, false>
where
    T: VisitableChildren<T> + PartialEq<U>,
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
        work.push_back((usize::MAX, idx, node.opaque()));

        struct ChildCollector<'a, T, U> {
            reference: &'a U,
            parent: usize,
            work: &'a mut VecDeque<(usize, usize, T)>,
        }

        impl<T, U> Visitor<T> for ChildCollector<'_, T, U>
        where
            T: PartialEq<U>,
        {
            type Continue = Self;
            type Break = usize;
            type Error = Infallible;

            fn visit<'program, N>(self, node: &'program N, idx: usize) -> VisitResult<Self, T>
            where
                N: Node<Type<'program> = T>,
                T: From<&'program N> + AsNodeRef<N>,
            {
                let t = node.opaque();
                if t == *self.reference {
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
                reference: &self.reference,
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
}

impl<const FORWARD: bool, const REF: bool> Advance<FORWARD, REF> {
    fn new(target: usize) -> Self {
        Self { count: 0, target }
    }
}

impl Advance<true, false> {
    /// Advance forwards within the tree and return a path.
    #[must_use]
    pub fn forward(target: usize) -> Self {
        Self::new(target)
    }
}

impl Advance<true, true> {
    /// Advance forwards within the tree and return a reference.
    #[must_use]
    pub fn forward_ref(target: usize) -> Self {
        Self::new(target)
    }
}

impl Advance<false, false> {
    /// Advance backwards within the tree and return a path.
    #[must_use]
    pub fn backwards(target: usize) -> Self {
        Self::new(target)
    }
}

impl Advance<false, true> {
    /// Advance backwards within the tree and return a tree.
    #[must_use]
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
        let mut traversal = if self.count == self.target {
            Ok(ControlFlow::Break(VecDeque::new()))
        } else {
            self.count += 1;
            if FORWARD {
                node.opaque().visit_each(self)
            } else {
                node.opaque().visit_each_reverse(self)
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
        if self.count == self.target {
            Ok(ControlFlow::Break(node.opaque()))
        } else {
            self.count += 1;
            if FORWARD {
                node.opaque().visit_each(self)
            } else {
                node.opaque().visit_each_reverse(self)
            }
        }
    }
}

impl<T, const FORWARD: bool> VisitorMut<T> for Advance<FORWARD, true>
where
    T: VisitableChildrenMut<T>,
{
    type Continue = Self;
    type Break = T;
    type Error = Infallible;

    fn visit_mut<'program, N>(
        mut self,
        node: &'program mut N,
        _idx: usize,
    ) -> VisitMutResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        if self.count == self.target {
            Ok(ControlFlow::Break(node.opaque_mut()))
        } else {
            self.count += 1;
            if FORWARD {
                node.opaque_mut().visit_each_mut(self)
            } else {
                node.opaque_mut().visit_each_reverse_mut(self)
            }
        }
    }
}

/// A visitor which goes to a provided path and fetches the node.
#[derive(Debug, Default)]
pub struct GoToVisitor<'a> {
    to: &'a [usize],
}

impl<'a> GoToVisitor<'a> {
    /// Create the visitor with the intended destination (not including the start node; see
    /// [`VisitFrom`] and [`crate::visitor_chain`] for details.
    #[must_use]
    pub fn new(to: &'a [usize]) -> Self {
        Self { to }
    }
}

impl<T> Visitor<T> for GoToVisitor<'_>
where
    T: VisitableChildren<T>,
{
    type Continue = Infallible;
    type Break = T;
    type Error = InvalidPath;

    fn visit<'program, N>(self, node: &'program N, _: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        if let Some((&next, to)) = self.to.split_first() {
            node.opaque()
                .visit_nth(Self { to }, next)
                .map_err(|_| InvalidPath)?
        } else {
            Ok(ControlFlow::Break(node.opaque()))
        }
    }
}

impl<T> VisitorMut<T> for GoToVisitor<'_>
where
    T: VisitableChildrenMut<T>,
{
    type Continue = Infallible;
    type Break = T;
    type Error = InvalidPath;

    fn visit_mut<'program, N>(self, node: &'program mut N, _: usize) -> VisitMutResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        if let Some((&next, to)) = self.to.split_first() {
            node.opaque_mut()
                .visit_nth_mut(Self { to }, next)
                .map_err(|_| InvalidPath)?
        } else {
            Ok(ControlFlow::Break(node.opaque_mut()))
        }
    }
}

/// Helper trait to use [`GoToVisitor`] as a method on the derivation tree from the provided node.
pub trait GoTo<'b> {
    /// The type which is returned (see [`VisitWith`] for why this is necessary).
    type Value;

    /// Perform the traversal!
    fn go_to(self, idx: usize, path: &'b [usize]) -> Result<Self::Value, InvalidPath>;
}

impl<'a, 'b, N> GoTo<'b> for &'a N
where
    N: Node + 'a,
    GoToVisitor<'b>: Visitor<N::Type<'a>, Break = N::Type<'a>, Error = InvalidPath>,
{
    type Value = N::Type<'a>;

    fn go_to(self, idx: usize, path: &'b [usize]) -> Result<N::Type<'a>, InvalidPath> {
        Ok(GoToVisitor::new(path)
            .visit(self, idx)?
            .break_value()
            .unwrap())
    }
}

/// Helper trait to use [`GoToVisitor`] as a method on the derivation tree from the provided node
/// while accessing the resulting node mutably.
pub trait GoToMut<'b> {
    /// The type which is returned (see [`super::VisitWithMut`] for why this is necessary).
    type Value;

    /// Perform the traversal!
    fn go_to_mut(self, idx: usize, path: &'b [usize]) -> Result<Self::Value, InvalidPath>;
}

impl<'a, 'b, N> GoToMut<'b> for &'a mut N
where
    N: Node + 'a,
    GoToVisitor<'b>: VisitorMut<N::TypeMut<'a>, Break = N::TypeMut<'a>, Error = InvalidPath>,
    N::TypeMut<'a>: From<&'a mut N> + AsNodeMut<N>,
{
    type Value = N::TypeMut<'a>;

    fn go_to_mut(self, idx: usize, path: &'b [usize]) -> Result<N::TypeMut<'a>, InvalidPath> {
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
}

impl NodeCountVisitor {
    /// Create a new counter.
    #[must_use]
    pub fn new() -> Self {
        Self { count: 0 }
    }

    /// Consume the visitor and acquire the final count.
    #[must_use]
    pub fn count(self) -> usize {
        self.count
    }
}

impl<T> Visitor<T> for NodeCountVisitor
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = InvalidPath;

    fn visit<'program, N>(mut self, node: &'program N, _: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        self.count += 1;
        node.opaque().visit_each(self)
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
}

impl ByteCountVisitor {
    /// Create a new counter.
    #[must_use]
    pub fn new() -> Self {
        Self { count: 0 }
    }

    /// Consume the visitor and acquire the final count.
    #[must_use]
    pub fn count(self) -> usize {
        self.count
    }
}

impl<T> Visitor<T> for ByteCountVisitor
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = InvalidPath;

    fn visit<'program, N>(mut self, node: &'program N, _: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        self.count += 1;
        node.opaque().visit_each(self)
    }
}

/// Helper trait to use [`ByteCountVisitor`] as a method on the derivation tree from the provided
/// node.
pub trait CountBytes<'a> {
    /// Perform the count!
    fn count_bytes(&'a self) -> usize;
}

/// Helper trait to use [`ByteCountVisitor`] as a method on the derivation tree from the provided
/// opaque node.
pub trait CountBytesWith<'a> {
    /// Perform the count!
    fn count_bytes(&'a self) -> usize;
}

impl<'a, N> CountBytes<'a> for N
where
    N: Node + 'a,
    ByteCountVisitor: Visitor<N::Type<'a>, Continue = ByteCountVisitor, Error = InvalidPath>,
    N::Type<'a>: From<&'a N> + AsNodeRef<N>,
{
    fn count_bytes(&'a self) -> usize {
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
    fn count_bytes(&'a self) -> usize {
        self.visit_with(ByteCountVisitor::new(), 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .count()
    }
}

/// Helper trait to start a visitor at a given traversal prefix (i.e. from a given path onward by
/// pre-order traversal)
pub trait StartingFrom: Sized {
    /// Start this visitor at a given path
    fn starting_from(self, path: &[usize]) -> VisitFrom<'_, Self>;
}

impl<V> StartingFrom for V {
    fn starting_from(self, from: &[usize]) -> VisitFrom<'_, Self> {
        VisitFrom {
            from,
            visitor: self,
        }
    }
}

/// Visitor which uses another visitor only starting from a given path by pre-order traversal
///
/// Use with [`StartingFrom`].
pub struct VisitFrom<'a, V> {
    from: &'a [usize],
    visitor: V,
}

impl<V> VisitFrom<'_, V> {
    /// Retrieve the contained visitor
    pub fn inner(self) -> V {
        self.visitor
    }
}

impl<T, V> Visitor<T> for VisitFrom<'_, V>
where
    V: Visitor<T, Continue = V>,
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = V::Break;
    type Error = V::Error;

    fn visit<'program, N>(self, node: &'program N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        let result = if let Some((&current, from)) = self.from.split_first() {
            if current == idx {
                if from.is_empty() {
                    self.visitor.visit(node, idx)
                } else {
                    return node.opaque().visit_each_from(
                        Self {
                            from,
                            visitor: self.visitor,
                        },
                        from[0],
                    );
                }
            } else if current < idx {
                self.visitor.visit(node, idx)
            } else {
                return Ok(ControlFlow::Continue(self)); // nothing to do
            }
        } else {
            // very rare corner case: no path provided, ever
            self.visitor.visit(node, idx)
        };
        match result {
            Ok(ControlFlow::Continue(visitor)) => Ok(ControlFlow::Continue(Self {
                from: self.from,
                visitor,
            })),
            Ok(ControlFlow::Break(b)) => Ok(ControlFlow::Break(b)),
            Err(e) => Err(e),
        }
    }
}
