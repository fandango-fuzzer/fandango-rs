//! Measurement primitives for performing the evolutionary algorithms

use crate::operators::Checker;
use alloc::collections::VecDeque;
use alloc::vec;
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

/// A measurer which computes some measurement over a given node
pub trait FitnessMeasurer<'a, N>
where
    N: Node,
{
    /// The measurement, e.g. [`SimpleMeasurement`]
    ///
    /// Measurements should implement [`HasFitness`] and [`HasViolations`], at least. Trivial
    /// measurements (e.g., integral or floating types and their [`Reverse`]) may be used directly.
    type Measurement;
    /// An error which may occur during the fitness evaluation that should halt evolution
    type Error;

    /// Performs the measurement
    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurement, Self::Error>;
}

/// Trait which indicates that this value contains a measurement (e.g.,
/// [`crate::evolvers::basic::BasicIndividual`])
pub trait HasMeasurement {
    /// The measurement contained within
    type Measurement;

    /// Get the measurement
    fn measurement(&self) -> &Self::Measurement;
}

/// Measurement which contains a fitness value
pub trait HasFitness {
    /// The fitness associated with this measurement
    type Fitness;

    /// The fitness of this measurement
    fn fitness(&self) -> &Self::Fitness;

    /// Extract the measurement and replace it with a default
    fn take_fitness(&mut self) -> Self::Fitness;
}

/// Measurement that contains a list of violations
///
/// Implementors which do not actually have any violations to report can use [`EMPTY_VIOLATIONS`]
/// instead.
pub trait HasViolations {
    /// The violations associated with this measurement
    fn violations(&self) -> &Violations;

    /// Extract the violations and replace it with [`Violations::default`]
    fn take_violations(&mut self) -> Violations;
}

/// A list of violations and a corresponding pass rate (i.e., the number of checks which passed over
/// those which did not).
pub struct Violations {
    pass_rate: Ratio<usize>,
    violations: Vec<VecDeque<usize>>,
}

/// An empty violations set, with the pass rate set to 100%
pub static EMPTY_VIOLATIONS: Violations = Violations {
    pass_rate: Ratio::new_raw(1, 1),
    violations: Vec::new(),
};

impl Default for Violations {
    fn default() -> Self {
        Violations {
            pass_rate: Ratio::new(1, 1),
            violations: vec![],
        }
    }
}

impl Violations {
    /// Create a list of violations, deduplicating the violations by path prefix
    pub fn new(pass_rate: Ratio<usize>, mut violations: Vec<VecDeque<usize>>) -> Self {
        let _removed = Self::simplify(&mut violations);
        Self {
            pass_rate,
            violations,
        }
    }

    /// The list of violations
    pub fn violations(&self) -> &[VecDeque<usize>] {
        &self.violations
    }

    /// Consume this list of violations and turn it into its components
    pub fn into_raw(self) -> (Ratio<usize>, Vec<VecDeque<usize>>) {
        (self.pass_rate, self.violations)
    }

    /// Simplify a list of violations, returning the number which we've removed as a result
    pub fn simplify(violations: &mut Vec<VecDeque<usize>>) -> usize {
        violations.sort();
        let before = violations.len();
        violations.dedup_by(|p1, p2| p1.iter().zip(p2.iter()).all(|(e1, e2)| e1 == e2));
        before - violations.len()
    }

    /// The pass rate associated with this list of violations
    pub fn pass_rate(&self) -> Ratio<usize> {
        self.pass_rate
    }
}

/// A very simple measurement, consisting only of a fractional fitness and a list of violations
pub struct SimpleMeasurement {
    /// The fitness of this measurement
    pub fitness: Ratio<usize>,
    /// The violations of this measurement
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

/// A [`Visitor`] which traverses to all the violations provided and calls back to [`V`], another
/// [`Visitor`], over only these subtrees
pub struct ViolationsVisitor<'a, I, V> {
    curr: &'a VecDeque<usize>,
    stack: Vec<usize>,
    rest: I,
    inner: V,
}

impl<'a, V> ViolationsVisitor<'a, slice::Iter<'a, VecDeque<usize>>, V> {
    /// Create this visitor with the provided visitor to be used over only violating subtrees
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

/// [`FitnessMeasurer`] using a visitor to extract the corresponding measurement
pub struct ViolationFitness<V> {
    phantom: PhantomData<V>,
}

impl<V> Default for ViolationFitness<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> ViolationFitness<V> {
    /// Create the fitness measurer
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
    type Measurement = SimpleMeasurement;
    type Error = Error;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurement, Self::Error> {
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

/// A [`FitnessMeasurer`] which tries to reduce the size
pub struct SizeFitness;

impl<'a, N> FitnessMeasurer<'a, N> for SizeFitness
where
    N: Node,
{
    type Measurement = Reverse<usize>;
    type Error = Infallible;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurement, Self::Error> {
        Ok(Reverse(node.count_bytes()))
    }
}

macro_rules! impl_trivial_fitness {
    ($fitness: ty) => {
        impl HasFitness for $fitness {
            type Fitness = $fitness;

            fn fitness(&self) -> &Self::Fitness {
                self
            }

            fn take_fitness(&mut self) -> Self::Fitness {
                ::core::mem::take(self)
            }
        }

        impl HasViolations for $fitness {
            fn violations(&self) -> &Violations {
                &$crate::measurement::EMPTY_VIOLATIONS
            }

            fn take_violations(&mut self) -> Violations {
                Violations::default()
            }
        }
    };
}

impl_trivial_fitness!(usize);
impl_trivial_fitness!(u128);
impl_trivial_fitness!(u64);
impl_trivial_fitness!(u32);
impl_trivial_fitness!(u16);
impl_trivial_fitness!(u8);

impl_trivial_fitness!(isize);
impl_trivial_fitness!(i128);
impl_trivial_fitness!(i64);
impl_trivial_fitness!(i32);
impl_trivial_fitness!(i16);
impl_trivial_fitness!(i8);

impl_trivial_fitness!(f64);
impl_trivial_fitness!(f32);

impl<T> HasFitness for Reverse<T>
where
    T: HasFitness<Fitness = T> + Default,
{
    type Fitness = Reverse<T>;

    fn fitness(&self) -> &Self::Fitness {
        self
    }

    fn take_fitness(&mut self) -> Self::Fitness {
        mem::take(self)
    }
}

impl<T> HasViolations for Reverse<T>
where
    T: HasViolations,
{
    fn violations(&self) -> &Violations {
        self.0.violations()
    }

    fn take_violations(&mut self) -> Violations {
        self.0.take_violations()
    }
}
