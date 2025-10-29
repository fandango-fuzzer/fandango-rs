use crate::inputs::{DerivationTree, NodeCountMetadata};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use core::num::NonZeroUsize;
use core::ops::DerefMut;
use fandango::generation::{InPlaceGenerated, Sampler};
use fandango::typing::{AsNodeMut, Node};
use fandango::visitor::navigation::{Advance, CountNodes};
use fandango::visitor::{VisitableChildren, Visitor, VisitorMut};
use libafl::HasMetadata;
use libafl::corpus::CorpusId;
use libafl::mutators::{MutationResult, Mutator};
use libafl::state::HasRand;
use libafl_bolts::rands::Rand;
use libafl_bolts::{Error, Named};

pub struct AdvanceMutator<SM, G> {
    sampler: SM,
    generator: G,
    name: Cow<'static, str>,
}

impl<SM, G> AdvanceMutator<SM, G> {
    pub fn new(sampler: SM, generator: G, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            sampler,
            generator,
            name: name.into(),
        }
    }
}

impl<G, SM> Named for AdvanceMutator<SM, G> {
    fn name(&self) -> &Cow<'static, str> {
        &self.name
    }
}

impl<G, N, S, SM> Mutator<DerivationTree<N>, S> for AdvanceMutator<SM, G>
where
    N: Node,
    // boilerplate for CountNodes and Advance
    for<'a> N::TypeMut<'a>: InPlaceGenerated<SM, G>,
    SM: Sampler<N>,
    S: HasRand + HasMetadata,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut DerivationTree<N>,
    ) -> Result<MutationResult, Error> {
        let seed = state.rand_mut().next();
        self.sampler.reseed(seed);

        let metadata = state.metadata_map_mut().get_or_insert_with_boxed(|| {
            Box::new(NodeCountMetadata::new(
                NonZeroUsize::new(input.node_mut().count_nodes()).unwrap(),
            ))
        });
        let position = self.sampler.sample() % metadata.count();
        let mut node = input.node_mut();
        let mut selected = Advance::forward_ref(position)
            .visit_mut(node.deref_mut(), 0)
            .unwrap()
            .break_value()
            .unwrap();

        selected.generate_in_place(&mut self.sampler, &mut self.generator, 0);
        drop(selected);

        let count = metadata.count_mut();
        *count = NonZeroUsize::new(node.count_nodes()).unwrap();

        Ok(MutationResult::Mutated)
    }

    fn post_exec(&mut self, _state: &mut S, _new_corpus_id: Option<CorpusId>) -> Result<(), Error> {
        // nothing to do, no feedback
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::inputs::DerivationTree;
    use crate::mutators::AdvanceMutator;
    use alloc::boxed::Box;
    use core::error::Error;
    use fandango::Fandango;
    use fandango::generation::Generated;
    use libafl::corpus::NopCorpus;
    use libafl::mutators::Mutator;
    use libafl::state::StdState;
    use libafl_bolts::rands::StdRand;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[derive(Fandango)]
    #[fandango(grammar = "../tests/grammars/simple.fan", parse = false, serde = true)]
    #[allow(dead_code)]
    struct Simple;

    #[test]
    fn mutator_works() -> Result<(), Box<dyn Error>> {
        let mut state = StdState::new(
            StdRand::new(),
            NopCorpus::<DerivationTree<nonterminal_start>>::new(),
            NopCorpus::new(),
            &mut (),
            &mut (),
        )?;

        let mut rng = StdRng::seed_from_u64(0);

        let input = nonterminal_start::generate(&mut rng, &mut (), 0);
        let mut input = DerivationTree::new(input);

        let mut mutator = AdvanceMutator::new(rng, (), "test");

        for _ in 0..10_000 {
            mutator.mutate(&mut state, &mut input)?;
        }

        Ok(())
    }
}
