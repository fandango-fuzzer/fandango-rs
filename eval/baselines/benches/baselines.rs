use criterion::{criterion_group, criterion_main};
use csv::CsvBenchmark;

criterion_group!(benches, common::perform_benchmark::<CsvBenchmark>);
criterion_main!(benches);
