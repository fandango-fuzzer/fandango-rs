//! Analysis for all the things! Run this to reproduce the tables in the paper.

use baselines::{DataRepr, OperationModel, regress};
use hashbrown::HashMap;
use linfa::dataset::Records;
use linfa::traits::Predict;
use linfa_linear::FittedLinearRegression;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct OperationData {
    crossover: DataRepr,
    evaluate: DataRepr,
    // fix: DataRepr,
    generate: DataRepr,
    mutate: DataRepr,
}

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
    time: f64,
    size: i64,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct Crossover {
    time: f64,
    parent1_size: i64,
    parent2_size: i64,
    child1_size: i64,
    child2_size: i64,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct Mutation {
    time: f64,
    size: i64,
    mutated_size: i64,
}

const SUBJECTS: &[&str] = &["csv", "rest", "scriptsizec", "xml"];
const PROPER_NAMES: &[&str] = &["CSV", "REST", "ScriptSizeC", "XML"];

fn main() -> Result<(), Box<dyn Error>> {
    let mut fandango_models = HashMap::new();
    let mut fandango_data = HashMap::new();
    for &subject in SUBJECTS {
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
        let (generate, generate_data) = regress(generates, ["size"]);
        println!(
            "  generate: {:.2} microseconds/node (MAE = {:.2}, {} samples)",
            generate.params()[0] * 1_000_000f64,
            (generate.predict(&generate_data.records) - generate_data.targets.view())
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            generate_data.records.len(),
        );
        let (fix, fix_data) = regress(fixes, ["size"]);
        println!(
            "  fix: {:.2} microseconds/node (MAE = {:.2}, {} samples)",
            fix.params()[0] * 1_000_000f64,
            (fix.predict(&fix_data.records) - fix_data.targets.view())
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            fix_data.records.len(),
        );
        let (evaluate, evaluate_data) = regress(evaluates, ["size"]);
        println!(
            "  evaluate: {:.2} microseconds/node (MAE = {:.2}, {} samples)",
            evaluate.params()[0] * 1_000_000f64,
            (evaluate.predict(&evaluate_data.records) - evaluate_data.targets.view())
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            evaluate_data.records.len(),
        );
        let (mutate, mutate_data) = regress(mutates, ["size", "mutated"]);
        println!(
            "  mutate: {:.2} microseconds/node + {:.2} microseconds/node generated (MAE = {:.2}, {} samples)",
            mutate.params()[0] * 1_000_000f64,
            mutate.params()[1] * 1_000_000f64,
            (mutate.predict(&mutate_data.records) - mutate_data.targets.view())
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            mutate_data.records.len(),
        );
        let (crossover, crossover_data) =
            regress(crossovers, ["parent1", "parent2", "child1", "child2"]);
        println!(
            "  crossover: {:.2} microseconds/node of parent 1 + {:.2} microseconds/node of parent 2 + {:.2} microseconds/node of child 1 + {:.2} microseconds/node of child 2 (MAE = {:.2}, {} samples)",
            crossover.params()[0] * 1_000_000f64,
            crossover.params()[1] * 1_000_000f64,
            crossover.params()[2] * 1_000_000f64,
            crossover.params()[3] * 1_000_000f64,
            (crossover.predict(&crossover_data.records) - crossover_data.targets.view())
                .mapv(|f| f.abs())
                .mean()
                .unwrap_or(0.0)
                * 1_000_000f64,
            crossover_data.records.len(),
        );
        fandango_models.insert(
            subject,
            OperationModel {
                crossover,
                evaluate: Some(evaluate),
                fix: Some(fix),
                generate,
                mutate,
            },
        );
        fandango_data.insert(
            subject,
            OperationData {
                crossover: crossover_data,
                evaluate: evaluate_data,
                // fix: fix_data,
                generate: generate_data,
                mutate: mutate_data,
            },
        );
    }

    let rs_models: HashMap<String, OperationModel> = serde_json::from_reader(File::open(
        "baselines/profiling-results/rs-models-default.json",
    )?)?;

    let maybe_print_rs = |model: &FittedLinearRegression<f64>, suffixes: &[&str]| {
        if model.params().iter().all(|&p| p * 1_000_000_000f64 < 0.05) {
            Cow::Borrowed(r"\emph{n.d.}\tnote{1}")
        } else {
            let mut collected = Vec::with_capacity(suffixes.len());
            for (idx, suffix) in suffixes.iter().enumerate() {
                collected.push(format!(
                    "{:.2}{suffix}",
                    model.params()[idx] * 1_000_000_000f64
                ));
            }
            Cow::Owned(format!("${}$", collected.join(" + ").replace("+ -", "- ")))
        }
    };
    let maybe_print_py =
        |data: &DataRepr, model: &FittedLinearRegression<f64>, suffixes: &[&str]| {
            if data.nsamples() < 25 {
                Cow::Borrowed(r"\emph{n.d.}\tnote{1}")
            } else {
                let mut collected = Vec::with_capacity(suffixes.len());
                for (idx, suffix) in suffixes.iter().enumerate() {
                    collected.push(format!("{:.2}{suffix}", model.params()[idx] * 1_000_000f64));
                }
                Cow::Owned(format!("${}$", collected.join(" + ").replace("+ -", "- ")))
            }
        };

    println!("Table 1: Wall time taken to complete various operations:");
    println!(
        "{}",
        r#"
    \begin{tabular}{lrrrr}
    \toprule
     & \multicolumn{4}{c}{\tool{} (nanoseconds)} \\
     \cmidrule(l{0.25em}r{0.25em}){2-5}
     & \textit{Generate} & \textit{Check} & \textit{Mutate} & \textit{Crossover} \\
     \midrule"#
            .trim_matches('\n')
    );
    for (&subject, &name) in SUBJECTS.into_iter().zip(PROPER_NAMES) {
        let model = rs_models.get(subject).unwrap();

        println!(
            r"    {name} & {} & {} & {} & {} \\",
            maybe_print_rs(&model.generate, &["n"]),
            maybe_print_rs(model.evaluate.as_ref().unwrap(), &["n"]),
            maybe_print_rs(&model.mutate, &["n", "m"]),
            maybe_print_rs(&model.crossover, &["n_1", "n_2", "m_1", "m_2"]),
        )
    }
    println!(
        "{}",
        r#"
    \midrule
     & \multicolumn{4}{c}{\fandango{} (microseconds)} \\
     \cmidrule(l{0.25em}r{0.25em}){2-5}
     & \textit{Generate} & \textit{Check} & \textit{Mutate} & \textit{Crossover} \\
     \midrule"#
            .trim_matches('\n')
    );
    for (&subject, &name) in SUBJECTS.into_iter().zip(PROPER_NAMES) {
        let model = fandango_models.get(subject).unwrap();
        let data = fandango_data.get(subject).unwrap();

        println!(
            r"    {name} & {} & {} & {} & {} \\",
            maybe_print_py(&data.generate, &model.generate, &["n"]),
            maybe_print_py(&data.evaluate, model.evaluate.as_ref().unwrap(), &["n"]),
            maybe_print_py(&data.mutate, &model.mutate, &["n", "m"]),
            maybe_print_py(
                &data.crossover,
                &model.crossover,
                &["n_1", "n_2", "m_1", "m_2"]
            ),
        )
    }
    println!(
        "{}",
        r#"
    \bottomrule
    \end{tabular}
    \begin{tablenotes}
        \item[1] Optimized out (\tool{}) or insufficient samples observed (\fandango{}).
    \end{tablenotes}"#
            .trim_matches('\n')
    );

    // -------------

    println!("Table 2: Wall time of dynamic operations:");
    println!(
        "{}",
        r#"
    \begin{tabular}{lrrrr}
    \toprule
     & \textit{Generate} & \textit{Check} & \textit{Mutate} & \textit{Crossover} \\
     \midrule"#
            .trim_matches('\n')
    );
    for (&subject, &name) in SUBJECTS.into_iter().zip(PROPER_NAMES) {
        let model = rs_models.get(&format!("{}_dyn", subject)).unwrap();
        println!(
            r"    {name} & {} & \emph{{n.d.}}\tnote{{2}} & {} & {} \\",
            maybe_print_rs(&model.generate, &["n"]),
            maybe_print_rs(&model.mutate, &["n", "m"]),
            maybe_print_rs(&model.crossover, &["n_1", "n_2", "m_1", "m_2"]),
        )
    }
    println!(
        "{}",
        r#"
    \bottomrule
    \end{tabular}
    \begin{tablenotes}
        \item[1] Optimized out.
        \item[2] Unimplemented.
    \end{tablenotes}"#
            .trim_matches('\n')
    );
    drop(rs_models);

    // -------------

    let rs_models_noopt: HashMap<String, OperationModel> = serde_json::from_reader(File::open(
        "baselines/profiling-results/rs-models-noopt.json",
    )?)?;

    println!("Table 3: Wall time taken for unoptimized operations:");
    println!(
        "{}",
        r#"
    \begin{tabular}{lrrrr}
    \toprule
     & \multicolumn{4}{c}{\tool{} (static, unoptimized; nanoseconds)} \\
     \cmidrule(l{0.25em}r{0.25em}){2-5}
     & \textit{Generate} & \textit{Check} & \textit{Mutate} & \textit{Crossover} \\
     \midrule"#
            .trim_matches('\n')
    );
    for (&subject, &name) in SUBJECTS.into_iter().zip(PROPER_NAMES) {
        let model = rs_models_noopt.get(subject).unwrap();
        println!(
            r"    {name} & {} & {} & {} & {} \\",
            maybe_print_rs(&model.generate, &["n"]),
            maybe_print_rs(model.evaluate.as_ref().unwrap(), &["n"]),
            maybe_print_rs(&model.mutate, &["n", "m"]),
            maybe_print_rs(&model.crossover, &["n_1", "n_2", "m_1", "m_2"]),
        )
    }
    println!(
        "{}",
        r#"
    \midrule
     & \multicolumn{4}{c}{\tool{} (dynamic, unoptimized; nanoseconds)} \\
     \cmidrule(l{0.25em}r{0.25em}){2-5}
     & \textit{Generate} & \textit{Check} & \textit{Mutate} & \textit{Crossover} \\
     \midrule"#
            .trim_matches('\n')
    );
    for (&subject, &name) in SUBJECTS.into_iter().zip(PROPER_NAMES) {
        let model = rs_models_noopt.get(&format!("{}_dyn", subject)).unwrap();
        println!(
            r"    {name} & {} & \emph{{n.d.}}\tnote{{2}} & {} & {} \\",
            maybe_print_rs(&model.generate, &["n"]),
            maybe_print_rs(&model.mutate, &["n", "m"]),
            maybe_print_rs(&model.crossover, &["n_1", "n_2", "m_1", "m_2"]),
        )
    }
    println!(
        "{}",
        r#"
    \bottomrule
    \end{tabular}
    \begin{tablenotes}
        \item[1] Optimized out.
        \item[2] Unimplemented.
    \end{tablenotes}"#
            .trim_matches('\n')
    );
    drop(rs_models_noopt);

    // -------------

    let rs_models_noopt_indirect: HashMap<String, OperationModel> = serde_json::from_reader(
        File::open("baselines/profiling-results/rs-models-noopt-indirect.json")?,
    )?;

    println!("Table 4: Wall time of operations without indirection optimization:");
    println!(
        "{}",
        r#"
    \begin{tabular}{lrrrr}
    \toprule
     & \textit{Generate} & \textit{Check} & \textit{Mutate} & \textit{Crossover} \\
     \midrule"#
            .trim_matches('\n')
    );
    for (&subject, &name) in SUBJECTS.into_iter().zip(PROPER_NAMES) {
        let model = rs_models_noopt_indirect.get(subject).unwrap();
        println!(
            r"    {name} & {} & {} & {} & {} \\",
            maybe_print_rs(&model.generate, &["n"]),
            maybe_print_rs(model.evaluate.as_ref().unwrap(), &["n"]),
            maybe_print_rs(&model.mutate, &["n", "m"]),
            maybe_print_rs(&model.crossover, &["n_1", "n_2", "m_1", "m_2"]),
        )
    }
    println!(
        "{}",
        r#"
    \bottomrule
    \end{tabular}
    \begin{tablenotes}
        \item[1] Optimized out.
    \end{tablenotes}"#
            .trim_matches('\n')
    );

    Ok(())
}
