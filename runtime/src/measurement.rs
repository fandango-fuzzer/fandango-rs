use crate::operators::CheckVisitor;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::ControlFlow;
use core::slice;
use either::Either;
use fandango::typing::{AsNodeRef, Node, Opaque};
use fandango::visitor::error::InvalidPath;
use fandango::visitor::navigation::{CountNodes, NodeCountVisitor};
use fandango::visitor::{VisitResult, VisitableChildren, Visitor};
use num_rational::Ratio;

pub trait FitnessMeasurer<'a, N>
where
    N: Node,
{
    type Value;
    type Error;

    fn check(&mut self, node: &'a N) -> Result<Violations, Self::Error>;
    fn evaluate(&mut self, node: &'a N, violations: Violations)
    -> Result<Self::Value, Self::Error>;
}

pub trait HasFitness {
    fn fitness(&self) -> Ratio<usize>;
}

pub trait HasViolations {
    fn violations(&self) -> &Violations;
}

pub struct Violations {
    violations: Vec<VecDeque<usize>>,
}

impl Violations {
    /// A list of violations, potentially unsimplified
    pub fn new(mut violations: Vec<VecDeque<usize>>) -> Self {
        Self::simplify(&mut violations);
        Self { violations }
    }

    pub fn violations(&self) -> &[VecDeque<usize>] {
        &self.violations
    }

    pub fn violations_mut(&mut self) -> &mut Vec<VecDeque<usize>> {
        &mut self.violations
    }

    pub fn simplify(violations: &mut Vec<VecDeque<usize>>) -> usize {
        violations.sort();
        let before = violations.len();
        violations.dedup_by(|p1, p2| p1.iter().zip(p2.iter()).all(|(e1, e2)| e1 == e2));
        before - violations.len()
    }
}

pub struct SimpleMeasurement {
    fitness: Ratio<usize>,
    violations: Violations,
}

impl HasFitness for SimpleMeasurement {
    fn fitness(&self) -> Ratio<usize> {
        self.fitness
    }
}

impl HasViolations for SimpleMeasurement {
    fn violations(&self) -> &Violations {
        &self.violations
    }
}

pub struct ViolationsVisitor<'a, I, V> {
    curr: &'a VecDeque<usize>,
    stack: Vec<usize>,
    rest: I,
    inner: V,
}

impl<'a, V> ViolationsVisitor<'a, slice::Iter<'a, VecDeque<usize>>, V> {
    pub fn new(violations: &'a Violations, inner: V) -> Option<Self> {
        let mut rest = violations.violations().iter();
        Some(Self {
            curr: rest.next()?,
            stack: Vec::new(),
            rest,
            inner,
        })
    }
}

impl<'a, I, T, V> Visitor<T> for ViolationsVisitor<'a, I, V>
where
    I: Iterator<Item = &'a VecDeque<usize>>,
    T: VisitableChildren<T> + Copy,
    V: Visitor<T, Continue = V, Break = Infallible>,
{
    type Continue = Self;
    type Break = V;
    type Error = Either<InvalidPath, V::Error>;

    fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        self.stack.push(idx);
        let opaque = node.opaque();
        while self
            .stack
            .iter()
            .zip(self.curr)
            .rev() // reduce the number of checked items
            .all(|(i1, i2)| i1 == i2)
        {
            if let Some(&next) = self.curr.get(self.stack.len()) {
                match opaque.visit_nth(self, next) {
                    Ok(Ok(ControlFlow::Continue(v))) => {
                        self = v;
                    }
                    Ok(r) => return r,
                    Err(_) => return Err(Either::Left(InvalidPath)),
                }
            } else {
                // we are at exactly the right path
                self.inner = self
                    .inner
                    .visit(node, idx)
                    .map_err(|e| Either::Right(e))?
                    .continue_value()
                    .unwrap();
                if let Some(next) = self.rest.next() {
                    self.curr = next;
                    break; // by violation properties, this is guaranteed to succeed
                } else {
                    return Ok(ControlFlow::Break(self.inner));
                }
            }
        }
        self.stack.pop();
        Ok(ControlFlow::Continue(self))
    }
}

pub struct ViolationFitness<V> {
    visitor: V,
}

impl<V> ViolationFitness<V> {
    pub fn new(visitor: V) -> Self {
        Self { visitor }
    }
}

impl<'a, N, V> FitnessMeasurer<'a, N> for ViolationFitness<V>
where
    N: Node + 'a,
    V: CheckVisitor<N::Type<'a>>,
{
    type Value = SimpleMeasurement;
    type Error = Either<InvalidPath, <V as Visitor<N::Type<'a>>>::Error>;

    fn check(&mut self, node: &'a N) -> Result<Violations, Self::Error> {
        Ok(Violations::new(
            self.visitor
                .clone()
                .visit(node, 0)
                .map_err(|e| Either::Right(e))?
                .continue_value()
                .unwrap()
                .violations(),
        ))
    }

    fn evaluate(
        &mut self,
        node: &'a N,
        violations: Violations,
    ) -> Result<Self::Value, Self::Error> {
        let total = node.count_nodes();
        let num_violations =
            if let Some(visitor) = ViolationsVisitor::new(&violations, NodeCountVisitor::new()) {
                visitor
                    .visit(node, 0)
                    .map_err(|e| Either::Left(e.left().expect("node counting is infallible")))?
                    .break_value()
                    .ok_or(Either::Left(InvalidPath))?
                    .count()
            } else {
                0
            };
        let fitness = Ratio::new(total - num_violations, total);
        Ok(SimpleMeasurement {
            fitness,
            violations,
        })
    }
}
