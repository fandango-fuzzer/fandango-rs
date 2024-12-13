use criterion::{criterion_group, criterion_main};

mod simple {
    use criterion::Criterion;
    use fandango::typing::Node;
    use fandango_core::generation::{DefaultGenerated, Generated};
    use fandango_derive::Fandango;
    use rand::thread_rng;
    #[derive(Fandango)]
    #[grammar = "tests/grammars/simple.fan"]
    pub struct Simple;

    pub fn criterion_benchmark(c: &mut Criterion) {
        let mut rng = thread_rng();
        c.bench_function("generate simple", |b| {
            b.iter(|| nonterminal_start::generate(&mut rng, &mut ()))
        });
    }
}

mod xml {
    use criterion::Criterion;
    use fandango::typing::Node;
    use fandango_core::generation::{DefaultGenerated, Generated};
    use fandango_derive::Fandango;
    use rand::thread_rng;
    #[derive(Fandango)]
    #[grammar = "tests/grammars/xml.fan"]
    pub struct Xml;

    pub fn criterion_benchmark(c: &mut Criterion) {
        let mut rng = thread_rng();
        c.bench_function("generate xml", |b| {
            b.iter(|| nonterminal_start::generate(&mut rng, &mut ()))
        });
    }
}

criterion_group!(
    benches,
    simple::criterion_benchmark,
    xml::criterion_benchmark
);
criterion_main!(benches);
