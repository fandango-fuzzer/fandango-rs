//! A libafl example fuzzer

use core::time::Duration;
use std::{env, path::PathBuf};

use clap::Parser;
use libafl::{
    Error,
    corpus::{Corpus, InMemoryCorpus, OnDiskCorpus},
    events::{EventConfig, EventRestarter, Launcher, LlmpRestartingEventManager},
    executors::{ExitKind, inprocess::InProcessExecutor},
    feedback_or, feedback_or_fast,
    feedbacks::{CrashFeedback, MaxMapFeedback, TimeFeedback, TimeoutFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    inputs::{BytesInput, HasTargetBytes},
    monitors::{MultiMonitor, OnDiskJsonMonitor},
    mutators::{havoc_mutations::havoc_mutations, scheduled::HavocScheduledMutator},
    observers::{CanTrack, HitcountsMapObserver, StdMapObserver, TimeObserver},
    schedulers::{
        IndexesLenTimeMinimizerScheduler, StdWeightedScheduler, powersched::PowerSchedule,
    },
    stages::{calibrate::CalibrationStage, power::StdPowerMutationalStage},
    state::{HasCorpus, StdState},
};
use libafl_bolts::{
    AsSlice,
    core_affinity::Cores,
    rands::StdRand,
    shmem::{ShMemProvider as _, StdShMemProvider},
    tuples::tuple_list,
};
use libafl_targets::{EDGES_MAP, MAX_EDGES_FOUND, libfuzzer_initialize, libfuzzer_test_one_input};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
struct Opt {
    #[arg(short, long, default_value = "./corpus")]
    corpus_dir: PathBuf,
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

        let corpus_dir = opt.corpus_dir.clone();

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
        let calibration = CalibrationStage::new(&map_feedback);

        let mut feedback = feedback_or!(map_feedback, TimeFeedback::new(&time_observer));

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

        let mutator = HavocScheduledMutator::new(havoc_mutations());

        let power: StdPowerMutationalStage<_, _, BytesInput, _, _, _> =
            StdPowerMutationalStage::new(mutator);

        let mut stages = tuple_list!(calibration, power);

        let scheduler = IndexesLenTimeMinimizerScheduler::new(
            &edges_observer,
            StdWeightedScheduler::with_schedule(
                &mut state,
                &edges_observer,
                Some(PowerSchedule::fast()),
            ),
        );

        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

        let mut harness = |input: &BytesInput| {
            let target = input.target_bytes();
            let buf = target.as_slice();
            unsafe {
                libfuzzer_test_one_input(buf);
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
            state
                .load_initial_inputs(
                    &mut fuzzer,
                    &mut executor,
                    &mut restarting_mgr,
                    std::slice::from_ref(&corpus_dir),
                )
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to load initial corpus at {:?}",
                        &corpus_dir.display()
                    )
                });
            println!("We imported {} inputs from disk.", state.corpus().count());
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
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {err:?}"),
    }
}
