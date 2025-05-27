use baselines::perform_benchmark;
use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches,
    perform_benchmark::<csv::Benchmark>,
    perform_benchmark::<rest::Benchmark>,
    perform_benchmark::<scriptsizec::Benchmark>,
    perform_benchmark::<xml::Benchmark>,
);
criterion_main!(benches);
