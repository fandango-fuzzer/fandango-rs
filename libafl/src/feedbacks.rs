use crate::inputs::DerivationTree;
use alloc::borrow::Cow;
use core::convert::Infallible;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ops::{ControlFlow, DerefMut};
use fandango::typing::{AsNodeMut, DiscriminantLookup, Node, Structured};
use fandango::visitor::kpath::{KPathUpdate, KPathVisit, KPathVisitor, KPaths};
use fandango::visitor::{VisitableChildren, Visitor};
use libafl::HasMetadata;
use libafl::corpus::Testcase;
use libafl::executors::ExitKind;
use libafl::feedbacks::{Feedback, StateInitializer};
use libafl::state::HasCorpus;
use libafl_bolts::{Error, Named, impl_serdeany};
use mappable_rc::Mrc;
use serde::{Deserialize, Serialize};

pub struct KPathFeedback<N> {
    k: NonZeroUsize,
    phantom: PhantomData<N>,
}

impl<N> KPathFeedback<N> {
    pub fn new(k: NonZeroUsize) -> Self {
        Self {
            k,
            phantom: PhantomData,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalKPathMetadata {
    kpaths: KPaths,
}

impl GlobalKPathMetadata {
    pub fn kpaths(&self) -> &KPaths {
        &self.kpaths
    }

    pub fn kpaths_mut(&mut self) -> &mut KPaths {
        &mut self.kpaths
    }
}

impl_serdeany!(GlobalKPathMetadata);

impl<N, S> StateInitializer<S> for KPathFeedback<N>
where
    N: Node + Structured + 'static,
    S: HasMetadata,
{
    fn init_state(&mut self, state: &mut S) -> Result<(), Error> {
        let kpaths = KPaths::new::<N::TypeMut<'static>>(self.k, N::ROOT.inner());
        state.add_metadata(GlobalKPathMetadata { kpaths });
        Ok(())
    }
}

impl<N> Named for KPathFeedback<N> {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("kpath");
        &NAME
    }
}

struct ZeroKPathVisitor;

impl KPathVisitor for ZeroKPathVisitor {
    type Value = ();
    type Break = ();
    type Error = Infallible;

    fn visit_path(
        &mut self,
        count: usize,
        _path: &Mrc<[usize]>,
    ) -> Result<ControlFlow<Self::Break>, Self::Error> {
        if count == 0 {
            Ok(ControlFlow::Break(()))
        } else {
            Ok(ControlFlow::Continue(()))
        }
    }

    fn value(self) -> Self::Value {}
}

impl<EM, N, OT, S> Feedback<EM, DerivationTree<N>, OT, S> for KPathFeedback<N>
where
    N: Node + Structured + 'static,
    S: HasMetadata + HasCorpus<DerivationTree<N>>,
    for<'a> N::TypeMut<'a>:
        DiscriminantLookup + From<&'a mut N> + AsNodeMut<N> + VisitableChildren<N::TypeMut<'a>>,
{
    fn is_interesting(
        &mut self,
        state: &mut S,
        _manager: &mut EM,
        input: &DerivationTree<N>,
        _observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error> {
        let kpaths = &state.metadata::<GlobalKPathMetadata>()?.kpaths;

        Ok(KPathVisit::new(kpaths, ZeroKPathVisitor)
            .visit(input.node(), 0)
            .unwrap()
            .is_break())
    }

    fn append_metadata(
        &mut self,
        state: &mut S,
        _manager: &mut EM,
        _observers: &OT,
        testcase: &mut Testcase<DerivationTree<N>>,
    ) -> Result<(), Error> {
        let kpaths = &mut state.metadata_mut::<GlobalKPathMetadata>()?.kpaths;

        // guaranteed success
        let _ = KPathUpdate::inserting(kpaths).visit(
            testcase
                .input_mut()
                .as_mut()
                .unwrap()
                .node_mut()
                .deref_mut(),
            0,
        );

        Ok(())
    }
}
