#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fandango_core::graph::IntoGraph;
use fandango_core::lang::Program;

pub const SIMPLE_GRAMMAR: &str = include_str!("../tests/grammars/simple.fan");

fn parse_simple(c: &mut Criterion) {
    c.bench_function("parse simple grammar", |b| {
        b.iter(|| {
            let _ = Program::try_from(black_box(SIMPLE_GRAMMAR)).unwrap();
        })
    });
}

fn graph_simple(c: &mut Criterion) {
    let program = Program::try_from(SIMPLE_GRAMMAR).unwrap();

    c.bench_function("graph simple grammar", |b| {
        b.iter(|| black_box(&program).into_graph())
    });
}

mod simple {
    use criterion::{black_box, BatchSize, BenchmarkId, Criterion, Throughput};
    use fandango_core::generation::util::Flattener;
    use fandango_core::generation::{Generated, InPlaceGenerated};
    use fandango_core::typing::Node;
    use fandango_core::visitor::navigation::{
        Advance, CountNodes, CountNodesWith, GoTo, StartingFrom,
    };
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_core::visitor_chain;
    use fandango_derive::Fandango;
    use rand::{thread_rng, Rng, SeedableRng};
    use std::collections::BTreeMap;
    use std::error::Error;
    use tuple_list::tuple_list;

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[grammar = "tests/grammars/simple.fan"]
    pub struct Simple;

    pub fn simple(c: &mut Criterion) {
        let mut group = c.benchmark_group("simple");

        let rngs = (0..1000)
            .map(|i| {
                let mut rng = rand::rngs::StdRng::seed_from_u64(i);
                let stashed = rng.clone();

                (
                    nonterminal_start::generate(&mut rng, &mut ()).count_nodes(),
                    stashed,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let count = rngs.len();
        let rngs = rngs.into_iter().step_by(count / 5).collect::<Vec<_>>();

        for (count, mut rng) in rngs {
            group.throughput(Throughput::Elements(count as u64));

            group.bench_with_input(BenchmarkId::new("generate", count), &rng, |b, rng| {
                b.iter_batched_ref(
                    || rng.clone(),
                    |rng| crate::simple::nonterminal_start::generate(black_box(rng), &mut ()),
                    BatchSize::SmallInput,
                );
            });

            let mut start = nonterminal_start::generate(&mut rng.clone(), &mut ());

            group.bench_with_input(BenchmarkId::new("visit", count), &start, |b, start| {
                b.iter_batched_ref(
                    || start.clone(),
                    |start| {
                        WriteVisitor::cacheless(Vec::new())
                            .visit(black_box(start), 0)
                            .unwrap()
                            .continue_value()
                            .unwrap()
                            .output()
                    },
                    BatchSize::SmallInput,
                );
            });

            let mut count = start.count_nodes();

            group.bench_with_input(BenchmarkId::new("mutate", count), &start, |b, start| {
                b.iter_batched_ref(
                    || start.clone(),
                    |start| {
                        let selection = rng.gen_range(0..count);
                        let _: Result<(), Box<dyn Error>> = (|| {
                            let mut target = Advance::forward(selection)
                                .visit(start, 0)?
                                .break_value()
                                .unwrap();
                            target.generate_in_place(&mut rng, &mut ());
                            Ok(())
                        })();
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }

    pub fn simple_flattened(c: &mut Criterion) {
        let flattener = Flattener::new().flatten::<nonterminal_digit>().unwrap();
        let mut generators = tuple_list!(flattener);

        let mut group = c.benchmark_group("simple flattened");

        let rngs = (0..1000)
            .map(|i| {
                let mut rng = rand::rngs::StdRng::seed_from_u64(i);
                let stashed = rng.clone();

                (
                    nonterminal_start::generate(&mut rng, &mut generators).count_nodes(),
                    stashed,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let count = rngs.len();
        let rngs = rngs.into_iter().step_by(count / 5).collect::<Vec<_>>();

        for (count, rng) in rngs {
            group.throughput(Throughput::Elements(count as u64));

            group.bench_with_input(BenchmarkId::new("generate", count), &rng, |b, rng| {
                b.iter_batched_ref(
                    || rng.clone(),
                    |rng| nonterminal_start::generate(black_box(rng), &mut generators),
                    BatchSize::SmallInput,
                );
            });
        }
    }
}

pub const XML_GRAMMAR: &str = include_str!("../tests/grammars/xml.fan");

fn parse_xml(c: &mut Criterion) {
    c.bench_function("parse xml grammar", |b| {
        b.iter(|| {
            let _ = Program::try_from(black_box(XML_GRAMMAR)).unwrap();
        })
    });
}

fn graph_xml(c: &mut Criterion) {
    let program = Program::try_from(XML_GRAMMAR).unwrap();

    c.bench_function("graph xml grammar", |b| {
        b.iter(|| black_box(&program).into_graph())
    });
}

mod xml {
    use criterion::{black_box, BatchSize, BenchmarkId, Criterion, Throughput};
    use fandango_core::generation::{Generated, InPlaceGenerated};
    use fandango_core::visitor::navigation::{Advance, CountNodes};
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_core::visitor_chain;
    use fandango_derive::Fandango;
    use rand::{thread_rng, Rng, SeedableRng};
    use std::collections::BTreeMap;
    use std::error::Error;

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[grammar = "tests/grammars/xml.fan"]
    pub struct Xml;

    pub fn xml(c: &mut Criterion) {
        let mut group = c.benchmark_group("xml");

        let rngs = (0..1000)
            .map(|i| {
                let mut rng = rand::rngs::StdRng::seed_from_u64(i);
                let stashed = rng.clone();

                (
                    nonterminal_start::generate(&mut rng, &mut ()).count_nodes(),
                    stashed,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let count = rngs.len();
        let rngs = rngs.into_iter().step_by(count / 5).collect::<Vec<_>>();

        for (count, mut rng) in rngs {
            group.throughput(Throughput::Elements(count as u64));

            group.bench_with_input(BenchmarkId::new("generate", count), &rng, |b, rng| {
                b.iter_batched_ref(
                    || rng.clone(),
                    |rng| crate::simple::nonterminal_start::generate(black_box(rng), &mut ()),
                    BatchSize::SmallInput,
                );
            });

            let mut start = nonterminal_start::generate(&mut rng.clone(), &mut ());

            group.bench_with_input(BenchmarkId::new("visit", count), &start, |b, start| {
                b.iter_batched_ref(
                    || start.clone(),
                    |start| {
                        WriteVisitor::cacheless(Vec::new())
                            .visit(black_box(start), 0)
                            .unwrap()
                            .continue_value()
                            .unwrap()
                            .output()
                    },
                    BatchSize::SmallInput,
                );
            });

            let mut count = start.count_nodes();

            group.bench_with_input(BenchmarkId::new("mutate", count), &start, |b, start| {
                b.iter_batched_ref(
                    || start.clone(),
                    |start| {
                        let selection = rng.gen_range(0..count);
                        let _: Result<(), Box<dyn Error>> = (|| {
                            let mut target = Advance::forward(selection)
                                .visit(start, 0)?
                                .break_value()
                                .unwrap();
                            target.generate_in_place(&mut rng, &mut ());
                            Ok(())
                        })();
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }
}

criterion_group!(
    benches,
    parse_simple,
    graph_simple,
    parse_xml,
    graph_xml,
    simple::simple,
    simple::simple_flattened,
    xml::xml,
);
criterion_main!(benches);
