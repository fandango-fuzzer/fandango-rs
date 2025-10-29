use crate::measurement::{FitnessMeasurer, HasFitness, HasViolations, Violations};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use anyhow::Error;
use core::cmp::{Ordering, Reverse};
use core::mem;
use core::ops::ControlFlow;
use fandango::typing::Node;
use num_rational::Ratio;

/// Pairwise evaluation of [`PartialOrd`] on two tuple lists
///
/// Evaluates a pair of tuple lists (i.e., a Haskell-style list of heterogeneous elements) into a
/// tuple list of the same length, but where each member is the [`PartialOrd`] ordering as evaluated
/// pairwise with the other list.
pub trait TuplePartialOrd<Rhs> {
    /// The output tuple list
    type Output;

    /// Evaluate [`PartialOrd`] across [`Self`] and [`Rhs`].
    fn partial_cmp_many(&self, other: &Rhs) -> Self::Output;
}

impl<H1, H2, T1, T2> TuplePartialOrd<(H2, T2)> for (H1, T1)
where
    H1: PartialOrd<H2>,
    T1: TuplePartialOrd<T2>,
{
    type Output = (Option<Ordering>, T1::Output);

    fn partial_cmp_many(&self, other: &(H2, T2)) -> Self::Output {
        (
            self.0.partial_cmp(&other.0),
            self.1.partial_cmp_many(&other.1),
        )
    }
}

impl TuplePartialOrd<()> for () {
    type Output = ();

    fn partial_cmp_many(&self, _other: &()) -> Self::Output {}
}

/// A type which can compute an output for two given inputs
pub trait Reducer<I1, I2> {
    /// The output of the reduction
    type Output;

    /// Apply the reduction
    fn apply(&self, v1: I1, v2: I2) -> Self::Output;
}

impl<I1, I2, O, F> Reducer<I1, I2> for F
where
    F: Fn(I1, I2) -> O,
{
    type Output = O;

    fn apply(&self, v1: I1, v2: I2) -> Self::Output {
        self(v1, v2)
    }
}

/// Performs a [`Iterator::fold`]-like operation over a tuple list over the provided function [`F`]
/// with data [`T`]
pub trait TupleFold<F, T> {
    /// Output of the fold operation
    type Output;

    /// Perform the fold with [`T`] over [`F`]
    fn fold(self, reducer: F, with: T) -> Self::Output;
}

impl<F, H, T, U> TupleFold<F, U> for (H, T)
where
    T: TupleFold<F, F::Output>,
    F: Reducer<U, H>,
{
    type Output = T::Output;

    fn fold(self, reducer: F, with: U) -> Self::Output {
        let res = reducer.apply(with, self.0);
        self.1.fold(reducer, res)
    }
}

impl<F, U> TupleFold<F, U> for () {
    type Output = U;

    fn fold(self, _reducer: F, with: U) -> Self::Output {
        with
    }
}

/// Compute the domination relationship between this sequence and another comparable sequence
pub trait Dom<Rhs = Self> {
    /// Compute the domination relationship
    fn dominates(&self, other: &Rhs) -> Option<Ordering>;
}

struct DomReducer;

impl Reducer<ControlFlow<(), Ordering>, Option<Ordering>> for DomReducer {
    type Output = ControlFlow<(), Ordering>;

    fn apply(&self, v1: ControlFlow<(), Ordering>, v2: Option<Ordering>) -> Self::Output {
        match v1 {
            ControlFlow::Continue(mut v1) => {
                let ordering = match v2 {
                    None => v1, // no way to compare on v2, so we just keep going
                    Some(mut v2) => {
                        if v2 < v1 {
                            mem::swap(&mut v1, &mut v2);
                        }
                        match (v1, v2) {
                            (Ordering::Less, Ordering::Greater) => return ControlFlow::Break(()),
                            (Ordering::Less, Ordering::Equal | Ordering::Less) => Ordering::Less,
                            (Ordering::Equal, Ordering::Equal) => Ordering::Equal,
                            (Ordering::Equal | Ordering::Greater, Ordering::Greater) => {
                                Ordering::Greater
                            }
                            _ => unreachable!("Unreachable by construction"),
                        }
                    }
                };
                ControlFlow::Continue(ordering)
            }
            b @ ControlFlow::Break(_) => b,
        }
    }
}

// we implement this over tuples to disambiguate and avoid hitting the child rule
impl<H, T1, T2> Dom<T2> for (H, T1)
where
    (H, T1): TuplePartialOrd<T2>,
    <(H, T1) as TuplePartialOrd<T2>>::Output:
        TupleFold<DomReducer, ControlFlow<(), Ordering>, Output = ControlFlow<(), Ordering>>,
{
    fn dominates(&self, other: &T2) -> Option<Ordering> {
        self.partial_cmp_many(other)
            .fold(DomReducer, ControlFlow::Continue(Ordering::Equal))
            .continue_value()
    }
}

impl Dom<()> for () {
    fn dominates(&self, _other: &()) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl<T> Dom<Reverse<T>> for Reverse<T>
where
    T: Dom<T>,
{
    fn dominates(&self, other: &Reverse<T>) -> Option<Ordering> {
        self.0.dominates(&other.0).map(core::cmp::Ordering::reverse)
    }
}

impl<K, V> Dom<BTreeMap<K, V>> for BTreeMap<K, V>
where
    K: Ord,
    V: PartialOrd,
{
    fn dominates(&self, other: &BTreeMap<K, V>) -> Option<Ordering> {
        let first_iter = self.iter();
        let mut second_iter = other.iter();

        let mut first_greater = false;
        let mut second_greater = false;

        let mut maybe_second = second_iter.next();
        for first in first_iter {
            loop {
                if let Some(second) = maybe_second {
                    match first.0.cmp(second.0) {
                        Ordering::Less => {
                            // we need to advance first
                            if second_greater {
                                return None; // neither dominates
                            }
                            first_greater = true;
                            break;
                        }
                        Ordering::Equal => {
                            // compare
                            match first.1.partial_cmp(second.1) {
                                Some(Ordering::Greater) => {
                                    if second_greater {
                                        return None; // neither dominates
                                    }
                                    first_greater = true;
                                }
                                Some(Ordering::Less) => {
                                    if first_greater {
                                        return None; // neither dominates
                                    }
                                    second_greater = true;
                                }
                                None => {
                                    return None; // cannot evaluate, so neither dominates
                                }
                                _ => {}
                            }
                            // advance both
                            maybe_second = second_iter.next();
                            break;
                        }
                        Ordering::Greater => {
                            // we need to advance second
                            if first_greater {
                                return None; // neither dominates
                            }
                            second_greater = true;
                            maybe_second = second_iter.next();
                        }
                    }
                } else if second_greater {
                    // second was greater, but shorter; neither dominates
                    return None;
                } else {
                    // second was not greater and shorter
                    return Some(Ordering::Greater);
                }
            }
        }
        if maybe_second.is_some() {
            if first_greater {
                // first was greater, but shorter; neither dominates
                None
            } else {
                // first was not greater and shorter
                Some(Ordering::Less)
            }
        } else if first_greater {
            // first is greater
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal) // we have scanned all elements; they are equal
        }
    }
}

/// A [newtype] wrapper for implementing [`PartialOrd`] based on [`Dom`]
///
/// [newtype]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html
pub struct DominationOrd<T>(pub T);

impl<T> PartialEq<DominationOrd<T>> for DominationOrd<T>
where
    T: Dom<T>,
{
    fn eq(&self, other: &DominationOrd<T>) -> bool {
        self.0.dominates(&other.0).is_some_and(Ordering::is_eq)
    }
}

impl<T> PartialOrd<DominationOrd<T>> for DominationOrd<T>
where
    T: Dom<T>,
{
    fn partial_cmp(&self, other: &DominationOrd<T>) -> Option<Ordering> {
        self.0.dominates(&other.0)
    }
}

/// A function which evaluates over a given input
pub trait Evaluator<I> {
    /// The output of evaluation
    type Output;

    /// Perform the evaluation
    fn evaluate(&mut self, input: &I) -> Self::Output;
}

impl<I, O, F> Evaluator<I> for F
where
    F: FnMut(&I) -> O,
{
    type Output = O;

    fn evaluate(&mut self, input: &I) -> Self::Output {
        self(input)
    }
}

/// Evaluates a tuple list of [`Evaluator`]s over a given tuple list
pub trait EvaluateTuple<I> {
    /// The tuple of outputs as a result of evaluation
    type Output;

    /// Perform the evaluation
    fn evaluate(&mut self, value: &I) -> Self::Output;
}

impl<T> EvaluateTuple<T> for () {
    type Output = ();

    fn evaluate(&mut self, _value: &T) -> Self::Output {}
}

impl<T, F1, FT> EvaluateTuple<T> for (F1, FT)
where
    FT: EvaluateTuple<T>,
    F1: Evaluator<T>,
{
    type Output = (F1::Output, FT::Output);

    fn evaluate(&mut self, value: &T) -> Self::Output {
        (self.0.evaluate(value), self.1.evaluate(value))
    }
}

/// A [newtype] over a tuple of objectives so that we can implement [`FitnessMeasurer`] over a tuple
/// list of other [`FitnessMeasurer`]s
///
/// [newtype]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html
pub struct Multiobjective<MT> {
    measurers: MT,
}

impl<MT> Multiobjective<MT> {
    /// Create a new [`Multiobjective`] over a tuple of other [`FitnessMeasurer`]s
    pub fn new(measurers: MT) -> Self {
        Self { measurers }
    }
}

/// A tuple list of [`FitnessMeasurer`]s
pub trait FitnessMeasurerTuple<'a, N, E> {
    /// A tuple list of [`FitnessMeasurer::Measurement`]s
    type Measurements;

    /// Perform the evaluation over all measurers

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurements, E>;
}

impl<'a, N, E> FitnessMeasurerTuple<'a, N, E> for () {
    type Measurements = ();

    fn evaluate(&mut self, _node: &'a N) -> Result<Self::Measurements, E> {
        Ok(())
    }
}

impl<'a, N, E, Head, Tail> FitnessMeasurerTuple<'a, N, E> for (Head, Tail)
where
    Head: FitnessMeasurer<'a, N>,
    E: From<Head::Error>,
    Tail: FitnessMeasurerTuple<'a, N, E>,
    N: Node,
{
    type Measurements = (Head::Measurement, Tail::Measurements);

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurements, E> {
        Ok((self.0.evaluate(node)?, self.1.evaluate(node)?))
    }
}

/// Computes the [`CombinedMeasurement`] over a list of measurements by composing their fitnesses
/// and merging the violations lists
pub trait MeasurementsCombined: Sized {
    /// The fitnesses as a result of this computation
    type Fitnesses;

    /// Compute the [`CombinedMeasurement`] by applying [`MeasurementsCombined::combine_step`] and
    /// computing the merge
    fn combined(self) -> CombinedMeasurement<Self::Fitnesses> {
        let (fitnesses, pass_rate, violations) = self.combine_step();
        CombinedMeasurement {
            fitnesses,
            violations: Violations::new(pass_rate, violations),
        }
    }

    /// Iteration step for merging fitnesses, pass rates, and violations
    fn combine_step(self) -> (Self::Fitnesses, Ratio<usize>, Vec<VecDeque<usize>>);
}

impl<M> MeasurementsCombined for (M, ())
where
    M: HasFitness + HasViolations,
{
    type Fitnesses = (M::Fitness, ());

    fn combine_step(mut self) -> (Self::Fitnesses, Ratio<usize>, Vec<VecDeque<usize>>) {
        let fitness = self.0.take_fitness();
        let (pass_rate, violations) = self.0.take_violations().into_raw();
        ((fitness, ()), pass_rate, violations)
    }
}

impl<M, MC> MeasurementsCombined for (M, MC)
where
    M: HasFitness + HasViolations,
    MC: MeasurementsCombined,
{
    type Fitnesses = (M::Fitness, MC::Fitnesses);

    fn combine_step(mut self) -> (Self::Fitnesses, Ratio<usize>, Vec<VecDeque<usize>>) {
        let fitness = self.0.take_fitness();
        let (pass_rate, mut violations) = self.0.take_violations().into_raw();
        let (other_fitness, other_pass, other_violations) = self.1.combine_step();
        let pass_rate = Ratio::new(
            pass_rate.numer() + other_pass.numer(),
            pass_rate.denom() + other_pass.denom(),
        );
        violations.extend(other_violations);
        ((fitness, other_fitness), pass_rate, violations)
    }
}

/// A measurement which represents the combined results of multiple independent [`FitnessMeasurer`]s
pub struct CombinedMeasurement<V> {
    fitnesses: V,
    violations: Violations,
}

impl<V> HasFitness for CombinedMeasurement<V>
where
    V: Default,
{
    type Fitness = V;

    fn fitness(&self) -> &Self::Fitness {
        &self.fitnesses
    }

    fn take_fitness(&mut self) -> Self::Fitness {
        mem::take(&mut self.fitnesses)
    }
}

impl<V> HasViolations for CombinedMeasurement<V> {
    fn violations(&self) -> &Violations {
        &self.violations
    }

    fn take_violations(&mut self) -> Violations {
        mem::take(&mut self.violations)
    }
}

impl<'a, N, MT> FitnessMeasurer<'a, N> for Multiobjective<MT>
where
    MT: FitnessMeasurerTuple<'a, N, Error>,
    MT::Measurements: MeasurementsCombined,
    N: Node,
{
    type Measurement = CombinedMeasurement<<MT::Measurements as MeasurementsCombined>::Fitnesses>;
    type Error = Error;

    fn evaluate(&mut self, node: &'a N) -> Result<Self::Measurement, Self::Error> {
        Ok(self.measurers.evaluate(node)?.combined())
    }
}
