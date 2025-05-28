//! Generic baseline benchmark runners against FANDANGO.

#![no_std]

extern crate alloc;

/// The number of segments to split the available samples into.
pub const NUM_SEGMENTS: usize = 25;

#[cfg(feature = "static_defs")]
mod defs {
    use crate::NUM_SEGMENTS;
    use alloc::collections::BTreeMap;
    use alloc::vec::Vec;
    use common::{BenchmarkSuite, StdGenerator, StdSampler};
    use core::convert::Infallible;
    use core::hint::black_box;
    use core::time::Duration;
    use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
    use fandango::dynamic::{DynamicNode, DynamicSampler};
    use fandango::generation::Generated;
    use fandango::tuple_list::tuple_list;
    use fandango::typing::{AsNodeMut, AsStaticNode, Node};
    use fandango::visitor::write::WriteVisitor;
    use fandango::visitor::{VisitableChildren, Visitor};
    use fandango_targets::operators::{DepthLimiter, NonterminalVisitor, mutate_dynamic};
    use rand::SeedableRng;
    use rand::seq::IndexedRandom;

    /// Do the benchmark! Set `B` to your desired baseline.
    pub fn perform_benchmark<B>(c: &mut Criterion)
    where
        B: BenchmarkSuite<StdSampler, StdGenerator>,
        B::Start: Node + Clone + Ord + AsStaticNode,
        // boilerplate since we're doing this generically
        for<'a> NonterminalVisitor: Visitor<
                <B::Start as Node>::TypeMut<'a>,
                Continue = NonterminalVisitor,
                Break = Infallible,
                Error = Infallible,
            > + Visitor<
                &'a mut DynamicNode,
                Continue = NonterminalVisitor,
                Break = Infallible,
                Error = Infallible,
            >,
        for<'a> <B::Start as Node>::TypeMut<'a>: VisitableChildren<<B::Start as Node>::TypeMut<'a>>
            + AsNodeMut<B::Start>
            + From<&'a mut B::Start>,
    {
        let mut group = c.benchmark_group(B::NAME);
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

            let mut value = B::generate(&mut rng, &mut generator);
            let size = NonterminalVisitor::default()
                .visit(&mut value, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .count();

            let mut dyn_value = DynamicNode::generate(&mut dyn_rng, &mut generator, 0);
            assert_eq!(
                size,
                NonterminalVisitor::default()
                    .visit(&mut dyn_value, 0)
                    .unwrap()
                    .continue_value()
                    .unwrap()
                    .count()
            );

            let first = WriteVisitor::new(Vec::new())
                .visit(&mut value, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .output();
            let second = WriteVisitor::new(Vec::new())
                .visit(&mut dyn_value, 0)
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
                )
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
                )
            });

            // fixing a generated input
            group.bench_function(BenchmarkId::new("fix", size), |b| {
                b.iter_batched_ref(
                    || {
                        let sample = B::generate(
                            &mut StdSampler::seed_from_u64(
                                seeds.choose(&mut global).copied().unwrap(),
                            ),
                            &mut setup_generator,
                        );
                        (sample, global.clone())
                    },
                    |(value, local)| B::fix(black_box(value), black_box(local), &mut generator),
                    BatchSize::SmallInput,
                )
            });

            // checking the correctness of a generated input
            group.bench_function(BenchmarkId::new("check", size), |b| {
                b.iter_batched_ref(
                    || {
                        B::generate(
                            &mut StdSampler::seed_from_u64(
                                seeds.choose(&mut global).copied().unwrap(),
                            ),
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
                            &mut StdSampler::seed_from_u64(
                                seeds.choose(&mut global).copied().unwrap(),
                            ),
                            &mut setup_generator,
                        );
                        let choices = B::check(&mut sample);
                        (sample, choices, global.clone())
                    },
                    |(value, choices, local)| {
                        B::mutate(
                            black_box(value),
                            black_box(choices),
                            black_box(local),
                            &mut generator,
                        )
                    },
                    BatchSize::SmallInput,
                )
            });

            // mutate a generated input
            // the generator and sampler are unconstrained for this operation
            group.bench_function(BenchmarkId::new("mutate dynamic", size), |b| {
                b.iter_batched_ref(
                    || {
                        let seed = seeds.choose(&mut global).copied().unwrap();
                        let mut sample =
                            B::generate(&mut StdSampler::seed_from_u64(seed), &mut setup_generator);
                        let choices = B::check(&mut sample);

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
                        (sample, choices, global.clone())
                    },
                    |(value, choices, local)| {
                        mutate_dynamic(
                            black_box(value),
                            black_box(choices),
                            &mut DynamicSampler::new(
                                <B::Start as AsStaticNode>::static_root(),
                                <B::Start as AsStaticNode>::static_definition(),
                                &nonterminals,
                                black_box(local),
                            ),
                            &mut generator,
                        )
                        .unwrap()
                        .is_some()
                    },
                    BatchSize::SmallInput,
                )
            });

            // crossover a generated input using another sampled input
            // the sampler and base input are unconstrained for this operation
            group.bench_function(BenchmarkId::new("crossover", size), |b| {
                b.iter_batched_ref(
                    || {
                        let mut sample = B::generate(
                            &mut StdSampler::seed_from_u64(
                                seeds.choose(&mut global).copied().unwrap(),
                            ),
                            &mut setup_generator,
                        );
                        let choices = B::check(&mut sample);

                        let other = B::generate(&mut global, &mut setup_generator);

                        (sample, other, choices, global.clone())
                    },
                    |(value, base, choices, local)| {
                        B::crossover(
                            black_box(value),
                            black_box(base),
                            black_box(choices),
                            black_box(local),
                        )
                    },
                    BatchSize::SmallInput,
                )
            });

            // crossover a generated input using another sampled input
            // the sampler and base input are unconstrained for this operation
            group.bench_function(BenchmarkId::new("crossover dynamic", size), |b| {
                b.iter_batched_ref(
                    || {
                        let seed = seeds.choose(&mut global).copied().unwrap();
                        let mut sample =
                            B::generate(&mut StdSampler::seed_from_u64(seed), &mut setup_generator);
                        let choices = B::check(&mut sample);

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

                        (sample, other, choices, global.clone())
                    },
                    |(value, base, choices, local)| {
                        B::crossover_dynamic(
                            black_box(value),
                            black_box(base),
                            black_box(choices),
                            &mut DynamicSampler::new(
                                <B::Start as AsStaticNode>::static_root(),
                                <B::Start as AsStaticNode>::static_definition(),
                                &nonterminals,
                                black_box(local),
                            ),
                        )
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }
}

#[cfg(not(feature = "static_defs"))]
mod defs {
    use crate::NUM_SEGMENTS;
    use alloc::collections::BTreeMap;
    use alloc::vec::Vec;
    use common::{DynamicBenchmarkSuite, StdSampler};
    use core::convert::Infallible;
    use core::hint::black_box;
    use core::time::Duration;
    use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
    use fandango::dynamic::{DynamicNode, DynamicSampler};
    use fandango::generation::Generated;
    use fandango::lang::FandangoNode;
    use fandango::tuple_list::tuple_list;
    use fandango::visitor::Visitor;
    use fandango_targets::operators::{DepthLimiter, NonterminalVisitor};
    use rand::SeedableRng;
    use rand::seq::IndexedRandom;

    /// Do the benchmark! Set `B` to your desired baseline.
    pub fn perform_benchmark<B>(c: &mut Criterion)
    where
        B: DynamicBenchmarkSuite,
        // boilerplate since we're doing this generically
        for<'a> NonterminalVisitor: Visitor<
                &'a mut DynamicNode,
                Continue = NonterminalVisitor,
                Break = Infallible,
                Error = Infallible,
            >,
    {
        let mut group = c.benchmark_group(B::NAME);
        group.warm_up_time(Duration::from_millis(100));
        group.measurement_time(Duration::from_millis(900));

        // FANDANGO originally uses a depth limiter with depth 100.
        let mut generator = tuple_list!(DepthLimiter::new(B::program(), 100));

        let nonterminals = B::program().nonterminals();
        let nonterminal_start = nonterminals
            .keys()
            .find_map(|k| match k {
                FandangoNode::Nonterminal(nt) => (nt.name() == "start").then_some(*k),
                _ => None,
            })
            .unwrap();

        let mut global = StdSampler::seed_from_u64(0);

        // collect seeds which produce inputs of a given size
        let mut rngs = BTreeMap::new();
        for seed in 0..1_000 {
            let mut dyn_rng = StdSampler::seed_from_u64(seed);
            let mut dyn_rng = DynamicSampler::new(
                FandangoNode::Program(B::program()),
                nonterminal_start,
                &nonterminals,
                &mut dyn_rng,
            );

            let mut dyn_value = DynamicNode::generate(&mut dyn_rng, &mut generator, 0);

            let size = NonterminalVisitor::default()
                .visit(&mut dyn_value, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .count();

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
            group.bench_function(BenchmarkId::new("generate dynamic", size), |b| {
                b.iter_batched_ref(
                    || StdSampler::seed_from_u64(seeds.choose(&mut global).copied().unwrap()),
                    |sampler| {
                        DynamicNode::generate(
                            &mut DynamicSampler::new(
                                FandangoNode::Program(B::program()),
                                nonterminal_start,
                                &nonterminals,
                                black_box(sampler),
                            ),
                            &mut generator,
                            0,
                        )
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }
}

pub use defs::*;
