#![no_std]

extern crate alloc;

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use core::hint::black_box;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
use fandango::lang::{FandangoNode, Program};
use fandango::tuple_list::{tuple_list, tuple_list_type};
use fandango::visitor::navigation::{CountNodes, CountNodesWith};
use fandango_eval::operators::DepthLimiter;
use hashbrown::HashMap;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

pub trait BenchmarkSuite<S, G> {
    type Start;

    const NAME: &'static str;

    fn generate(sampler: &mut S, generator: &mut G) -> Self::Start;

    fn fix(item: &mut Self::Start, sampler: &mut S, generator: &mut G);

    fn check(item: &mut Self::Start) -> Vec<VecDeque<usize>>;

    fn mutate(
        item: &mut Self::Start,
        choices: &mut Vec<VecDeque<usize>>,
        sampler: &mut S,
        generator: &mut G,
    ) -> bool;

    fn crossover(
        item: &mut Self::Start,
        other: &mut Self::Start,
        choices: &mut Vec<VecDeque<usize>>,
        sampler: &mut S,
    ) -> bool;

    fn program() -> &'static Program<'static>;
}

pub type StdSampler = StdRng;
pub type StdGenerator =
    tuple_list_type!(DepthLimiter<HashMap<FandangoNode<'static, 'static>, Vec<usize>>>);

pub fn perform_benchmark<B>(c: &mut Criterion)
where
    B: BenchmarkSuite<StdSampler, StdGenerator>,
    for<'a> B::Start: Clone + CountNodes<'a> + Ord,
{
    let mut group = c.benchmark_group(B::NAME);
    let mut generator = tuple_list!(DepthLimiter::new(B::program(), 100));
    let mut setup_generator = generator.clone();

    let mut global = StdRng::seed_from_u64(0);

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

        group.bench_function(BenchmarkId::new("generate", size), |b| {
            b.iter_batched_ref(
                || StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                |sampler| B::generate(black_box(sampler), &mut generator),
                BatchSize::SmallInput,
            )
        });

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

        group.bench_function(BenchmarkId::new("crossover", size), |b| {
            b.iter_batched_ref(
                || {
                    let mut sample = B::generate(
                        &mut StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                        &mut setup_generator,
                    );
                    let mut base = B::generate(&mut global, &mut setup_generator);
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
