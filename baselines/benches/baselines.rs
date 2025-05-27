use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches,
    common::perform_benchmark::<csv::Benchmark>,
    common::perform_benchmark::<rest::Benchmark>,
    common::perform_benchmark::<scriptsizec::Benchmark>,
    common::perform_benchmark::<xml::Benchmark>,
);
criterion_main!(benches);
