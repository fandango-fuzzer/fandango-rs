mod util;

use crate::type_eq::type_eq;
use crate::typing::Discriminable;
use rand::Rng;
use std::mem;

pub trait Sampler<N> {
    fn sample_kleene(&mut self) -> usize;
    fn sample_plus(&mut self) -> usize;
    fn sample_optional(&mut self) -> bool;
    fn sample_repetition(&mut self, lower: usize, upper: usize) -> usize;
    fn sample_alternative(&mut self, count: usize) -> usize;
}

pub const DEFAULT_UPPER_COUNT: usize = 64;

impl<N, R> Sampler<N> for R
where
    R: Rng,
{
    fn sample_kleene(&mut self) -> usize {
        self.gen_range(0..DEFAULT_UPPER_COUNT)
    }

    fn sample_plus(&mut self) -> usize {
        self.gen_range(0..DEFAULT_UPPER_COUNT)
    }

    fn sample_optional(&mut self) -> bool {
        self.r#gen()
    }

    fn sample_repetition(&mut self, lower: usize, upper: usize) -> usize {
        self.gen_range(lower..=upper)
    }

    fn sample_alternative(&mut self, count: usize) -> usize {
        self.gen_range(0..count)
    }
}

pub trait DefaultGenerated<S> {
    fn generate_default(sampler: &mut S) -> Self;
}

impl<N, S> DefaultGenerated<S> for Box<N>
where
    N: DefaultGenerated<S>,
{
    fn generate_default(sampler: &mut S) -> Self {
        Box::new(N::generate_default(sampler))
    }
}

pub trait Generator<N, W, S> {
    fn generate(&mut self, with: &mut W, sampler: &mut S) -> Option<N>;
}

pub trait SpecificGenerator<W, S> {
    type Generated;

    fn generate(&mut self, with: &mut W, sampler: &mut S) -> Option<Self::Generated>;
}

impl<G, N, W, S> Generator<N, W, S> for G
where
    G: SpecificGenerator<W, S>,
    G::Generated: Discriminable,
    N: Discriminable,
    W: GeneratorTuple<G::Generated, N>,
{
    fn generate(&mut self, with: &mut W, sampler: &mut S) -> Option<N> {
        if <G::Generated as Discriminable>::DISCRIMINANT == N::DISCRIMINANT {
            assert!(type_eq::<G::Generated, N>());
            let generated = <Self as SpecificGenerator<W, S>>::generate(self, with, sampler);
            generated.map(|n| {
                // SAFETY: G::Generated == N, and we've checked the discriminants are the same
                // it is possible that the lifetimes are not compatible, but (at least for
                // generated code) there are no non-owned values
                let actual = unsafe { mem::transmute_copy::<G::Generated, N>(&n) };
                mem::forget(n);
                actual
            })
        } else {
            None
        }
    }
}

pub trait GeneratorTuple<N, S> {
    fn generate(&mut self, sampler: &mut S) -> N;
}

impl<Head, Tail, N, S> GeneratorTuple<N, S> for (Head, Tail)
where
    Head: Generator<N, Tail, S>,
    Tail: GeneratorTuple<N, S>,
    N: Discriminable,
{
    fn generate(&mut self, sampler: &mut S) -> N {
        self.0
            .generate(&mut self.1, sampler)
            .unwrap_or_else(|| self.1.generate(sampler))
    }
}

impl<N, S> GeneratorTuple<N, S> for ()
where
    N: DefaultGenerated<S>,
{
    fn generate(&mut self, sampler: &mut S) -> N {
        N::generate_default(sampler)
    }
}
