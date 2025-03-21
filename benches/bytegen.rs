#![allow(missing_docs)]

extern crate alloc;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use fandango_core::typing::Node;
use fandango_core::visitor::{VisitableChildren, Visitor};
use rand::{RngCore, SeedableRng};
use std::error::Error;
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
                urandom.read_exact(&mut scratch).unwrap();
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
                rng.fill_bytes(&mut scratch);
            })
        });
    }
}

mod xml {
    use criterion::{black_box, BatchSize, BenchmarkId, Criterion, Throughput};
    use fandango_core::generation::{Generated, InPlaceGenerated};
    use fandango_core::visitor::navigation::{Advance, CountBytes, CountNodes};
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::seq::IndexedRandom;
    use rand::{Rng, SeedableRng};
    use std::collections::{BTreeMap, Bound};
    use std::error::Error;

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[grammar = "tests/grammars/xml.fan"]
    pub struct Xml;

    pub fn throughput(c: &mut Criterion) {
        let mut group = c.benchmark_group("xml");

        let rngs = (0..10000)
            .map(|i| {
                let mut rng = rand::rngs::StdRng::seed_from_u64(i);
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

            let mut picker = rand::rngs::StdRng::seed_from_u64(0);

            group.bench_with_input(BenchmarkId::new("throughput", count), &rngs, |b, rngs| {
                b.iter_batched_ref(
                    || rngs.choose(&mut picker).copied().unwrap().clone(),
                    |rng| nonterminal_start::generate(black_box(rng), &mut ()),
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
    xml::throughput
);
criterion_main!(benches);
