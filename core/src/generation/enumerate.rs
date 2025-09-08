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
use crate::lang::{FandangoNode, Operator, Program};
use crate::typing::{AsNodeMut, Node};
use crate::visitor::{VisitResult, VisitableChildren, Visitor};
use crate::{impl_definition_of, impl_has_dynamic_sampler};
use core::cmp::Ordering;
use core::convert::Infallible;
use core::fmt::Debug;
use core::ops::{Add, ControlFlow, Div, Mul, Shl, Sub};
use hashbrown::HashMap;
use num_integer::{Integer, Roots};
use num_traits::{Euclid, Unsigned, Zero};

/// Encodes an integer sequence into a single integer using Rosenberg-Strong: <https://arxiv.org/abs/1706.04129>
#[derive(Debug, Copy, Clone)]
pub struct IntegerizedStack<T> {
    value: T,
}

impl<T> IntegerizedStack<T>
where
    T: Unsigned + Zero,
{
    /// Creates a stack, or returns nothing if the value is zero.
    pub fn new(value: T) -> Option<Self> {
        if value.is_zero() {
            None
        } else {
            Some(Self { value })
        }
    }
}
impl<T> IntegerizedStack<T>
where
    T: Unsigned
        + Integer
        + Roots
        + Mul<Output = T>
        + Sub<Output = T>
        + Div<Output = T>
        + Shl<u16, Output = T>
        + Euclid
        + Clone,
{
    /// Decodes an integer with Rosenberg-Strong: https://arxiv.org/abs/1706.04129
    pub fn decode(z: T) -> (T, T) {
        let m = z.sqrt();
        let m2 = m.clone() * m.clone(); // guaranteed, because m<sqrt(z)

        let zm2 = z.clone() - m2.clone();
        if zm2 < m {
            (zm2, m)
        } else {
            let shl_m = m.clone() << 1;
            (m, m2 + shl_m - z)
        }
    }

    /// Decodes a pair of integers as remainder and quotient
    pub fn mod_decode(z: T, m: T) -> (T, T) {
        if m.is_zero() {
            (z.clone(), T::zero())
        } else {
            let remainder = z.rem_euclid(&m);
            ((z - remainder.clone()) / m, remainder)
        }
    }

    /// Pops a value from this stack using [`Self::decode`]
    pub fn pop(self) -> (Option<Self>, T) {
        let (value, result) = Self::decode(self.value);
        ((!value.is_zero()).then_some(Self { value }), result)
    }

    /// Pops a value from this stack using [`Self::mod_decode`]
    pub fn modpop(self, m: T) -> (Option<Self>, T) {
        let (value, result) = Self::mod_decode(self.value, m);
        ((!value.is_zero()).then_some(Self { value }), result)
    }
}

impl<T> IntegerizedStack<T>
where
    T: Unsigned + Integer + Mul<Output = T> + Sub<Output = T> + Add<Output = T> + Clone,
{
    /// Encodes an integer with Rosenberg-Strong: https://arxiv.org/abs/1706.04129
    pub fn encode(x: T, y: T) -> T {
        let max = match x.cmp(&y) {
            Ordering::Less | Ordering::Equal => y.clone(),
            Ordering::Greater => x.clone(),
        };
        max.clone() * max.clone() + max - y + x
    }

    /// Encodes an integer with remainder and quotient
    pub fn mod_encode(y: T, x: T, m: T) -> T {
        m * y + x
    }

    /// Pushes a value to the stack using [`Self::encode`]
    pub fn push(mut self, y: T) -> Self {
        self.value = Self::encode(self.value, y);
        self
    }

    /// Pushes a value to the stack using [`Self::mod_encode`]
    pub fn modpush(mut self, y: T, m: T) -> Self {
        self.value = Self::mod_encode(self.value, y, m);
        self
    }
}

/// A sampler which maps the integers to derivation trees
///
/// Note that because of how the stack integerization is implemented, this can only be used once.
///
/// In addition to being able to produce a tree from a given integer, it can also produce an integer
/// from a given tree by visitation. Use [`Visitor::visit`] to do so.
pub struct EnumerationSampler<S, T> {
    remappings: HashMap<FandangoNode<'static, 'static>, usize>,
    sampler: S,
    stack: Option<IntegerizedStack<T>>,
}

impl<S, T> EnumerationSampler<S, T>
where
    T: Unsigned
        + Integer
        + Roots
        + Mul<Output = T>
        + Sub<Output = T>
        + Shl<u16, Output = T>
        + Euclid,
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

    /// Gets the current integral value of the stack
    pub fn stack(&self) -> Option<&T> {
        if let Some(stack) = self.stack.as_ref() {
            Some(&stack.value)
        } else {
            None
        }
    }

    /// Reset the value for enumeration, required after every usage
    pub fn with_stack(mut self, value: T) -> Self {
        self.stack = IntegerizedStack::new(value);
        self
    }
}

impl<N, S, T> Sampler<N> for EnumerationSampler<S, T>
where
    S: DefinitionOf<N>,
    T: Unsigned
        + Integer
        + Roots
        + Mul<Output = T>
        + Sub<Output = T>
        + Div<Output = T>
        + Shl<u16, Output = T>
        + Euclid
        + Clone
        + From<usize>,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    fn sample_kleene(&mut self) -> usize {
        let mut choice = 0;
        self.stack = if let Some(s) = self.stack.take() {
            let (next, value) = s.pop();
            choice = usize::try_from(value)
                .expect("If this is too large, then we have no chance of expanding this anyways");
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
            choice += usize::try_from(value)
                .expect("If this is too large, then we have no chance of expanding this anyways");
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
            choice += usize::try_from(value)
                .expect("If this is too large, then we have no chance of expanding this anyways");
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
            choice = usize::try_from(value)
                .expect("If this is too large, then we have no chance of expanding this anyways");
            next
        } else {
            None
        };

        // we need to sanitize this choice in case we have exhausted the stack
        // requirement (ii) from https://arxiv.org/abs/2305.00522
        let remapped = self.remappings[&self.definition_of()];
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
            choice = usize::try_from(value)
                .expect("If this is too large, then we have no chance of expanding this anyways");
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

impl<S, T, V> Visitor<T> for EnumerationSampler<S, V>
where
    T: VisitableChildren<T>,
    V: Unsigned
        + Integer
        + Mul<Output = V>
        + Sub<Output = V>
        + Add<Output = V>
        + Clone
        + From<usize>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = (Option<IntegerizedStack<V>>, V, Option<V>);

    fn visit<'program, N>(self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        struct WithChildIndex<S, V> {
            last_idx: usize,
            count: usize,
            sampler: EnumerationSampler<S, V>,
        }

        impl<S, T, V> Visitor<T> for WithChildIndex<S, V>
        where
            T: VisitableChildren<T>,
            V: Unsigned
                + Integer
                + Mul<Output = V>
                + Sub<Output = V>
                + Add<Output = V>
                + Clone
                + From<usize>,
        {
            type Continue = Self;
            type Break = Infallible;
            type Error = (Option<IntegerizedStack<V>>, V, Option<V>);

            fn visit<'program, N>(
                mut self,
                node: &'program mut N,
                idx: usize,
            ) -> VisitResult<Self, T>
            where
                N: Node<TypeMut<'program> = T>,
                T: From<&'program mut N> + AsNodeMut<N>,
            {
                let definition = node.definition();
                let mut res: Self = T::from(node)
                    .visit_each_reverse(WithChildIndex {
                        last_idx: 0,
                        count: 0,
                        sampler: self.sampler,
                    })?
                    .continue_value()
                    .unwrap();

                self.last_idx = idx;
                self.count += 1;
                self.sampler = res.sampler;

                let (v, m) = match definition {
                    FandangoNode::Alternative(alt) => {
                        let remapped = self.sampler.remappings[&FandangoNode::Alternative(alt)];
                        if res.last_idx == 0 {
                            res.last_idx = remapped;
                        } else if res.last_idx == remapped {
                            res.last_idx = 0;
                        }
                        (
                            V::from(res.last_idx),
                            Some(V::from(alt.concatenations().len())),
                        )
                    }
                    FandangoNode::Operator(op) => (
                        V::from(res.count),
                        match op {
                            Operator::Kleene(_) | Operator::Plus(_) => None,
                            Operator::Option(_) => Some(V::one()),
                            &Operator::Repeat(_, lower, upper) => Some(V::from(upper - lower)),
                            Operator::Symbol(_) => {
                                unreachable!("Not represented in complete grammars")
                            }
                        },
                    ),
                    _ => return Ok(ControlFlow::Continue(self)),
                };
                if let Some(modulus) = m {
                    self.sampler.stack = if let Some(stack) = self.sampler.stack {
                        Some(stack.modpush(v, modulus))
                    } else {
                        let encoded =
                            IntegerizedStack::mod_encode(V::zero(), v.clone(), modulus.clone());
                        IntegerizedStack::new(encoded) // could be none if we encoded idx == 0
                    };
                } else {
                    self.sampler.stack = if let Some(stack) = self.sampler.stack {
                        Some(stack.push(v))
                    } else {
                        let encoded = IntegerizedStack::encode(V::zero(), v);
                        IntegerizedStack::new(encoded) // could be none if we encoded idx == 0
                    };
                }

                Ok(ControlFlow::Continue(self))
            }
        }

        let visitor = WithChildIndex {
            last_idx: 0,
            count: 0,
            sampler: self,
        };

        Ok(ControlFlow::Continue(
            visitor.visit(node, idx)?.continue_value().unwrap().sampler,
        ))
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

#[cfg(test)]
mod test {
    use crate::generation::enumerate::IntegerizedStack;

    #[test]
    fn simple_encoding() {
        let seq = [1usize, 0, 1, 2, 3, 4, 5];
        let initial = IntegerizedStack::encode(0usize, seq[0]);
        let mut stack = IntegerizedStack::new(initial).unwrap();
        for &i in &seq[1..] {
            stack = stack.push(i);
        }
        let mut stack = Some(stack);
        for i in seq.into_iter().rev() {
            let (next_stack, retrieved) = stack.unwrap().pop();
            stack = next_stack;
            assert_eq!(retrieved, i);
        }
        assert!(stack.is_none());
    }

    #[test]
    fn mod_encoding() {
        let seq = [1usize, 0, 1, 2, 3, 4, 5];
        let initial = IntegerizedStack::mod_encode(0usize, seq[0], 12);
        let mut stack = IntegerizedStack::new(initial).unwrap();
        for &i in &seq[1..] {
            stack = stack.modpush(i, 12);
        }
        let mut stack = Some(stack);
        for i in seq.into_iter().rev() {
            let (next_stack, retrieved) = stack.unwrap().modpop(12);
            stack = next_stack;
            assert_eq!(retrieved, i);
        }
        assert!(stack.is_none());
    }
}
