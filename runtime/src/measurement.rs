use crate::operators::Checker;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use anyhow::Error;
use core::convert::Infallible;
use core::marker::PhantomData;
use core::ops::ControlFlow;
use core::slice;
use either::Either;
use fandango::typing::{AsNodeRef, Node, Opaque};
use fandango::visitor::error::InvalidPath;
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
    pass_rate: Ratio<usize>,
    violations: Vec<VecDeque<usize>>,
}

impl Violations {
    /// A list of violations, potentially unsimplified
    pub fn new(pass_rate: Ratio<usize>, mut violations: Vec<VecDeque<usize>>) -> Self {
        let removed = Self::simplify(&mut violations);
        Self {
            pass_rate: Ratio::new(*pass_rate.numer() - removed, *pass_rate.denom() - removed),
            violations,
        }
    }

    pub fn violations(&self) -> &[VecDeque<usize>] {
        &self.violations
    }

    pub fn simplify(violations: &mut Vec<VecDeque<usize>>) -> usize {
        violations.sort();
        let before = violations.len();
        violations.dedup_by(|p1, p2| p1.iter().zip(p2.iter()).all(|(e1, e2)| e1 == e2));
        before - violations.len()
    }

    pub fn pass_rate(&self) -> Ratio<usize> {
        self.pass_rate
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
                    .map_err(Either::Right)?
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
    phantom: PhantomData<V>,
}

impl<V> Default for ViolationFitness<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> ViolationFitness<V> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<'a, N, V> FitnessMeasurer<'a, N> for ViolationFitness<V>
where
    N: Node + 'a,
    V: Visitor<N::Type<'a>, Error = Infallible, Break = Infallible, Continue = V> + Checker,
{
    type Value = SimpleMeasurement;
    type Error = Error;

    fn check(&mut self, node: &'a N) -> Result<Violations, Self::Error> {
        Ok(V::default()
            .visit(node, 0)?
            .continue_value()
            .unwrap()
            .violations())
    }

    fn evaluate(
        &mut self,
        _node: &'a N,
        violations: Violations,
    ) -> Result<Self::Value, Self::Error> {
        Ok(SimpleMeasurement {
            fitness: violations.pass_rate(),
            violations,
        })
    }
}
