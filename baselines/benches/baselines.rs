use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches,
    common::perform_benchmark::<csv::Benchmark>,
    common::perform_benchmark::<rest::Benchmark>
);
criterion_main!(benches);
