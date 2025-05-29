use linfa::Dataset;
use linfa::metrics::SingleTargetRegression;
use linfa::traits::{Fit, Predict};
use ndarray::{Array2, s};
use plotters::backend::{BitMapBackend, SVGBackend};
use plotters::chart::{ChartBuilder, LabelAreaPosition};
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::series::LineSeries;
use plotters::style::{BLACK, BLUE, Color, WHITE};
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::num::NonZeroIsize;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, iter};

#[derive(Debug, Clone, Deserialize)]
struct ExperimentSamples {
    sampling_mode: String,
    iters: Vec<f64>,
    times: Vec<f64>,
}

const TARGETS: &'static [&'static str] = &["csv", "rest", "scriptsizec", "xml"];

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
    for (operation, name) in OPERATIONS.iter().zip(FORMAL_NAMES) {
        print!("\\textit{{{name}}}");
        for target in TARGETS {
            let model = results.get(&(target, operation)).unwrap();
            if !operation.starts_with("crossover") {
                print!(" & {:.2}$n$", model.params()[0])
            } else {
                print!(" & {:.2}", model.intercept())
            }
        }
        println!(" \\\\");
    }
}
