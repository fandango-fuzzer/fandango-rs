use crate::feedbacks::GlobalKPathMetadata;
use crate::inputs::{DerivationTree, KPathReducer};
use alloc::collections::{BTreeMap, VecDeque};
use core::ops::{Deref, DerefMut};
use fandango::typing::Node;
use fandango::visitor::Visitor;
use fandango::visitor::kpath::KPathVisit;
use libafl::HasMetadata;
use libafl::corpus::{Corpus, CorpusId, HasTestcase};
use libafl::schedulers::Scheduler;
use libafl::state::HasCorpus;
use libafl_bolts::{Error, impl_serdeany};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct KPathWeightMetadata {
    best: BTreeMap<usize, VecDeque<CorpusId>>,
}

impl_serdeany!(KPathWeightMetadata);

pub struct KPathScheduler;

impl<N, S> Scheduler<DerivationTree<N>, S> for KPathScheduler
where
    N: Node,
    S: HasMetadata + HasTestcase<DerivationTree<N>> + HasCorpus<DerivationTree<N>>,
{
    fn on_add(&mut self, state: &mut S, id: CorpusId) -> Result<(), Error> {
        let mut testcase = state.testcase_mut(id)?;
        let tree = testcase.input_mut().as_mut().unwrap();

        let kpaths = state.metadata::<GlobalKPathMetadata>()?.kpaths();
        let lowest = KPathVisit::new(kpaths, KPathReducer::min())
            .visit(tree.node().deref(), 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .value();

        drop(testcase);

        let metadata = state.metadata_or_insert_with(|| KPathWeightMetadata {
            best: BTreeMap::new(),
        });

        metadata.best.entry(lowest).or_default().push_back(id);

        Ok(())
    }

    fn next(&mut self, state: &mut S) -> Result<CorpusId, Error> {
        let id = state
            .metadata_mut::<KPathWeightMetadata>()?
            .best
            .first_entry()
            .unwrap()
            .get_mut()
            .pop_front()
            .unwrap();
        let mut testcase = state.testcase_mut(id)?;
        state.corpus().load_input_into(&mut testcase)?;

        let kpaths = state.metadata::<GlobalKPathMetadata>()?.kpaths();
        let lowest = KPathVisit::new(kpaths, KPathReducer::min())
            .visit(testcase.input().as_ref().unwrap().node(), 0)
            .unwrap()
            .continue_value()
            .unwrap()
            .value();

        drop(testcase);

        let metadata = state.metadata_mut::<KPathWeightMetadata>()?;
        metadata.best.entry(lowest).or_default().push_back(id);

        let first = metadata.best.first_entry().unwrap();
        if first.get().is_empty() {
            first.remove();
        }
        self.set_current_scheduled(state, Some(id))?;

        Ok(id)
    }

    fn set_current_scheduled(
        &mut self,
        state: &mut S,
        next_id: Option<CorpusId>,
    ) -> Result<(), Error> {
        *state.corpus_mut().current_mut() = next_id;
        Ok(())
    }
}
