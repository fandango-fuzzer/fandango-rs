use crate::generation::InPlaceGenerated;
use crate::typing::Node;
use crate::visitor::error::InvalidPath;
use crate::visitor::navigation::StartingFrom;
use crate::visitor::{VisitResult, VisitableChildren, Visitor};
use std::collections::VecDeque;
use std::convert::Infallible;
use std::ops::ControlFlow;

pub struct Mutator<'s, 'g, S, G> {
    sampler: &'s mut S,
    generator: &'g mut G,
    path: VecDeque<usize>,
}

impl<'s, 'g, S, G> Mutator<'s, 'g, S, G> {
    pub fn new(sampler: &'s mut S, generator: &'g mut G) -> Self {
        Self {
            sampler,
            generator,
            path: VecDeque::new(),
        }
    }
}

impl<'s, 'g, S, G> StartingFrom for Mutator<'s, 'g, S, G> {
    type WithPath = Self;

    fn starting_from(self, path: VecDeque<usize>) -> Self::WithPath {
        Self::WithPath {
            sampler: self.sampler,
            generator: self.generator,
            path,
        }
    }
}

impl<'s, 'g, T, S, G> Visitor<T> for Mutator<'s, 'g, S, G>
where
    T: VisitableChildren<T> + InPlaceGenerated<S, G>,
{
    type Continue = Infallible;
    type Break = T;
    type Error = InvalidPath;

    fn visit<'program, N>(mut self, node: &'program mut N, _: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        node.clear_span();
        if let Some(i) = self.path.pop_front() {
            T::from(node).visit_nth(self, i).map_err(|_| InvalidPath)?
        } else {
            Ok(ControlFlow::Break(
                T::from(node).generate_in_place(self.sampler, self.generator),
            ))
        }
    }
}
