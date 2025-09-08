//! Enumeration strategy presented in
//! ["How to enumerate a context-free grammar"](https://arxiv.org/abs/2305.00522), implemented as
//! a sampler
//!
//! Here, we guarantee property (i) by because the sampler is simply not invoked for anything which
//! expands to only terminals. Property (ii) is guaranteed by precomputing the indices in an
//! alternation which are guaranteed to terminate, and then making the default choice for these
//! nodes the zeroeth choice.
//!
//! Enumeration order differs, but this actually doesn't matter. Since the pairing function
//! guarantees that each sequence of integers is guaranteed to appear, we will necessarily enumerate
//! the tree regardless of whether we generate in DFS or BFS order.
//!
//! The source code here is based loosely on: https://github.com/piantado/enumerateCFG

use crate::dynamic::{DefinitionOf, HasDynamicSampler};
use crate::generation::Sampler;
use crate::graph::{IntoGraph, shortest_path};
use crate::lang::{FandangoNode, Program};
use crate::{impl_definition_of, impl_has_dynamic_sampler};
use hashbrown::HashMap;
use num_integer::{Integer, Roots};
use num_traits::ops::overflowing::{OverflowingMul, OverflowingSub};
use num_traits::{Euclid, One, Unsigned, WrappingShl, Zero};

struct IntegerizedStack<T> {
    value: T,
}

impl<T> IntegerizedStack<T>
where
    T: Unsigned + Integer + Roots + OverflowingMul + OverflowingSub + WrappingShl + Euclid,
{
    fn new(value: T) -> Option<Self> {
        if value.is_zero() {
            None
        } else {
            Some(Self { value })
        }
    }

    /// Rosenberg-Strong decoding function: https://arxiv.org/abs/1706.04129
    fn decode(z: T) -> (T, T) {
        let m = z.sqrt();
        let m2 = m.overflowing_mul(&m).0; // guaranteed, because m<sqrt(z)

        if z.overflowing_sub(&m2).0 < m {
            (z.overflowing_sub(&m2).0, m)
        } else {
            let shl_m = m.wrapping_shl(1);
            (m, m2 + shl_m - z)
        }
    }

    fn mod_decode(z: T, m: T) -> (T, T) {
        let remainder = z.rem_euclid(&m);
        if m.is_zero() {
            (z, T::zero())
        } else {
            (z.overflowing_sub(&remainder).0 / m, remainder)
        }
    }

    fn pop(self) -> (Option<Self>, T) {
        let (value, result) = Self::decode(self.value);
        ((!value.is_zero()).then_some(Self { value }), result)
    }

    fn modpop(self, m: T) -> (Option<Self>, T) {
        let (value, result) = Self::mod_decode(self.value, m);
        ((!value.is_zero()).then_some(Self { value }), result)
    }
}

/// A sampler which maps the integers to derivation trees
///
/// Note that because of how the stack integerization is implemented, this can only be used once.
pub struct EnumerationSampler<S, T> {
    remappings: HashMap<FandangoNode<'static, 'static>, usize>,
    sampler: S,
    stack: Option<IntegerizedStack<T>>,
}

impl<S, T> EnumerationSampler<S, T>
where
    T: Unsigned + Integer + Roots + OverflowingMul + OverflowingSub + WrappingShl + Euclid,
{
    /// Construct an enumeration sampler over the given value
    pub fn new(root: &'static Program<'static>, sampler: S, value: T) -> Self {
        let (_nt, graph) = root.into_graph();
        let sp = shortest_path(&graph);
        let remappings = HashMap::from_iter(sp.into_iter().map(|(k, v)| (k, v[0])));

        let result = Self {
            remappings,
            sampler,
            stack: None,
        };
        result.with_stack(value)
    }

    /// Reset the value for enumeration, required after every usage.
    pub fn with_stack(mut self, value: T) -> Self {
        self.stack = IntegerizedStack::new(value);
        self
    }
}

impl<N, S, T> Sampler<N> for EnumerationSampler<S, T>
where
    S: DefinitionOf<N>,
    T: Integer + Roots + OverflowingMul + OverflowingSub + WrappingShl + Euclid + From<usize>,
    usize: From<T>,
{
    fn sample_kleene(&mut self) -> usize {
        let mut choice = 0;
        self.stack = if let Some(s) = self.stack.take() {
            let (next, value) = s.pop();
            choice = usize::from(value);
            next
        } else {
            None
        };
        choice
    }

    fn sample_plus(&mut self) -> usize {
        let mut choice = 1;
        self.stack = if let Some(s) = self.stack.take() {
            let (next, value) = s.pop();
            choice += usize::from(value);
            next
        } else {
            None
        };
        choice
    }

    fn sample_optional(&mut self) -> bool {
        let mut choice = false;
        self.stack = if let Some(s) = self.stack.take() {
            let (next, value) = s.modpop(T::one());
            choice = value.is_one();
            next
        } else {
            None
        };
        choice
    }

    fn sample_repetition(&mut self, lower: usize, upper: usize) -> usize {
        let mut choice = lower;
        self.stack = if let Some(s) = self.stack.take() {
            let (next, value) = s.modpop(T::from(upper - lower));
            choice += usize::from(value);
            next
        } else {
            None
        };
        choice
    }

    fn sample_alternative(&mut self, count: usize) -> usize {
        let mut choice = 0;
        self.stack = if let Some(s) = self.stack.take() {
            let (next, value) = s.modpop(T::from(count));
            choice = usize::from(value);
            next
        } else {
            None
        };

        // we need to sanitize this choice in case we have exhausted the stack
        // requirement (ii) from https://arxiv.org/abs/2305.00522
        let remapped = self
            .remappings
            .get(&self.definition_of())
            .copied()
            .expect("Must be present.");
        if choice == 0 {
            choice = remapped;
        } else if choice == remapped {
            choice = 0;
        }

        choice
    }

    fn sample(&mut self) -> usize {
        let mut choice = 0;
        self.stack = if let Some(s) = self.stack.take() {
            let (next, value) = s.pop();
            choice = usize::from(value);
            next
        } else {
            None
        };
        choice
    }

    fn reseed(&mut self, _seed: u64) {
        // nothing to do
    }
}

impl<S, T> HasDynamicSampler for EnumerationSampler<S, T>
where
    S: HasDynamicSampler,
{
    impl_has_dynamic_sampler!(sampler);
}

impl<N, S, T> DefinitionOf<N> for EnumerationSampler<S, T>
where
    S: DefinitionOf<N>,
{
    impl_definition_of!(sampler);
}
