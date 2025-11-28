use core::{fmt::Debug, marker::PhantomData};

use alloc::{format, vec::Vec};
use fandango::typing::Node;
use fandango_runtime::evolvers::PopulationEvaluator;
use libafl::{
    Error, Evaluator,
    mutators::Mutator,
    stages::{Restartable, Stage},
    state::{HasCurrentTestcase, HasRand},
};

use crate::inputs::DerivationTree;

pub struct FandangoEvolutionStage<EV, G, SM, MT, N> {
    phantom: PhantomData<N>,
    mutator: MT,
    evolver: EV,
    generator: G,
    sampler: SM,
    population_size_from_mutation: usize,
    evolution_steps: usize,
}

impl<EV, G, SM, MT, N> FandangoEvolutionStage<EV, G, SM, MT, N> {
    pub fn new(
        mutator: MT,
        evolver: EV,
        generator: G,
        sampler: SM,
        population_size_from_mutation: usize,
        evolution_steps: usize,
    ) -> Self {
        Self {
            phantom: PhantomData,
            mutator,
            evolver,
            generator,
            sampler,
            population_size_from_mutation,
            evolution_steps,
        }
    }
}

impl<E, EM, EV, G, SM, MT, N, S, Z> Stage<E, EM, S, Z> for FandangoEvolutionStage<EV, G, SM, MT, N>
where
    S: HasRand + HasCurrentTestcase<DerivationTree<N>>,
    MT: Mutator<DerivationTree<N>, S>,
    Z: Evaluator<E, EM, DerivationTree<N>, S>,
    EV: PopulationEvaluator<G, SM, DerivationTree<N>>,
    EV::Error: Debug,
    N: Clone + Node,
{
    fn perform(
        &mut self,
        fuzzer: &mut Z,
        executor: &mut E,
        state: &mut S,
        manager: &mut EM,
    ) -> Result<(), libafl::Error> {
        let testcase = state.current_testcase_mut()?;
        let input = testcase
            .input()
            .clone()
            .ok_or_else(|| Error::unknown("No input found"))?;
        drop(testcase);

        let mutated_population = (0..self.population_size_from_mutation)
            .map(|_| {
                let mut input = input.clone();
                self.mutator.mutate(state, &mut input)?;
                Ok(input)
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let mut population = self
            .evolver
            .evaluate_population(&mutated_population)
            .map_err(|e| Error::unknown(format!("Failed to evaluate population: {e:?}")))?;

        for _ in 0..self.evolution_steps {
            population = self
                .evolver
                .step(&mut self.generator, &mut self.sampler, population)
                .map_err(|e| Error::unknown(format!("Failed to evolve population: {e:?}")))?;
        }

        for mutated in mutated_population {
            let (_, corpus_id) = fuzzer.evaluate_filtered(state, executor, manager, &mutated)?;
        }

        Ok(())
    }
}

impl<EV, G, SM, MT, N, S> Restartable<S> for FandangoEvolutionStage<EV, G, SM, MT, N> {
    fn should_restart(&mut self, state: &mut S) -> Result<bool, Error> {
        Ok(true)
    }

    fn clear_progress(&mut self, state: &mut S) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use core::time::Duration;

    use fandango::tuple_list::tuple_list;
    use fandango_runtime::{evolvers::basic::BasicEvolver, measurement::ViolationFitness};
    use fandango_targets::xml;
    use libafl::{
        Fuzzer as _, StdFuzzer,
        corpus::{InMemoryCorpus, NopCorpus},
        events::SimpleEventManager,
        executors::{ExitKind, nop::NopExecutor},
        feedbacks::ConstFeedback,
        monitors::SimpleMonitor,
        mutators::HavocScheduledMutator,
        schedulers::StdScheduler,
        stages::Stage,
        state::{HasRand, StdState},
    };
    use libafl_bolts::rands::StdRand;
    use num_rational::Ratio;
    use rand::{RngCore as _, SeedableRng, rngs::StdRng};

    use crate::{
        generator::FandangoGenerator, inputs::DerivationTree, mutators::AdvanceMutator,
        stage::FandangoEvolutionStage,
    };

    #[test]
    #[expect(clippy::similar_names)]
    fn test_fandango_evolution_stage() {
        let mut feedback = ConstFeedback::new(true);

        let mut state = StdState::new(
            StdRand::new(),
            InMemoryCorpus::<DerivationTree<xml::nonterminal_start>>::new(),
            NopCorpus::new(),
            &mut feedback,
            &mut (),
        )
        .unwrap();

        let sampler = StdRng::seed_from_u64(state.rand_mut().next_u64());
        let mut generator = FandangoGenerator::new(sampler, ());
        let fitness = ViolationFitness::<xml::ConstraintVisitor>::new();

        let fixer = ();
        let evolver = BasicEvolver::new::<xml::nonterminal_start>(
            fitness,
            fixer,
            100,
            10,
            1000,
            Ratio::new(50, 100),
        )
        .expect("Should be valid.");

        let sampler = StdRng::seed_from_u64(state.rand_mut().next_u64());

        let mutator =
            HavocScheduledMutator::new(tuple_list!(AdvanceMutator::new(sampler, (), "test")));

        let sampler = StdRng::seed_from_u64(state.rand_mut().next_u64());
        let stage = FandangoEvolutionStage::new(mutator, evolver, (), sampler, 100, 10);

        let mut executor = NopExecutor::new(ExitKind::Ok, Duration::from_micros(10), ());
        let mut manager = SimpleEventManager::new(SimpleMonitor::new(|_| {}));
        let scheduler = StdScheduler::new();

        let mut fuzzer = StdFuzzer::new(scheduler, feedback, ());

        state
            .generate_initial_inputs(&mut fuzzer, &mut executor, &mut generator, &mut manager, 1)
            .unwrap();
        fuzzer
            .fuzz_one(
                &mut tuple_list!(stage),
                &mut executor,
                &mut state,
                &mut manager,
            )
            .unwrap();
    }
}
