use crate::operators::Checker;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use anyhow::Error;
use core::cmp::Reverse;
use core::convert::Infallible;
use core::marker::PhantomData;
use core::ops::ControlFlow;
use core::{mem, slice};
use either::Either;
use fandango::typing::{AsNodeRef, Node, Opaque};
use fandango::visitor::error::InvalidPath;
use fandango::visitor::navigation::CountBytes;
use fandango::visitor::{VisitResult, VisitableChildren, Visitor};
use num_rational::Ratio;

pub trait FitnessMeasurer<'a, N>
where
    N: Node,
{
    type Value;
    type Error;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Value, Self::Error>;
}

pub trait HasMeasurement {
    type Measurement;

    fn measurement(&self) -> &Self::Measurement;
}

pub trait HasFitness {
    type Fitness;

    fn fitness(&self) -> &Self::Fitness;

    fn take_fitness(&mut self) -> Self::Fitness;
}

pub trait HasViolations {
    fn violations(&self) -> &Violations;

    fn take_violations(&mut self) -> Violations;
}

#[derive(Default)]
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

    pub fn into_raw(self) -> (Ratio<usize>, Vec<VecDeque<usize>>) {
        (self.pass_rate, self.violations)
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
    pub fitness: Ratio<usize>,
    pub violations: Violations,
}

impl HasFitness for SimpleMeasurement {
    type Fitness = Ratio<usize>;

    fn fitness(&self) -> &Self::Fitness {
        &self.fitness
    }

    fn take_fitness(&mut self) -> Self::Fitness {
        mem::take(&mut self.fitness)
    }
}

impl HasViolations for SimpleMeasurement {
    fn violations(&self) -> &Violations {
        &self.violations
    }

    fn take_violations(&mut self) -> Violations {
        mem::take(&mut self.violations)
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

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Value, Self::Error> {
        let violations = V::default()
            .visit(node, 0)?
            .continue_value()
            .unwrap()
            .violations();
        Ok(SimpleMeasurement {
            fitness: violations.pass_rate(),
            violations,
        })
    }
}

pub struct SizeFitness;

pub struct SizeMeasurement {
    size: Reverse<usize>,
    violations: Violations,
}

impl HasFitness for SizeMeasurement {
    type Fitness = Reverse<usize>;

    fn fitness(&self) -> &Self::Fitness {
        &self.size
    }

    fn take_fitness(&mut self) -> Self::Fitness {
        mem::take(&mut self.size)
    }
}

impl HasViolations for SizeMeasurement {
    fn violations(&self) -> &Violations {
        &self.violations
    }

    fn take_violations(&mut self) -> Violations {
        mem::take(&mut self.violations)
    }
}

impl<'a, N> FitnessMeasurer<'a, N> for SizeFitness
where
    N: Node,
{
    type Value = SizeMeasurement;
    type Error = Infallible;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Value, Self::Error> {
        Ok(SizeMeasurement {
            size: Reverse(node.count_bytes()),
            violations: Violations::default(),
        })
    }
}
