//! A libafl example fuzzer

extern crate alloc;
use core::time::Duration;
use std::{env, path::PathBuf};

use clap::Parser;
use fandango::visitor::{Visitor as _, write::WriteVisitor};
use fandango_targets::xml::nonterminal_start;
use libafl::{
    Error,
    corpus::{InMemoryCorpus, OnDiskCorpus},
    events::{EventConfig, EventRestarter, Launcher, LlmpRestartingEventManager},
    executors::{ExitKind, inprocess::InProcessExecutor},
    feedback_or, feedback_or_fast,
    feedbacks::{CrashFeedback, MaxMapFeedback, TimeFeedback, TimeoutFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    monitors::{MultiMonitor, OnDiskJsonMonitor},
    observers::{CanTrack, HitcountsMapObserver, StdMapObserver, TimeObserver},
    schedulers::StdScheduler,
    stages::{StdMutationalStage, calibrate::CalibrationStage},
    state::{HasRand, StdState},
};
use libafl_bolts::{
    core_affinity::Cores,
    rands::StdRand,
    shmem::{ShMemProvider as _, StdShMemProvider},
    tuples::tuple_list,
};
use libafl_fandango::{
    generator::FandangoGenerator, inputs::DerivationTree, mutators::AdvanceMutator,
};
use libafl_targets::{EDGES_MAP, MAX_EDGES_FOUND, libfuzzer_initialize, libfuzzer_test_one_input};
use mimalloc::MiMalloc;
use rand::{RngCore, SeedableRng as _, rngs::StdRng};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
struct Opt {
    #[arg(short, long, default_value = "./crashes")]
    objective_dir: PathBuf,
    #[arg(short, long, default_value = "./fuzzer_stats.json")]
    stats_file: PathBuf,
    #[arg(short, long)]
    stdout_file: Option<String>,
    #[arg(short, long, default_value = "1337")]
    broker_port: u16,
    #[arg(short, long, default_value = "0", value_parser = Cores::from_cmdline)]
    cores: Cores,
}

/// The main fn, `no_mangle` as it is a C main
#[unsafe(no_mangle)]
#[expect(clippy::missing_panics_doc, clippy::too_many_lines)]
pub extern "C" fn libafl_main() {
    let opt = Opt::parse();

    let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");

    let monitor = tuple_list!(
        OnDiskJsonMonitor::new(opt.stats_file, |_| true),
        MultiMonitor::new(|s| println!("{s}"))
    );

    let mut run_client = |state: Option<_>,
                          mut restarting_mgr: LlmpRestartingEventManager<_, _, _, _, _>,
                          _client_description| {
        let objective_dir = opt.objective_dir.clone();

        #[allow(static_mut_refs)] // only a problem on nightly
        let edges_observer = unsafe {
            HitcountsMapObserver::new(StdMapObserver::from_mut_ptr(
                "edges",
                EDGES_MAP.as_mut_ptr(),
                MAX_EDGES_FOUND,
            ))
            .track_indices()
        };

        let time_observer = TimeObserver::new("time");
        let map_feedback = MaxMapFeedback::new(&edges_observer);
        // let kpath_feedback = KPathFeedback::new(nonzero!(nonterminal_start::DISCRIMINANT));
        let calibration = CalibrationStage::new(&map_feedback);

        let mut feedback = feedback_or!(
            map_feedback,
            // kpath_feedback,
            TimeFeedback::new(&time_observer)
        );

        let mut objective = feedback_or_fast!(CrashFeedback::new(), TimeoutFeedback::new());

        let mut state = state.unwrap_or_else(|| {
            StdState::new(
                StdRand::new(),
                InMemoryCorpus::new(),
                OnDiskCorpus::new(objective_dir).unwrap(),
                &mut feedback,
                &mut objective,
            )
            .unwrap()
        });

        let mutator = AdvanceMutator::new(
            StdRng::seed_from_u64(state.rand_mut().next_u64()),
            (),
            "AdvanceMutator",
        );

        let mutational_stage = StdMutationalStage::new(mutator);

        let mut stages = tuple_list!(calibration, mutational_stage);

        let scheduler = StdScheduler::new();

        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

        let mut harness = |input: &DerivationTree<nonterminal_start>| {
            let linearized = WriteVisitor::new(Vec::new())
                .visit(input.node(), 0)
                .unwrap()
                .continue_value()
                .unwrap()
                .output();

            unsafe {
                libfuzzer_test_one_input(&linearized);
            }
            ExitKind::Ok
        };

        // Create the executor for an in-process function with one observer for edge coverage and one for the execution time
        let mut executor = InProcessExecutor::with_timeout(
            &mut harness,
            tuple_list!(edges_observer, time_observer),
            &mut fuzzer,
            &mut state,
            &mut restarting_mgr,
            Duration::new(10, 0),
        )?;

        let args: Vec<String> = env::args().collect();
        if unsafe { libfuzzer_initialize(&args) } == -1 {
            println!("Warning: LLVMFuzzerInitialize failed with -1");
        }

        if state.must_load_initial_inputs() {
            let sampler = StdRng::seed_from_u64(state.rand_mut().next_u64());
            let mut generator = FandangoGenerator::new(sampler, ());
            state.generate_initial_inputs(
                &mut fuzzer,
                &mut executor,
                &mut generator,
                &mut restarting_mgr,
                10,
            )?;
        }

        let iters = 1_000_000;
        fuzzer.fuzz_loop_for(
            &mut stages,
            &mut executor,
            &mut state,
            &mut restarting_mgr,
            iters,
        )?;

        restarting_mgr.on_restart(&mut state)?;

        Ok(())
    };

    match Launcher::builder()
        .shmem_provider(shmem_provider)
        .configuration(EventConfig::from_name("default"))
        .monitor(monitor)
        .run_client(&mut run_client)
        .cores(&opt.cores)
        .broker_port(opt.broker_port)
        .stdout_file(opt.stdout_file.as_deref())
        .build()
        .launch()
    {
        Ok(()) => (),
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Goodbye."),
        Err(err) => panic!("Failed to run launcher: {err:?}"),
    }
}
