//! Run fandango-rs for 10 minutes, see how many programs we generate.

use anyhow::Error;
use fandango::tuple_list::tuple_list;
use fandango::visitor::Visitor;
use fandango::visitor::navigation::CountNodes;
use fandango::visitor::write::WriteVisitor;
use fandango_runtime::evolvers::Evolver;
use fandango_runtime::evolvers::multi::{KPathDiversityHook, Nsga2Evolver};
use fandango_runtime::measurement::HasMeasurement;
use fandango_runtime::measurement::{HasFitness, ViolationFitness};
use fandango_runtime::operators::DepthLimiter;
use fandango_runtime::population::Individual;
use fandango_targets::clang::{self};
use num_rational::Ratio;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::num::NonZeroUsize;
use std::process::{Command, Stdio};

fn run_once(
    fine_print: bool,
    print_successful_compile: bool,
) -> Result<(i32, i32, i32, f32, f32), Error> {
    let fitness = ViolationFitness::<clang::CombinedConstraintVisitor>::new();
    let fixer = ();
    let hook = KPathDiversityHook::new(fixer, NonZeroUsize::new(10).unwrap());
    let mut runtime = Nsga2Evolver::new::<clang::nonterminal_start>(
        tuple_list!(fitness),
        hook,
        100,
        1000,
        Ratio::new(80, 100),
    )
    .expect("Should be valid.");

    // For returning
    let mut number_of_generated_programs = 0;
    let mut number_of_programs_with_fitness_1 = 0;
    let mut number_of_programs_accepted_by_gcc = 0;

    let generator = DepthLimiter::new(clang::STRUCTURE.inner(), 100);
    let mut generators = tuple_list!(generator);
    let mut sampler = StdRng::from_os_rng();

    let mut population = runtime.initial(&mut generators, &mut sampler)?;

    // Time the generation process
    let start_time = std::time::Instant::now();
    for i in 0..100 {
        let fitness = population
            .iter()
            .map(|i| i.measurement().fitness())
            .fold(0.0f64, |v, r| v + *r.0.numer() as f64 / *r.0.denom() as f64)
            / population.len() as f64;
        if fitness == 1.0 {
            println!("saturated fitness at generation {i}");
            break;
        }
        println!("average fitness at generation {i}: {fitness}");
        population = runtime.step(&mut generators, &mut sampler, population)?;
    }

    population.sort_by(|i1, i2| i1.node().cmp(i2.node()));
    population.dedup_by(|i1, i2| i1.node() == i2.node());

    let elapsed_gen = start_time.elapsed();

    if fine_print {
        println!("Population:");
    }
    // Time the gcc checking process
    let start_time_compile = std::time::Instant::now();
    for (i, candidate) in population.into_iter().enumerate() {
        if fine_print {
            println!("Candidate #{i} ===============================================");
        }
        number_of_generated_programs += 1;
        // If fitness is 1.0, it means no violations.
        // Try to pass the candidate to gcc.
        // Write the candidate to a file, pre-pended with necessary includes.
        // Then run `gcc -x c -o /dev/null - <file>`.
        // If gcc returns 0, it means the program is valid.
        // If gcc returns non-zero, it means the program is invalid.
        // First, check fitness ratio to see if it's 1.0
        if *candidate.measurement().fitness().0.numer()
            == *candidate.measurement().fitness().0.denom()
        {
            number_of_programs_with_fitness_1 += 1;
        } else {
            if fine_print {
                println!("Candidate has fitness less than 1.0, skipping gcc check.");
                println!("Fitness: {:?}", candidate.measurement().fitness());
            }
            continue;
        }

        let process_or_not = Command::new("gcc")
            .arg("-x")
            .arg("c")
            .arg("-o")
            .arg("/dev/null")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut process = match process_or_not {
            Ok(p) => p,
            Err(e) => {
                if fine_print {
                    println!("Failed to spawn gcc process: {}", e);
                }
                continue;
            }
        };

        let stdin = process.stdin.as_mut().expect("Failed to open stdin");
        use std::io::Write;
        writeln!(stdin, "#include <stdio.h>").unwrap();
        writeln!(stdin, "#include <stdbool.h>").unwrap();
        writeln!(stdin).unwrap();
        // Wrap this in a main function.
        stdin
            .write_all(
                &WriteVisitor::new(Vec::new())
                    .visit(candidate.node(), 0)?
                    .continue_value()
                    .unwrap()
                    .output(),
            )
            .unwrap();
        // Also add a main function that returns 0 to make it a valid C program.
        writeln!(stdin).unwrap();
        writeln!(stdin, "int main() {{ return 0; }}").unwrap();

        let output = process
            .wait_with_output()
            .expect("Failed to read gcc output");

        if output.status.success() {
            if fine_print {
                println!("GCC accepted the program.");
            }
            if print_successful_compile {
                println!(
                    "{}",
                    String::from_utf8(
                        WriteVisitor::new(Vec::new())
                            .visit(candidate.node(), 0)?
                            .continue_value()
                            .unwrap()
                            .output()
                    )?
                );
            }
            number_of_programs_accepted_by_gcc += 1;
        } else if fine_print {
            println!("GCC rejected the program.");
            println!("GCC stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        if fine_print {
            println!("GCC exit code: {}", output.status);
            println!("GCC stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!(
                "{}",
                String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(candidate.node(), 0)?
                        .continue_value()
                        .unwrap()
                        .output()
                )?
            );
        }

        if fine_print {
            println!("Fitness: {:?}", candidate.measurement().fitness());
            println!("Size: {}", candidate.node().count_nodes());
        }
    }

    let elapsed_compile = start_time_compile.elapsed();

    // Print a small summary of this run
    println!("Completed a run.");

    Ok((
        number_of_generated_programs,
        number_of_programs_with_fitness_1,
        number_of_programs_accepted_by_gcc,
        elapsed_gen.as_secs_f32(),
        elapsed_compile.as_secs_f32(),
    ))
}

#[allow(deprecated)]
fn main() -> Result<(), Error> {
    // Run for 10 minutes

    // Get ready for statistics for whole run
    let mut total_programs_generated = 0;
    let mut total_programs_with_fitness_1 = 0;
    let mut total_programs_accepted_by_gcc = 0;
    let mut total_elapsed_gen = 0.0;
    let mut total_elapsed_compile = 0.0;

    // let start = std::time::Instant::now();
    // Actually, just run for 1 minute for testing
    // let duration = std::time::Duration::from_secs(60);
    let target_time_seconds = 60;
    let mut total_elapsed_gen_and_compile = 0.0;
    loop {
        let (generated, fitness_1, accepted, elapsed_gen, elapsed_compile) = run_once(false, true)?;
        total_programs_generated += generated;
        total_programs_with_fitness_1 += fitness_1;
        total_programs_accepted_by_gcc += accepted;
        total_elapsed_gen += elapsed_gen;
        total_elapsed_compile += elapsed_compile;
        total_elapsed_gen_and_compile += elapsed_gen + elapsed_compile;
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        if total_elapsed_gen_and_compile >= target_time_seconds as f32 {
            break;
        }
    }

    println!(
        "\\newcommand{{\\validConstrainedTotalRs}}{{{}\\xspace}}",
        total_programs_generated
    );
    println!(
        "\\newcommand{{\\validConstrainedFitOneRs}}{{{}\\xspace}}",
        total_programs_with_fitness_1
    );
    println!(
        "\\newcommand{{\\validConstrainedCompileRs}}{{{}\\xspace}}",
        total_programs_accepted_by_gcc
    );
    println!(
        "\\newcommand{{\\validConstrainedGenTimeRs}}{{{}s\\xspace}}",
        total_elapsed_gen
    );
    println!(
        "\\newcommand{{\\validConstrainedCompileTimeRs}}{{{}s\\xspace}}",
        total_elapsed_compile
    );
    Ok(())
}
