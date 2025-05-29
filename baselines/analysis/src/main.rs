use linfa::Dataset;
use linfa::metrics::SingleTargetRegression;
use linfa::traits::{Fit, Predict};
use ndarray::{Array2, s};
use plotters::backend::SVGBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::series::LineSeries;
use plotters::style::{BLACK, BLUE, Color, WHITE};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Cursor;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::{fs, iter};

#[derive(Debug, Copy, Clone, Deserialize)]
struct FandangoSample {
    nodes: usize,
    time: f64,
}

#[derive(Debug, Copy, Clone, Deserialize)]
struct FandangoProfiling {
    initial_population: FandangoRecord,
    evaluate_individual_constr: FandangoRecord,
    evaluate_individual: FandangoRecord,
    select_elites: FandangoRecord,
    tournament_selection: FandangoRecord,
    filling: FandangoRecord,
    fixing: FandangoRecord,
    crossover: FandangoRecord,
    mutation: FandangoRecord,
}

#[derive(Debug, Copy, Clone, Deserialize)]
struct FandangoRecord {
    count: usize,
    time: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct ExperimentSamples {
    sampling_mode: String,
    iters: Vec<f64>,
    times: Vec<f64>,
}

const TARGETS: &'static [&'static str] = &["csv", "rest", "scriptsizec", "xml"];
const FANDANGO_NAMES: &'static [&'static str] = &["csv", "rest", "c", "xml"];

const FANDANGO_SPEEDUP: &'static [f64] = &[1355.0, 146.0, 29.0, 48.0, 183.0];

const OPERATIONS: &'static [&'static str] = &[
    "generate",
    "fix",
    "check",
    "mutate",
    "crossover",
    "generate dynamic",
    "mutate dynamic",
    "crossover dynamic",
];

const FORMAL_NAMES: &'static [&'static str] = &["Generate", "Fix", "Check", "Mutate", "Crossover"];

const ABLATION: &'static [&'static str] = &["generate", "mutate", "crossover"];

fn main() {
    let mut results = HashMap::new();
    for operation in OPERATIONS {
        for target in TARGETS {
            let path = PathBuf::from_iter([".", "target", "criterion", target, operation]);
            let mut samples = Vec::new();
            let mut x_max = 0.0;
            let mut y_max = 0.0;
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if let Ok(count) = usize::from_str(entry.file_name().to_str().unwrap()) {
                    let measured: ExperimentSamples = serde_json::from_reader(
                        File::open(entry.path().join("base/sample.json")).unwrap(),
                    )
                    .unwrap();

                    let x = count as f64;
                    x_max = x.max(x_max);
                    samples.extend(
                        iter::repeat(count as f64)
                            .zip(measured.iters.into_iter().zip(measured.times.into_iter()))
                            .map(|(x, (iters, time))| {
                                let y = time / iters;
                                y_max = y.max(y_max);
                                (x, y)
                            })
                            .map(|(x, y)| [x, y]),
                    );
                }
            }

            let formatted = Array2::from(samples);

            let dataset = Dataset::new(
                formatted.slice(s![.., 0..1]).to_owned(),
                formatted.column(1).to_owned(),
            )
            .with_feature_names(vec!["elements", "time (ns)"]);
            let regression = linfa_linear::LinearRegression::new();
            let mut model = regression
                .with_intercept(operation.starts_with("crossover"))
                .fit(&dataset)
                .unwrap();

            let mape = dataset
                .mean_absolute_percentage_error(&model.predict(&dataset))
                .unwrap();

            let backend_path = PathBuf::from_iter([
                ".",
                "target",
                "figures",
                target,
                operation,
                "linear_regression.svg",
            ]);
            fs::create_dir_all(backend_path.parent().unwrap()).unwrap();
            let root_area = SVGBackend::new(&backend_path, (600, 400)).into_drawing_area();
            root_area.fill(&WHITE).unwrap();

            let mut ctx = ChartBuilder::on(&root_area)
                .caption(
                    format!("Linear Regression of {target} {operation}"),
                    ("sans-serif", 40),
                )
                .build_cartesian_2d(0.0..x_max, 0.0..y_max)
                .unwrap();

            let mut line_points = Vec::with_capacity(2);
            for i in [0.0, x_max] {
                line_points.push((i, (i * model.params()[0]) + model.intercept()));
            }
            let label = format!(
                "y = {:.2}x + {:.2} (MAPE = {:.2}%)",
                model.params()[0],
                model.intercept(),
                mape * 100.0,
            );
            ctx.draw_series(LineSeries::new(line_points, &BLACK))
                .unwrap()
                .label(&label);

            let num_points = formatted.shape()[0];
            let mut points = Vec::with_capacity(num_points);
            for i in 0..formatted.shape()[0] {
                let point = (formatted[[i, 0]], formatted[[i, 1]]);
                let circle = Circle::new(point, 1, BLUE.filled());
                points.push(circle);
            }
            ctx.draw_series(points).unwrap();

            ctx.configure_series_labels()
                .border_style(&BLACK)
                .background_style(&WHITE.mix(0.8))
                .draw()
                .unwrap();

            results.insert((target, operation), model);
        }
    }

    let mut orig_results = HashMap::new();
    let mut orig_profiling = HashMap::new();

    for fandango_name in FANDANGO_NAMES {
        let path = PathBuf::from_iter([".", "fandango-results", fandango_name]);
        let mut distribution = BTreeMap::new();
        let mut total_observations = 0usize;
        let mut profiling_data = fs::read(path.join("profiling.txt")).unwrap();
        for byte in &mut profiling_data {
            if *byte == b'\'' {
                *byte = b'"'; // dict to json lol
            }
        }
        let profiling_data: FandangoProfiling =
            serde_json::from_reader(Cursor::new(profiling_data)).unwrap();
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                continue; // this is the profiling.txt
            }
            let mut content = fs::read(entry.path().join("evaluations.csv")).unwrap();
            for byte in &mut content {
                if *byte == b';' {
                    *byte = b','; // Rust's CSV reader expects commas
                }
            }
            let mut reader = serde_csv::Reader::from_reader(Cursor::new(content));
            for entry in reader.deserialize() {
                let entry: FandangoSample = entry.unwrap();
                *distribution.entry(entry.nodes).or_insert(0usize) += 1;
                total_observations += 1;
            }
        }

        let dist_total = distribution
            .iter()
            .map(|(&nodes, &count)| nodes * count)
            .sum::<usize>();

        for (record, operation) in [
            profiling_data.initial_population,
            profiling_data.fixing,
            profiling_data.evaluate_individual_constr,
            profiling_data.mutation,
        ]
        .into_iter()
        .zip(&OPERATIONS[..4])
        {
            // this gives us the amount of time spent per node relative to the record
            orig_results.insert(
                (fandango_name, operation),
                NonZeroUsize::new(record.count).map(|count| {
                    total_observations as f64 * record.time / (count.get() * dist_total) as f64
                }),
            );
        }

        // this gives us the amount of time spent relative to the record
        orig_results.insert(
            (fandango_name, &OPERATIONS[4]),
            NonZeroUsize::new(profiling_data.crossover.count)
                .map(|count| profiling_data.crossover.time / count.get() as f64),
        );

        let avg_size = dist_total as f64 / total_observations as f64;
        orig_profiling.insert(fandango_name, (profiling_data, avg_size));
    }

    for (operation, name) in OPERATIONS.iter().zip(FORMAL_NAMES) {
        print!("\\textit{{{name}}}");
        for target in TARGETS {
            let model = results.get(&(target, operation)).unwrap();
            if !operation.starts_with("crossover") {
                if model.params()[0] < 1.0 {
                    print!(" & \\textit{{n.d.}}\\tnote{{2}}\\phantom{{{{\\tiny /node}}}}")
                } else {
                    print!(" & {:.2}ns {{\\tiny /node}}", model.params()[0])
                }
            } else {
                print!(
                    " & {}ns {{\\tiny \\phantom{{/node}}}}",
                    model.intercept().round() as usize
                )
            }
        }
        for fandango_name in FANDANGO_NAMES {
            let value = orig_results.get(&(fandango_name, operation)).unwrap();
            if !operation.starts_with("crossover") {
                if let Some(value) = value {
                    // rescale to microseconds
                    print!(" & {:.2}$\\mu$s {{\\tiny /node}}", *value * 1_000_000.0)
                } else {
                    print!(" & \\textit{{n.d.}}\\tnote{{2}}\\phantom{{{{\\tiny /node}}}}");
                }
            } else {
                // rescale to microseconds
                print!(
                    " & {:.2}$\\mu$s {{\\tiny \\phantom{{/node}}}}",
                    value.unwrap() * 1_000_000.0
                )
            }
        }
        println!(" \\\\");
    }

    println!();
    for ((target, fandango_name), &speedup) in
        TARGETS.iter().zip(FANDANGO_NAMES).zip(FANDANGO_SPEEDUP)
    {
        print!("{target}");
        let (profiling, avg_nodes) = orig_profiling.get(fandango_name).unwrap();
        let models = OPERATIONS[..5]
            .iter()
            .map(|op| (op, results.get(&(target, op)).unwrap()))
            .zip([
                &profiling.initial_population,
                &profiling.fixing,
                &profiling.evaluate_individual_constr,
                &profiling.mutation,
                &profiling.crossover,
            ])
            .map(|((op, result), profiling)| {
                (
                    op,
                    (
                        // time in seconds to complete the same task
                        (result.params()[0] * *avg_nodes + result.intercept())
                            * profiling.count as f64
                            / 1_000_000_000f64,
                        profiling.time,
                    ),
                )
            })
            .collect::<HashMap<_, _>>();

        let total_time = [
            profiling.initial_population,
            profiling.fixing,
            profiling.evaluate_individual_constr,
            profiling.mutation,
            profiling.crossover,
            profiling.filling,
            profiling.select_elites,
            profiling.tournament_selection,
        ]
        .into_iter()
        .map(|r| r.time)
        .sum::<f64>();

        let reducible_time = models.values().map(|(_, r)| *r).sum::<f64>();
        let reduced_time =
            models.values().map(|(v, _)| *v).sum::<f64>() + total_time - reducible_time;

        let isla_time = Duration::from_secs_f64(total_time * speedup);
        print!(" & {:.2} seconds", reduced_time);
        print!(" & \\textbf{{{:.2}$\\times$}}", total_time / reduced_time);
        print!(" & 1 hour");
        println!(
            " & {:.2} days \\\\",
            isla_time.as_secs_f64() / (24 * 60 * 60) as f64
        );
    }

    println!();

    for operation in ABLATION {
        print!("\\textit{{{operation}}}");
        for target in TARGETS {
            let model = results.get(&(target, operation)).unwrap();
            let dynamic = format!("{operation} dynamic");
            let reference = dynamic.as_str(); // easier than fixing the rest...
            let dynamic = results.get(&(target, &reference)).unwrap();
            if !operation.starts_with("crossover") {
                print!(" & {:.2}$\\times$", dynamic.params()[0] / model.params()[0])
            } else {
                print!(" & {:.2}$\\times$", dynamic.intercept() / model.intercept())
            }
        }
        println!(" \\\\");
    }
}
