//! Analysis for all the things! Run this to reproduce the tables in the paper.

use common::{BenchmarkSuite, StdGenerator, StdSampler};
use fandango::generation::InPlaceGenerated;
use fandango::lang::FandangoNode;
use fandango::tuple_list::{tuple_list, tuple_list_type};
use fandango::typing::{AsNodeRef, AsStaticNode, Discriminable, Node, Opaque};
use fandango::visitor::assignment::SwapVisitor;
use fandango::visitor::navigation::{Advance, CountNodes, GoTo, GoToMut};
use fandango::visitor::{VisitResult, VisitWith, VisitWithMut, VisitableChildren, Visitor};
use fandango_runtime::operators::{DepthLimiter, NodeScan};
use hashbrown::HashMap;
use linfa::traits::{Fit, Predict};
use linfa::{Dataset, DatasetBase};
use linfa_linear::FittedLinearRegression;
use ndarray::{Array, Array2, ArrayBase, FixedInitializer, Ix1, Ix2, OwnedRepr};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::error::Error;
use std::fs::File;
use std::hint::black_box;
use std::io::{BufRead, BufReader};
use std::iter;
use std::ops::Div;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(tag = "kind")]
enum FandangoMeasurement {
    #[serde(rename = "crossover")]
    Crossover(Crossover),
    #[serde(rename = "evaluate-and-fix-individual")]
    EvaluateAndFix(GenericMeasurement),
    #[serde(rename = "evaluation(cache-hit)")]
    EvaluateCached(GenericMeasurement),
    #[serde(rename = "evaluation(cache-miss)")]
    EvaluateMissed(GenericMeasurement),
    #[serde(rename = "generate-population-entry")]
    Generate(GenericMeasurement),
    #[serde(rename = "mutation")]
    Mutation(Mutation),
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct GenericMeasurement {
    pub time: f64,
    pub size: i64,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct Crossover {
    pub time: f64,
    pub parent1_size: i64,
    pub parent2_size: i64,
    pub child1_size: i64,
    pub child2_size: i64,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct Mutation {
    pub time: f64,
    pub size: i64,
    pub mutated_size: i64,
}

struct OperationModel {
    crossover: FittedLinearRegression<f64>,
    evaluate: FittedLinearRegression<f64>,
    fix: FittedLinearRegression<f64>,
    generate: FittedLinearRegression<f64>,
    mutate: FittedLinearRegression<f64>,
}

const SUBJECTS: &'static [&'static str] = &["csv", "rest", "scriptsizec", "xml"];

#[derive(Default)]
struct FandangoNodeCounter {
    count: usize,
}

impl<T> Visitor<T> for FandangoNodeCounter
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        match node.definition() {
            FandangoNode::Nonterminal(_) | FandangoNode::String(_) => {
                self.count += 1;
            }
            _ => {}
        }
        node.opaque().visit_each(self)
    }
}

fn regress<const DIM: usize>(
    measurements: Vec<(f64, [f64; DIM])>,
    features: [&'static str; DIM],
) -> (
    FittedLinearRegression<f64>,
    DatasetBase<ArrayBase<OwnedRepr<f64>, Ix2>, ArrayBase<OwnedRepr<f64>, Ix1>>,
)
where
    [f64; DIM]: FixedInitializer<Elem = f64>, // give the compiler a little help
{
    let targets = Array::from_iter(measurements.iter().map(|(t, _)| *t));
    let data = Array2::from(
        measurements
            .into_iter()
            .map(|(_, e)| e)
            .collect::<Vec<[f64; DIM]>>(),
    );
    let dataset = Dataset::new(data, targets).with_feature_names(features.to_vec());
    let regression = linfa_linear::LinearRegression::new();
    (
        regression.with_intercept(false).fit(&dataset).unwrap(),
        dataset,
    )
}

const WARM_UP: usize = 1 << 5;
const REPETITIONS: usize = 1 << 10;
const DISTR_ATTEMPTS: u64 = 1 << 20;
const DISTR_SEGMENTS: usize = 1 << 8;
const CROSSOVERS: usize = 1 << 4;
const MUTATIONS: usize = 1 << 5;

fn measure<I, F, O>(input: I, f: F) -> (Duration, O)
where
    I: Clone,
    F: FnMut(&mut I) -> O,
{
    let mut inputs = vec![input; REPETITIONS + WARM_UP];
    let mut output = Vec::with_capacity(REPETITIONS + WARM_UP);
    let mut f = black_box(f);

    let chunked = inputs.split_at_mut(WARM_UP);
    let chunked = [chunked.0, chunked.1];

    let mut elapsed = Duration::default();
    for chunk in chunked {
        let start = Instant::now();
        for input in chunk {
            output.push(f(black_box(input)));
        }
        elapsed = Instant::now() - start;
    }
    drop(inputs); // force late drop

    (
        elapsed.div(REPETITIONS as u32),
        output.into_iter().next().unwrap(),
    )
}

fn perform_benchmark<B>(subject: &str) -> OperationModel
where
    B: BenchmarkSuite<StdSampler, StdGenerator>,
    // boilerplate since we're doing this generically
    B::Start: Node + Clone + Ord + AsStaticNode,
    for<'a> <B::Start as Node>::TypeMut<'a>: InPlaceGenerated<
            StdRng,
            tuple_list_type!(DepthLimiter<HashMap<FandangoNode<'static, 'static>, Vec<usize>>>),
        >,
    for<'a> <B::Start as Node>::Type<'a>: VisitWith<'a, FandangoNodeCounter>,
    for<'a> <<B::Start as Node>::Type<'a> as VisitWith<'a, FandangoNodeCounter>>::Visited:
        VisitableChildren<
            <<B::Start as Node>::Type<'a> as VisitWith<'a, FandangoNodeCounter>>::Visited,
        >,
    for<'a> <B::Start as Node>::TypeMut<'a>:
        VisitWithMut<SwapVisitor<<B::Start as Node>::TypeMut<'a>>>,
{
    let mut generator = tuple_list!(DepthLimiter::new(B::program(), 100));

    let mut distribution = BTreeMap::new();
    for seed in 0..DISTR_ATTEMPTS {
        let mut rng = StdRng::seed_from_u64(seed);
        let generated = B::generate(&mut rng, &mut generator);

        distribution.insert(
            FandangoNodeCounter::default()
                .visit(&generated, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .count,
            seed,
        );
    }

    let start = *distribution.first_key_value().unwrap().0;
    let end = *distribution.last_key_value().unwrap().0;

    let mut distributed = (start..end)
        .step_by((end - start).div_ceil(DISTR_SEGMENTS))
        .chain(iter::once(end))
        .map(|size| {
            let (size, seed) = distribution.range(..=size).last().unwrap();
            (*size, *seed)
        })
        .collect::<Vec<_>>();
    drop(distribution);

    distributed.dedup();

    println!("RS models for {subject}:");

    // generation
    let mut samples = Vec::with_capacity(DISTR_SEGMENTS);
    for &(size, seed) in &distributed {
        let (time, generated) = measure(StdRng::seed_from_u64(seed), |sampler| {
            B::generate(sampler, &mut generator)
        });
        assert_eq!(
            FandangoNodeCounter::default()
                .visit(&generated, 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .count,
            size
        );
        samples.push((time.as_secs_f64(), [size as f64]));
    }
    let (generate, data) = regress(samples, ["size"]);
    println!(
        "  generate: {:.2} nanosecond/node (MAE = {:.2}, {} samples)",
        generate.params()[0] * 1_000_000_000f64,
        (generate.predict(&data.records) - data.targets)
            .mapv(|f| f.abs())
            .mean()
            .unwrap_or(0.0)
            * 1_000_000_000f64,
        data.records.len(),
    );

    // fixing
    let mut samples = Vec::with_capacity(DISTR_SEGMENTS);
    for &(size, seed) in &distributed {
        let generated = B::generate(&mut StdRng::seed_from_u64(seed), &mut generator);
        let mut local_sampler = StdSampler::seed_from_u64(0xdeadbeef);
        for _ in 0..MUTATIONS {}
        let (time, _) = measure(generated, |fixed| {
            B::fix(fixed, &mut local_sampler, &mut generator);
        });
        samples.push((time.as_secs_f64(), [size as f64]));
    }
    let (fix, data) = regress(samples, ["size"]);
    println!(
        "  fix: {:.2} nanoseconds/node (MAE = {:.2}, {} samples)",
        fix.params()[0] * 1_000_000_000f64,
        (fix.predict(&data.records) - data.targets)
            .mapv(|f| f.abs())
            .mean()
            .unwrap_or(0.0)
            * 1_000_000_000f64,
        data.records.len(),
    );

    // evaluate
    let mut samples = Vec::with_capacity(DISTR_SEGMENTS);
    for &(size, seed) in &distributed {
        let generated = B::generate(&mut StdRng::seed_from_u64(seed), &mut generator);
        let (time, _) = measure(generated, |fixed| {
            B::check(fixed);
        });
        samples.push((time.as_secs_f64(), [size as f64]));
    }
    let (evaluate, data) = regress(samples, ["size"]);
    println!(
        "  evaluate: {:.2} nanoseconds/node (MAE = {:.2}, {} samples)",
        evaluate.params()[0] * 1_000_000_000f64,
        (evaluate.predict(&data.records) - data.targets)
            .mapv(|f| f.abs())
            .mean()
            .unwrap_or(0.0)
            * 1_000_000_000f64,
        data.records.len(),
    );

    // mutation
    let mut samples = Vec::with_capacity(DISTR_SEGMENTS);
    for &(size, seed) in &distributed {
        let generated = B::generate(&mut StdRng::seed_from_u64(seed), &mut generator);
        let count = generated.count_nodes();
        let mut local_sampler = StdRng::seed_from_u64(0xdeadbeef + seed);
        for _ in 0..MUTATIONS {
            let choice = local_sampler.next_u64() as usize % count;
            let mut path = Advance::forward(choice)
                .visit(&generated, 0)
                .unwrap()
                .break_value()
                .unwrap();
            let (&idx, path) = path.make_contiguous().split_first().unwrap();
            let (time, _) = measure(
                (generated.clone(), local_sampler.clone()),
                |(mutated, sampler)| {
                    black_box(mutated)
                        .go_to_mut(idx, black_box(path))
                        .unwrap()
                        .generate_in_place(sampler, &mut generator, path.len());
                },
            );
            let mut cloned = generated.clone();
            let mut mutated = cloned.go_to_mut(idx, path).unwrap();
            mutated.generate_in_place(&mut local_sampler, &mut generator, path.len());
            drop(mutated);
            let mutated_size = cloned
                .go_to(idx, path)
                .unwrap()
                .visit_with(FandangoNodeCounter::default(), 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .count;
            samples.push((time.as_secs_f64(), [size as f64, mutated_size as f64]));
        }
    }
    let (mutate, data) = regress(samples, ["size", "mutated"]);
    println!(
        "  mutate: {:.2} nanoseconds/node + {:.2} nanoseconds/node generated (MAE = {:.2}, {} samples)",
        mutate.params()[0] * 1_000_000_000f64,
        mutate.params()[1] * 1_000_000_000f64,
        (mutate.predict(&data.records) - data.targets)
            .mapv(|f| f.abs())
            .mean()
            .unwrap_or(0.0)
            * 1_000_000_000f64,
        data.records.len(),
    );

    // crossover
    let mut samples = Vec::with_capacity(DISTR_SEGMENTS);
    for &(size1, seed1) in distributed
        .chunks(distributed.len() / CROSSOVERS)
        .map(|a| a.first().unwrap())
    {
        let generated1 = B::generate(&mut StdRng::seed_from_u64(seed1), &mut generator);
        let count1 = generated1.count_nodes();
        for &(size2, seed2) in distributed
            .chunks(distributed.len() / CROSSOVERS)
            .map(|a| a.last().unwrap())
        {
            let generated2 = B::generate(&mut StdRng::seed_from_u64(seed2), &mut generator);
            let mut local_sampler = StdRng::seed_from_u64(0xdeadbeef + seed1);
            for _ in 0..MUTATIONS {
                let choice = local_sampler.next_u64() as usize % count1;
                let mut path = Advance::forward(choice)
                    .visit(&generated1, 0)
                    .unwrap()
                    .break_value()
                    .unwrap();
                let (&idx, path) = path.make_contiguous().split_first().unwrap();
                let (time, completed) = measure(
                    (
                        generated1.clone(),
                        generated2.clone(),
                        local_sampler.clone(),
                    ),
                    |(parent1, parent2, sampler)| {
                        let crossed1 = black_box(parent1).go_to_mut(idx, black_box(path)).unwrap();
                        let discriminant = crossed1.discriminant();
                        let scan = NodeScan::new_paths(discriminant)
                            .visit(parent2, 0)
                            .unwrap()
                            .continue_value()
                            .unwrap()
                            .matches();
                        if scan.is_empty() {
                            return false;
                        }
                        let choice = scan[sampler.next_u64() as usize % scan.len()].as_slice();
                        let (&idx, path) = choice.split_first().unwrap();
                        let crossed2 = parent2.go_to_mut(idx, path).unwrap();
                        let _ = crossed2
                            .visit_with_mut(
                                SwapVisitor::new(crossed1),
                                path.last().copied().unwrap_or(idx),
                            )
                            .unwrap()
                            .break_value()
                            .unwrap();
                        true
                    },
                );
                if completed {
                    let crossed1 = generated1.go_to(idx, path).unwrap();
                    let discriminant = crossed1.discriminant();
                    let scan = NodeScan::new_paths(discriminant)
                        .visit(&generated2, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .matches();
                    let choice = scan[local_sampler.next_u64() as usize % scan.len()].as_slice();
                    let (&second_idx, second_path) = choice.split_first().unwrap();
                    let crossed2 = generated2.go_to(second_idx, second_path).unwrap();

                    let child1_size = crossed1
                        .visit_with(
                            FandangoNodeCounter::default(),
                            path.last().copied().unwrap_or(second_idx),
                        )
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .count;
                    let child2_size = crossed2
                        .visit_with(
                            FandangoNodeCounter::default(),
                            path.last().copied().unwrap_or(second_idx),
                        )
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .count;

                    samples.push((
                        time.as_secs_f64(),
                        [
                            size1 as f64,
                            size2 as f64,
                            // these get swapped in the mutation
                            child2_size as f64,
                            child1_size as f64,
                        ],
                    ));
                }
            }
        }
    }
    let (crossover, data) = regress(samples, ["parent1", "parent2", "child1", "child2"]);
    println!(
        "  crossover: {:.2} nanoseconds/node of parent 1 + {:.2} nanoseconds/node of parent 2 + {:.2} nanoseconds/node of child 1 + {:.2} nanoseconds/node of child 2 (MAE = {:.2}, {} samples)",
        crossover.params()[0] * 1_000_000_000f64,
        crossover.params()[1] * 1_000_000_000f64,
        crossover.params()[2] * 1_000_000_000f64,
        crossover.params()[3] * 1_000_000_000f64,
        (crossover.predict(&data.records) - data.targets)
            .mapv(|f| f.abs())
            .mean()
            .unwrap_or(0.0)
            * 1_000_000_000f64,
        data.records.len(),
    );

    OperationModel {
        crossover,
        evaluate,
        fix,
        generate,
        mutate,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut fandango_models = HashMap::new();
    for subject in SUBJECTS {
        let mut crossovers = Vec::new();
        let mut evaluates = Vec::new();
        let mut fixes = Vec::new();
        let mut generates = Vec::new();
        let mut mutates = Vec::new();

        for trial in 1..=5 {
            let file = File::open(format!(
                "baselines/profiling-results/{subject}/{trial}/profile.json"
            ))?;
            let reader = BufReader::new(file);
            let mut last = None;
            for line in reader.lines() {
                let line = line?;
                let trimmed = line.trim_end_matches(","); // fix format
                let measurement: FandangoMeasurement = serde_json::from_str(trimmed)?;
                match measurement {
                    FandangoMeasurement::Crossover(cross) => crossovers.push((
                        cross.time,
                        [
                            cross.parent1_size as f64,
                            cross.parent2_size as f64,
                            cross.child1_size as f64,
                            cross.child2_size as f64,
                        ],
                    )),
                    FandangoMeasurement::EvaluateAndFix(ef) => {
                        let Some(
                            FandangoMeasurement::EvaluateMissed(e)
                            | FandangoMeasurement::EvaluateCached(e),
                        ) = last
                        else {
                            panic!(
                                "Encountered an evaluate and fix that was not preceded by an evaluation?"
                            );
                        };
                        fixes.push((ef.time - e.time, [e.size as f64]));
                    }
                    FandangoMeasurement::EvaluateCached(_) => {
                        // skip; we don't measure the caching because there's no reasonable way to
                        // replicate this behavior in Rust
                    }
                    FandangoMeasurement::EvaluateMissed(eval) => {
                        evaluates.push((eval.time, [eval.size as f64]))
                    }
                    FandangoMeasurement::Generate(generate) => {
                        generates.push((generate.time, [generate.size as f64]))
                    }
                    FandangoMeasurement::Mutation(mutation) => mutates.push((
                        mutation.time,
                        [mutation.size as f64, mutation.mutated_size as f64],
                    )),
                }
                last = Some(measurement);
            }
        }

        println!("Fandango models for {subject}:");
        let (generate, data) = regress(generates, ["size"]);
        println!(
            "  generate: {:.2} microseconds/node (MAE = {:.2}, {} samples)",
            generate.params()[0] * 1_000_000f64,
            (generate.predict(&data.records) - data.targets)
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            data.records.len(),
        );
        let (fix, data) = regress(fixes, ["size"]);
        println!(
            "  fix: {:.2} microseconds/node (MAE = {:.2}, {} samples)",
            fix.params()[0] * 1_000_000f64,
            (fix.predict(&data.records) - data.targets)
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            data.records.len(),
        );
        let (evaluate, data) = regress(evaluates, ["size"]);
        println!(
            "  evaluate: {:.2} microseconds/node (MAE = {:.2}, {} samples)",
            evaluate.params()[0] * 1_000_000f64,
            (evaluate.predict(&data.records) - data.targets)
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            data.records.len(),
        );
        let (mutate, data) = regress(mutates, ["size", "mutated"]);
        println!(
            "  mutate: {:.2} microseconds/node + {:.2} microseconds/node generated (MAE = {:.2}, {} samples)",
            mutate.params()[0] * 1_000_000f64,
            mutate.params()[1] * 1_000_000f64,
            (mutate.predict(&data.records) - data.targets)
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            data.records.len(),
        );
        let (crossover, data) = regress(crossovers, ["parent1", "parent2", "child1", "child2"]);
        println!(
            "  crossover: {:.2} microseconds/node of parent 1 + {:.2} microseconds/node of parent 2 + {:.2} microseconds/node of child 1 + {:.2} microseconds/node of child 2 (MAE = {:.2}, {} samples)",
            crossover.params()[0] * 1_000_000f64,
            crossover.params()[1] * 1_000_000f64,
            crossover.params()[2] * 1_000_000f64,
            crossover.params()[3] * 1_000_000f64,
            (crossover.predict(&data.records) - data.targets)
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            data.records.len(),
        );
        fandango_models.insert(
            subject,
            OperationModel {
                crossover,
                evaluate,
                fix,
                generate,
                mutate,
            },
        );
    }

    let mut rs_models = HashMap::new();
    rs_models.insert("csv", perform_benchmark::<csv::Benchmark>("csv"));
    rs_models.insert("rest", perform_benchmark::<rest::Benchmark>("rest"));
    rs_models.insert(
        "scriptsizec",
        perform_benchmark::<scriptsizec::Benchmark>("scriptsizec"),
    );
    rs_models.insert("xml", perform_benchmark::<xml::Benchmark>("xml"));

    Ok(())
}
