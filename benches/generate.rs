#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fandango_core::graph::IntoGraph;
use fandango_core::lang::Program;

pub const SIMPLE_GRAMMAR: &str = include_str!("../tests/grammars/simple.fan");
pub const XML_GRAMMAR: &str = include_str!("../tests/grammars/xml.fan");

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

mod simple {
    use criterion::{black_box, Criterion};
    use fandango_core::generation::util::Flattener;
    use fandango_core::generation::Generated;
    use fandango_core::typing::Node;
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::thread_rng;
    use tuple_list::tuple_list;

    #[derive(Fandango)]
    #[grammar = "tests/grammars/simple.fan"]
    pub struct Simple;

    pub fn simple(c: &mut Criterion) {
        let mut rng = thread_rng();
        c.bench_function("generate simple", |b| {
            b.iter(|| nonterminal_start::generate(&mut rng, &mut ()))
        });
    }

    pub fn visit_simple(c: &mut Criterion) {
        let mut rng = thread_rng();

        c.bench_function("visit simple", |b| {
            b.iter(|| {
                let mut start = nonterminal_start::generate(&mut rng, &mut ());
                let _ = WriteVisitor::cacheless(Vec::new())
                    .visit(black_box(&mut start), 0)
                    .unwrap()
                    .continue_value()
                    .unwrap()
                    .output();
            })
        });
    }

    pub fn simple_flattened(c: &mut Criterion) {
        let mut rng = thread_rng();
        let flattener = Flattener::new().flatten::<nonterminal_digit>().unwrap();
        let mut generators = tuple_list!(flattener);
        c.bench_function("generate flattened", |b| {
            b.iter(|| nonterminal_start::generate(&mut rng, &mut generators))
        });
    }
}

mod xml {
    use criterion::{black_box, Criterion};
    use fandango_core::generation::Generated;
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::thread_rng;

    #[derive(Fandango)]
    #[grammar = "tests/grammars/xml.fan"]
    pub struct Xml;

    pub fn xml(c: &mut Criterion) {
        let mut rng = thread_rng();
        c.bench_function("generate xml", |b| {
            b.iter(|| nonterminal_start::generate(&mut rng, &mut ()))
        });
    }

    pub fn visit_xml(c: &mut Criterion) {
        let mut rng = thread_rng();

        c.bench_function("visit xml", |b| {
            b.iter(|| {
                let mut start = nonterminal_start::generate(&mut rng, &mut ());
                let _ = WriteVisitor::cacheless(Vec::new())
                    .visit(black_box(&mut start), 0)
                    .unwrap()
                    .continue_value()
                    .unwrap()
                    .output();
            })
        });
    }
}

criterion_group!(
    benches,
    parse_simple,
    graph_simple,
    parse_xml,
    graph_xml,
    simple::simple,
    simple::visit_simple,
    simple::simple_flattened,
    xml::xml,
    xml::visit_xml
);
criterion_main!(benches);
