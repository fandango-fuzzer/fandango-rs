#![allow(missing_docs)]
#![allow(deprecated)] // for DynamicNode

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fandango_core::graph::IntoGraph;
use fandango_core::lang::Program;

extern crate alloc;

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
    use fandango_core::dynamic::{DynamicNode, DynamicSampler};
    use fandango_core::generation::util::Flattener;
    use fandango_core::generation::{Generated, InPlaceGenerated};
    use fandango_core::typing::{AsNode, AsStaticNode, Node, Structured};
    use fandango_core::visitor::navigation::{Advance, CountNodes};
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::{Rng, SeedableRng};
    use std::collections::BTreeMap;
    use std::error::Error;
    use tuple_list::tuple_list;

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[fandango(grammar = "tests/grammars/simple.fan")]
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

        let count = rngs.len() - 1;
        let rngs = rngs.into_iter().step_by(count / 5).collect::<Vec<_>>();

        let nonterminals = nonterminal_start::ROOT.inner().nonterminals();

        for (count, mut rng) in rngs {
            group.throughput(Throughput::Elements(count as u64));

            group.bench_with_input(BenchmarkId::new("generate", count), &rng, |b, rng| {
                b.iter_batched_ref(
                    || rng.clone(),
                    |rng| nonterminal_start::generate(black_box(rng), &mut ()),
                    BatchSize::SmallInput,
                );
            });

            group.bench_with_input(
                BenchmarkId::new("generate dynamic", count),
                &rng,
                |b, rng| {
                    b.iter_batched_ref(
                        || rng.clone(),
                        |rng| {
                            DynamicNode::generate(
                                &mut DynamicSampler::new(
                                    nonterminal_start::static_root(),
                                    nonterminal_start::static_definition(),
                                    &nonterminals,
                                    black_box(rng),
                                ),
                                &mut (),
                            )
                        },
                        BatchSize::SmallInput,
                    );
                },
            );

            let mut start = nonterminal_start::generate(&mut rng.clone(), &mut ());
            let dyn_start = DynamicNode::generate(
                &mut DynamicSampler::new(
                    nonterminal_start::static_root(),
                    nonterminal_start::static_definition(),
                    &nonterminals,
                    &mut rng.clone(),
                ),
                &mut (),
            );

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

            group.bench_with_input(
                BenchmarkId::new("visit dynamic", count),
                &dyn_start,
                |b, start| {
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
                },
            );

            let count = start.count_nodes();

            group.bench_with_input(BenchmarkId::new("mutate", count), &start, |b, start| {
                b.iter_batched_ref(
                    || start.clone(),
                    |start| {
                        let selection = rng.random_range(0..count);
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

            group.bench_with_input(
                BenchmarkId::new("mutate dynamic", count),
                &dyn_start,
                |b, start| {
                    b.iter_batched_ref(
                        || start.clone(),
                        |start| {
                            let selection = rng.random_range(0..count);
                            let _: Result<(), Box<dyn Error>> = (|| {
                                let target = Advance::forward(selection)
                                    .visit(start, 0)?
                                    .break_value()
                                    .unwrap();
                                let mut rng = rng.clone();
                                let mut sampler = DynamicSampler::new(
                                    target.root(),
                                    target.definition(),
                                    &nonterminals,
                                    &mut rng,
                                );
                                target.generate_in_place(&mut sampler, &mut ());
                                Ok(())
                            })();
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }

        let flattener = Flattener::new().flatten::<nonterminal_digit>().unwrap();
        let mut generators = tuple_list!(flattener);
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

        let count = rngs.len() - 1;
        let rngs = rngs.into_iter().step_by(count / 5).collect::<Vec<_>>();

        let nonterminals = nonterminal_start::ROOT.inner().nonterminals();

        for (count, rng) in rngs {
            group.throughput(Throughput::Elements(count as u64));

            group.bench_with_input(
                BenchmarkId::new("generate flattened", count),
                &rng,
                |b, rng| {
                    b.iter_batched_ref(
                        || rng.clone(),
                        |rng| nonterminal_start::generate(black_box(rng), &mut generators),
                        BatchSize::SmallInput,
                    );
                },
            );

            group.bench_with_input(
                BenchmarkId::new("generate flattened dynamic", count),
                &rng,
                |b, rng| {
                    b.iter_batched_ref(
                        || rng.clone(),
                        |rng| {
                            DynamicNode::generate(
                                &mut DynamicSampler::new(
                                    nonterminal_start::static_root(),
                                    nonterminal_start::static_definition(),
                                    &nonterminals,
                                    black_box(rng),
                                ),
                                &mut generators,
                            )
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
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
    use fandango_core::dynamic::{DynamicNode, DynamicSampler};
    use fandango_core::generation::util::Flattener;
    use fandango_core::generation::{Generated, InPlaceGenerated};
    use fandango_core::typing::{AsNode, AsStaticNode, Structured};
    use fandango_core::visitor::navigation::{Advance, CountNodes};
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::{Rng, SeedableRng};
    use std::collections::BTreeMap;
    use std::error::Error;
    use tuple_list::tuple_list;

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[fandango(grammar = "tests/grammars/xml.fan")]
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

        let count = rngs.len() - 1;
        let rngs = rngs.into_iter().step_by(count / 5).collect::<Vec<_>>();

        let nonterminals = nonterminal_start::ROOT.inner().nonterminals();

        for (count, mut rng) in rngs {
            group.throughput(Throughput::Elements(count as u64));

            group.bench_with_input(BenchmarkId::new("generate", count), &rng, |b, rng| {
                b.iter_batched_ref(
                    || rng.clone(),
                    |rng| nonterminal_start::generate(black_box(rng), &mut ()),
                    BatchSize::SmallInput,
                );
            });

            group.bench_with_input(
                BenchmarkId::new("generate dynamic", count),
                &rng,
                |b, rng| {
                    b.iter_batched_ref(
                        || rng.clone(),
                        |rng| {
                            DynamicNode::generate(
                                &mut DynamicSampler::new(
                                    nonterminal_start::static_root(),
                                    nonterminal_start::static_definition(),
                                    &nonterminals,
                                    black_box(rng),
                                ),
                                &mut (),
                            )
                        },
                        BatchSize::SmallInput,
                    );
                },
            );

            let mut start = nonterminal_start::generate(&mut rng.clone(), &mut ());
            let dyn_start = DynamicNode::generate(
                &mut DynamicSampler::new(
                    nonterminal_start::static_root(),
                    nonterminal_start::static_definition(),
                    &nonterminals,
                    &mut rng.clone(),
                ),
                &mut (),
            );

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

            group.bench_with_input(
                BenchmarkId::new("visit dynamic", count),
                &dyn_start,
                |b, start| {
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
                },
            );

            let count = start.count_nodes();

            group.bench_with_input(BenchmarkId::new("mutate", count), &start, |b, start| {
                b.iter_batched_ref(
                    || start.clone(),
                    |start| {
                        let selection = rng.random_range(0..count);
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

            group.bench_with_input(
                BenchmarkId::new("mutate dynamic", count),
                &dyn_start,
                |b, start| {
                    b.iter_batched_ref(
                        || start.clone(),
                        |start| {
                            let selection = rng.random_range(0..count);
                            let _: Result<(), Box<dyn Error>> = (|| {
                                let target = Advance::forward(selection)
                                    .visit(start, 0)?
                                    .break_value()
                                    .unwrap();
                                let mut rng = rng.clone();
                                let mut sampler = DynamicSampler::new(
                                    target.root(),
                                    target.definition(),
                                    &nonterminals,
                                    &mut rng,
                                );
                                target.generate_in_place(&mut sampler, &mut ());
                                Ok(())
                            })();
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }

        let flattener = Flattener::new().flatten::<nonterminal_id_char>().unwrap();
        let mut generators = tuple_list!(flattener);

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

        let count = rngs.len() - 1;
        let rngs = rngs.into_iter().step_by(count / 5).collect::<Vec<_>>();

        let nonterminals = nonterminal_start::ROOT.inner().nonterminals();

        for (count, rng) in rngs {
            group.throughput(Throughput::Elements(count as u64));

            group.bench_with_input(
                BenchmarkId::new("generate flattened", count),
                &rng,
                |b, rng| {
                    b.iter_batched_ref(
                        || rng.clone(),
                        |rng| nonterminal_start::generate(black_box(rng), &mut generators),
                        BatchSize::SmallInput,
                    );
                },
            );

            group.bench_with_input(
                BenchmarkId::new("generate flattened dynamic", count),
                &rng,
                |b, rng| {
                    b.iter_batched_ref(
                        || rng.clone(),
                        |rng| {
                            DynamicNode::generate(
                                &mut DynamicSampler::new(
                                    nonterminal_start::static_root(),
                                    nonterminal_start::static_definition(),
                                    &nonterminals,
                                    black_box(rng),
                                ),
                                &mut generators,
                            )
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
}

// mod ssl {
//     use criterion::{black_box, BatchSize, BenchmarkId, Criterion, Throughput};
//     use fandango_core::generation::{Generated, InPlaceGenerated};
//     use fandango_core::typing::Node;
//     use fandango_core::visitor::navigation::{Advance, CountNodes};
//     use fandango_core::visitor::write::WriteVisitor;
//     use fandango_core::visitor::Visitor;
//     use fandango_derive::Fandango;
//     use rand::{Rng, SeedableRng};
//     use std::collections::BTreeMap;
//     use std::error::Error;
//
//     #[allow(dead_code)]
//     #[derive(Fandango)]
//     #[fandango(grammar = "tests/grammars/ssl.fan", parse = false)]
//     pub struct Ssl;
//
//     pub fn ssl(c: &mut Criterion) {
//         let mut group = c.benchmark_group("ssl");
//
//         let rngs = (0..1000)
//             .map(|i| {
//                 let mut rng = rand::rngs::StdRng::seed_from_u64(i);
//                 let stashed = rng.clone();
//
//                 (
//                     nonterminal_start::generate(&mut rng, &mut ()).count_nodes(),
//                     stashed,
//                 )
//             })
//             .collect::<BTreeMap<_, _>>();
//
//         let count = rngs.len() - 1;
//         let rngs = rngs.into_iter().step_by(count / 5).collect::<Vec<_>>();
//
//         for (count, mut rng) in rngs {
//             group.throughput(Throughput::Elements(count as u64));
//
//             group.bench_with_input(BenchmarkId::new("generate", count), &rng, |b, rng| {
//                 b.iter_batched_ref(
//                     || rng.clone(),
//                     |rng| nonterminal_start::generate(black_box(rng), &mut ()),
//                     BatchSize::SmallInput,
//                 );
//             });
//
//             let mut start = nonterminal_start::generate(&mut rng.clone(), &mut ());
//
//             group.bench_with_input(BenchmarkId::new("visit", count), &start, |b, start| {
//                 b.iter_batched_ref(
//                     || start.clone(),
//                     |start| {
//                         WriteVisitor::cacheless(Vec::new())
//                             .visit(black_box(start), 0)
//                             .unwrap()
//                             .continue_value()
//                             .unwrap()
//                             .output()
//                     },
//                     BatchSize::SmallInput,
//                 );
//             });
//
//             let count = start.count_nodes();
//
//             group.bench_with_input(BenchmarkId::new("mutate", count), &start, |b, start| {
//                 b.iter_batched_ref(
//                     || start.clone(),
//                     |start| {
//                         let selection = rng.random_range(0..count);
//                         let _: Result<(), Box<dyn Error>> = (|| {
//                             let mut target = Advance::forward(selection)
//                                 .visit(start, 0)?
//                                 .break_value()
//                                 .unwrap();
//                             target.generate_in_place(&mut rng, &mut ());
//                             Ok(())
//                         })();
//                     },
//                     BatchSize::SmallInput,
//                 );
//             });
//         }
//     }
// }

criterion_group!(
    benches,
    parse_simple,
    graph_simple,
    parse_xml,
    graph_xml,
    simple::simple,
    xml::xml,
    // ssl::ssl,
);
criterion_main!(benches);
