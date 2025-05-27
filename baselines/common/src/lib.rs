//! Common definitions for benchmarking the baselines.

#![no_std]

extern crate alloc;

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use core::hint::black_box;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
use fandango::lang::{FandangoNode, Program};
use fandango::tuple_list::{tuple_list, tuple_list_type};
use fandango::visitor::navigation::CountNodes;
use fandango_targets::operators::DepthLimiter;
use hashbrown::HashMap;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

/// Trait which each benchmark target will implement, for consistency.
pub trait BenchmarkSuite<S, G> {
    /// The start node type (often, `nonterminal_start`) used by the benchmark.
    type Start;

    /// The name to associate with this benchmark.
    const NAME: &'static str;

    /// Generate a start node.
    fn generate(sampler: &mut S, generator: &mut G) -> Self::Start;

    /// Fix a given start node.
    ///
    /// Sampler and generator is provided to allow for mutation-based fixing.
    fn fix(item: &mut Self::Start, sampler: &mut S, generator: &mut G);

    /// Check a given start node's constraints and return any violations as paths.
    fn check(item: &mut Self::Start) -> Vec<VecDeque<usize>>;

    /// Mutate the given start node at the given points, where possible.
    fn mutate(
        item: &mut Self::Start,
        choices: &mut Vec<VecDeque<usize>>,
        sampler: &mut S,
        generator: &mut G,
    ) -> bool;

    /// Crossover the given start node at the given points with the provided base.
    fn crossover(
        item: &mut Self::Start,
        other: &mut Self::Start,
        choices: &mut Vec<VecDeque<usize>>,
        sampler: &mut S,
    ) -> bool;

    /// The static [`Program`] node which represents the grammar.
    fn program() -> &'static Program<'static>;
}

/// The sampler to be used throughout the evaluation.
pub type StdSampler = StdRng;
/// The generator to be used throughout the evaluation.
pub type StdGenerator =
    tuple_list_type!(DepthLimiter<HashMap<FandangoNode<'static, 'static>, Vec<usize>>>);

/// Do the benchmark! Set `B` to your desired baseline.
pub fn perform_benchmark<B>(c: &mut Criterion)
where
    B: BenchmarkSuite<StdSampler, StdGenerator>,
    for<'a> B::Start: Clone + CountNodes<'a> + Ord,
{
    let mut group = c.benchmark_group(B::NAME);
    // FANDANGO originally uses a depth limiter with depth 100.
    let mut generator = tuple_list!(DepthLimiter::new(B::program(), 100));
    let mut setup_generator = generator.clone();

    let mut global = StdRng::seed_from_u64(0);

    // collect seeds which produce inputs of a given size
    let mut rngs = BTreeMap::new();
    for seed in 0..100_000 {
        let mut rng = StdSampler::seed_from_u64(seed);

        let mut value = B::generate(&mut rng, &mut generator);
        let size = value.count_nodes();
        rngs.entry(size).or_insert_with(Vec::new).push(seed);
    }

    let count = rngs.len() - 1;
    let rngs = rngs.into_iter().step_by(count / 10).collect::<Vec<_>>();

    for (size, seeds) in rngs {
        group.throughput(Throughput::Elements(size as u64));

        // raw generation
        group.bench_function(BenchmarkId::new("generate", size), |b| {
            b.iter_batched_ref(
                || StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                |sampler| B::generate(black_box(sampler), &mut generator),
                BatchSize::SmallInput,
            )
        });

        // fixing a generated input
        group.bench_function(BenchmarkId::new("fix", size), |b| {
            b.iter_batched_ref(
                || {
                    let sample = B::generate(
                        &mut StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                        &mut setup_generator,
                    );
                    (sample, global.clone())
                },
                |(value, local)| B::fix(black_box(value), local, &mut generator),
                BatchSize::SmallInput,
            )
        });

        // checking the correctness of a generated input
        group.bench_function(BenchmarkId::new("check", size), |b| {
            b.iter_batched_ref(
                || {
                    B::generate(
                        &mut StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                        &mut setup_generator,
                    )
                },
                |value| B::check(black_box(value)),
                BatchSize::SmallInput,
            )
        });

        // mutate a generated input
        // the generator and sampler are unconstrained for this operation
        group.bench_function(BenchmarkId::new("mutate", size), |b| {
            b.iter_batched_ref(
                || {
                    let mut sample = B::generate(
                        &mut StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                        &mut setup_generator,
                    );
                    B::fix(&mut sample, &mut global, &mut setup_generator);
                    let choices = B::check(&mut sample);
                    (sample, choices, global.clone())
                },
                |(value, choices, local)| B::mutate(value, choices, local, &mut generator),
                BatchSize::SmallInput,
            )
        });

        // mutate a generated input
        // the crossover source and sampler are unconstrained for this operation
        group.bench_function(BenchmarkId::new("crossover", size), |b| {
            b.iter_batched_ref(
                || {
                    let mut sample = B::generate(
                        &mut StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                        &mut setup_generator,
                    );
                    let base = B::generate(&mut global, &mut setup_generator);
                    B::fix(&mut sample, &mut global, &mut setup_generator);
                    let choices = B::check(&mut sample);
                    (sample, base, choices, global.clone())
                },
                |(value, base, choices, local)| B::crossover(value, base, choices, local),
                BatchSize::SmallInput,
            )
        });
    }
}
