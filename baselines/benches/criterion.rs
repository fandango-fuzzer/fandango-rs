//! The actual benchmarks against csv, rest, scriptsizec, and xml.

#![expect(deprecated)]
#![expect(missing_docs)]
// we only work on 64-bit platforms; casting u64->usize is fine
#![allow(clippy::cast_possible_truncation)]

use common::{BenchmarkSuite, StdGenerator, StdSampler};
use criterion::{
    BatchSize, BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use fandango::dynamic::{DynamicNode, DynamicSampler};
use fandango::generation::{Generated, InPlaceGenerated};
use fandango::lang::FandangoNode;
use fandango::tuple_list::{tuple_list, tuple_list_type};
use fandango::typing::{AsNode, AsStaticNode, Node};
use fandango::visitor::Visitor;
use fandango::visitor::navigation::{Advance, CountNodes, GoToMut};
use fandango::visitor::write::WriteVisitor;
use fandango_runtime::operators::{DepthLimiter, NonterminalVisitor, crossover};
use hashbrown::HashMap;
use rand::prelude::{IndexedRandom, StdRng};
use rand::{RngCore, SeedableRng};
use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Duration;

/// The number of segments to split the available samples into.
pub const NUM_SEGMENTS: usize = 25;

/// Do the benchmark! Set `B` to your desired baseline.
#[allow(clippy::too_many_lines)]
fn perform_benchmark<B>(c: &mut Criterion)
where
    B: BenchmarkSuite<StdSampler, StdGenerator>,
    // boilerplate since we're doing this generically
    B::Start: Node + Clone + Ord + AsStaticNode,
    for<'a> <B::Start as Node>::TypeMut<'a>: InPlaceGenerated<
            StdRng,
            tuple_list_type!(DepthLimiter<HashMap<FandangoNode<'static, 'static>, Vec<usize>>>),
        >,
{
    let mut group = c.benchmark_group(B::NAME);
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(900));

    // FANDANGO originally uses a depth limiter with depth 100.
    let mut generator = tuple_list!(DepthLimiter::new(B::program(), 100));
    let mut setup_generator = generator.clone();

    let nonterminals = B::program().nonterminals();

    let mut global = StdSampler::seed_from_u64(0);

    // collect seeds which produce inputs of a given size
    let mut rngs = BTreeMap::new();
    for seed in 0..1_000 {
        let mut rng = StdSampler::seed_from_u64(seed);
        let mut dyn_rng = StdSampler::seed_from_u64(seed);
        let mut dyn_rng = DynamicSampler::new(
            <B::Start as AsStaticNode>::static_root(),
            <B::Start as AsStaticNode>::static_definition(),
            &nonterminals,
            &mut dyn_rng,
        );

        let value = B::generate(&mut rng, &mut generator);
        let size = NonterminalVisitor::default()
            .visit(&value, 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .count();

        let dyn_value = DynamicNode::generate(&mut dyn_rng, &mut generator, 0);
        assert_eq!(
            size,
            NonterminalVisitor::default()
                .visit(&dyn_value, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .count()
        );

        let first = WriteVisitor::new(Vec::new())
            .visit(&value, 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .output();
        let second = WriteVisitor::new(Vec::new())
            .visit(&dyn_value, 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .output();
        assert_eq!(first, second);

        rngs.entry(size).or_insert_with(Vec::new).push(seed);
    }

    let count = rngs.len() - 1;
    let rngs = Vec::from_iter(rngs);

    let grouped = rngs.chunks(count / NUM_SEGMENTS).map(|chunk| {
        let (sum, combined) =
            chunk
                .iter()
                .fold((0, Vec::new()), |(sum, mut combined), (size, seq)| {
                    combined.extend(seq.iter().copied());
                    (sum + *size * seq.len(), combined)
                });
        (sum / combined.len(), combined)
    });

    for (size, seeds) in grouped {
        group.throughput(Throughput::Elements(size as u64));

        // raw generation
        group.bench_function(BenchmarkId::new("generate", size), |b| {
            b.iter_batched_ref(
                || StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                |sampler| B::generate(black_box(sampler), &mut generator),
                BatchSize::SmallInput,
            );
        });

        // raw generation
        group.bench_function(BenchmarkId::new("generate dynamic", size), |b| {
            b.iter_batched_ref(
                || StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                |sampler| {
                    DynamicNode::generate(
                        &mut DynamicSampler::new(
                            <B::Start as AsStaticNode>::static_root(),
                            <B::Start as AsStaticNode>::static_definition(),
                            &nonterminals,
                            black_box(sampler),
                        ),
                        &mut generator,
                        0,
                    )
                },
                BatchSize::SmallInput,
            );
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
                |(value, local)| B::fix(black_box(value), black_box(local), &mut generator),
                BatchSize::SmallInput,
            );
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
            );
        });

        // mutate a generated input
        // the generator and sampler are unconstrained for this operation
        group.bench_function(BenchmarkId::new("mutate", size), |b| {
            b.iter_batched(
                || {
                    let sample = B::generate(
                        &mut StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                        &mut setup_generator,
                    );

                    let count = sample.count_nodes();
                    let choice = Advance::forward(global.next_u64() as usize % count)
                        .visit(&sample, 0)
                        .unwrap()
                        .break_value()
                        .unwrap();

                    (sample, choice, global.clone())
                },
                |(mut value, mut choice, mut local)| {
                    let (&idx, choice) = choice.make_contiguous().split_first().unwrap();
                    let depth = choice.len();
                    let mut mutated = value.go_to_mut(idx, choice).unwrap();
                    mutated.generate_in_place(black_box(&mut local), &mut generator, depth);
                },
                BatchSize::SmallInput,
            );
        });

        // mutate a generated input
        // the generator and sampler are unconstrained for this operation
        group.bench_function(BenchmarkId::new("mutate dynamic", size), |b| {
            b.iter_batched(
                || {
                    let seed = seeds.choose(&mut global).copied().unwrap();

                    let sample = DynamicNode::generate(
                        &mut DynamicSampler::new(
                            <B::Start as AsStaticNode>::static_root(),
                            <B::Start as AsStaticNode>::static_definition(),
                            &nonterminals,
                            &mut StdSampler::seed_from_u64(seed),
                        ),
                        &mut setup_generator,
                        0,
                    );

                    let count = sample.count_nodes();
                    let choice = Advance::forward(global.next_u64() as usize % count)
                        .visit(&sample, 0)
                        .unwrap()
                        .break_value()
                        .unwrap();

                    (sample, choice, global.clone())
                },
                |(mut value, mut choice, mut local)| {
                    let (&idx, choice) = choice.make_contiguous().split_first().unwrap();
                    let depth = choice.len();
                    let mutated = value.go_to_mut(idx, choice).unwrap();
                    let mut sampler = DynamicSampler::new(
                        mutated.root(),
                        mutated.definition(),
                        &nonterminals,
                        black_box(&mut local),
                    );
                    mutated.generate_in_place(&mut sampler, &mut generator, depth);
                },
                BatchSize::SmallInput,
            );
        });

        // crossover a generated input using another sampled input
        // the sampler and base input are unconstrained for this operation
        group.bench_function(BenchmarkId::new("crossover", size), |b| {
            b.iter_batched(
                || {
                    let sample = B::generate(
                        &mut StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                        &mut setup_generator,
                    );

                    let count = sample.count_nodes();
                    let choice = Advance::forward(global.next_u64() as usize % count)
                        .visit(&sample, 0)
                        .unwrap()
                        .break_value()
                        .unwrap();

                    let other = B::generate(&mut global, &mut setup_generator);

                    (sample, other, choice, global.clone())
                },
                |(mut value, base, mut choice, mut local)| {
                    let (&idx, choice) = choice.make_contiguous().split_first().unwrap();
                    assert_eq!(idx, 0);
                    let mut value = value.go_to_mut(0, choice).expect("Must be a valid path");

                    crossover(
                        black_box(&mut value),
                        black_box(&base),
                        black_box(&mut local),
                    )
                },
                BatchSize::SmallInput,
            );
        });

        // crossover a generated input using another sampled input
        // the sampler and base input are unconstrained for this operation
        group.bench_function(BenchmarkId::new("crossover dynamic", size), |b| {
            b.iter_batched(
                || {
                    let seed = seeds.choose(&mut global).copied().unwrap();
                    let sample = DynamicNode::generate(
                        &mut DynamicSampler::new(
                            <B::Start as AsStaticNode>::static_root(),
                            <B::Start as AsStaticNode>::static_definition(),
                            &nonterminals,
                            &mut StdSampler::seed_from_u64(seed),
                        ),
                        &mut setup_generator,
                        0,
                    );

                    let count = sample.count_nodes();
                    let choice = Advance::forward(global.next_u64() as usize % count)
                        .visit(&sample, 0)
                        .unwrap()
                        .break_value()
                        .unwrap();

                    let other = DynamicNode::generate(
                        &mut DynamicSampler::new(
                            <B::Start as AsStaticNode>::static_root(),
                            <B::Start as AsStaticNode>::static_definition(),
                            &nonterminals,
                            &mut global,
                        ),
                        &mut setup_generator,
                        0,
                    );

                    (sample, other, choice, global.clone())
                },
                |(mut value, base, mut choice, mut local)| {
                    let (&idx, choice) = choice.make_contiguous().split_first().unwrap();
                    assert_eq!(idx, 0);
                    let mut value = value.go_to_mut(0, choice).expect("Must be a valid path");

                    #[allow(clippy::mut_mut)] // clippy overdetection
                    crossover(&mut value, &base, &mut local)
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(
    benches,
    perform_benchmark::<csv::Benchmark>,
    perform_benchmark::<rest::Benchmark>,
    perform_benchmark::<scriptsizec::Benchmark>,
    perform_benchmark::<xml::Benchmark>,
);
criterion_main!(benches);
