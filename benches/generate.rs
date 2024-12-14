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
    use criterion::{black_box, Criterion};
    use fandango_core::generation::util::Flattener;
    use fandango_core::generation::Generated;
    use fandango_core::typing::Node;
    use fandango_core::visitor::mutator::Mutator;
    use fandango_core::visitor::navigation::{
        Advance, CountNodes, CountNodesWith, GoTo, StartingFrom,
    };
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::{thread_rng, Rng};
    use tuple_list::tuple_list;

    #[allow(dead_code)]
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

    pub fn mutate_simple(c: &mut Criterion) {
        let mut rng = thread_rng();
        let mut start = Simple::extract("0").unwrap();
        let mut count = start.count_nodes();
        let mut generators = ();

        c.bench_function("mutate simple", |b| {
            b.iter(|| {
                let old_start = start.clone();
                let selection = rng.gen_range(0..count);
                let mut path = Advance::forward(selection)
                    .visit(&mut start, 0)
                    .unwrap()
                    .break_value()
                    .unwrap();
                let idx = path.pop_front().unwrap();
                assert_eq!(0, idx);
                let old_count = start.go_to(idx, path.clone()).unwrap().count_nodes();
                let mutator = Mutator::new(&mut rng, &mut generators);
                let new = mutator
                    .starting_from(path)
                    .visit(&mut start, idx)
                    .unwrap()
                    .break_value()
                    .unwrap();
                let new_count = new.count_nodes();
                count = count - old_count + new_count;
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
    use criterion::{black_box, Criterion};
    use fandango_core::generation::Generated;
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::thread_rng;

    #[allow(dead_code)]
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
    simple::mutate_simple,
    simple::visit_simple,
    simple::simple_flattened,
    xml::xml,
    xml::visit_xml
);
criterion_main!(benches);
