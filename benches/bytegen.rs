#![allow(missing_docs)]

extern crate alloc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::{RngCore, SeedableRng};
use std::fs::File;
use std::io::Read;

fn urandom_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("urandom");
    let mut urandom = File::open("/dev/urandom").unwrap();

    for buffer_size in (0u64..=10u64).map(|i| 1u64 << i) {
        group.throughput(Throughput::Bytes(buffer_size));
        let mut scratch = vec![0u8; buffer_size as usize];

        group.bench_function("throughput", |b| {
            b.iter(|| {
                urandom.read_exact(black_box(&mut scratch)).unwrap();
            })
        });
    }
}

fn chacha_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("chacha");
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    for buffer_size in (0u64..=10u64).map(|i| 1u64 << i) {
        group.throughput(Throughput::Bytes(buffer_size));
        let mut scratch = vec![0u8; buffer_size as usize];

        group.bench_function("throughput", |b| {
            b.iter(|| {
                rng.fill_bytes(black_box(&mut scratch));
            })
        });
    }
}

fn xoshiro_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("Xoshiro256PlusPlus");
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

    for buffer_size in (0u64..=10u64).map(|i| 1u64 << i) {
        group.throughput(Throughput::Bytes(buffer_size));
        let mut scratch = vec![0u8; buffer_size as usize];

        group.bench_function("throughput", |b| {
            b.iter(|| {
                rng.fill_bytes(black_box(&mut scratch));
            })
        });
    }
}

mod xml {
    use criterion::{black_box, BatchSize, BenchmarkId, Criterion, Throughput};
    use fandango_core::generation::Generated;
    use fandango_core::visitor::navigation::CountBytes;

    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::seq::IndexedRandom;
    use rand::SeedableRng;
    use std::collections::{BTreeMap, Bound};

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[fandango(grammar = "eval/grammars/xml.fan")]
    pub struct Xml;

    pub fn nowrite(c: &mut Criterion) {
        let mut group = c.benchmark_group("xml nowrite");

        let rngs = (0..10000)
            .map(|i| {
                let mut rng = rand::rngs::SmallRng::seed_from_u64(i);
                let stashed = rng.clone();

                (
                    nonterminal_start::generate(&mut rng, &mut ()).count_bytes(),
                    stashed,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let rngs = (0usize..=10usize)
            .map(|i| 1usize << i)
            .map(|size| {
                (
                    size,
                    rngs.range((Bound::Included(size), Bound::Excluded(size + size / 10)))
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, v)| !v.is_empty())
            .collect::<BTreeMap<_, _>>();

        for (count, rngs) in rngs {
            group.throughput(Throughput::Bytes(count as u64));

            let mut picker = rand::rngs::SmallRng::seed_from_u64(0);

            group.bench_with_input(BenchmarkId::new("throughput", count), &rngs, |b, rngs| {
                b.iter_batched_ref(
                    || rngs.choose(&mut picker).copied().unwrap().clone(),
                    |rng| nonterminal_start::generate(black_box(rng), &mut ()),
                    BatchSize::SmallInput,
                );
            });
        }
    }

    pub fn throughput(c: &mut Criterion) {
        let mut group = c.benchmark_group("xml");

        let rngs = (0..10000)
            .map(|i| {
                let mut rng = rand::rngs::SmallRng::seed_from_u64(i);
                let stashed = rng.clone();

                (
                    nonterminal_start::generate(&mut rng, &mut ()).count_bytes(),
                    stashed,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let rngs = (0usize..=10usize)
            .map(|i| 1usize << i)
            .map(|size| {
                (
                    size,
                    rngs.range((Bound::Included(size), Bound::Excluded(size + size / 10)))
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, v)| !v.is_empty())
            .collect::<BTreeMap<_, _>>();

        for (count, rngs) in rngs {
            group.throughput(Throughput::Bytes(count as u64));

            let mut picker = rand::rngs::SmallRng::seed_from_u64(0);
            let mut scratch = vec![0u8; count << 1];

            group.bench_with_input(BenchmarkId::new("throughput", count), &rngs, |b, rngs| {
                b.iter_batched_ref(
                    || rngs.choose(&mut picker).copied().unwrap().clone(),
                    |rng| {
                        WriteVisitor::new(black_box(&mut scratch))
                            .visit(&mut nonterminal_start::generate(black_box(rng), &mut ()), 0)
                            .unwrap()
                            .continue_value()
                            .unwrap();
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }
}

criterion_group!(
    benches,
    urandom_throughput,
    chacha_throughput,
    xoshiro_throughput,
    xml::nowrite,
    xml::throughput
);
criterion_main!(benches);
