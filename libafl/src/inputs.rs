use core::convert::Infallible;
use core::fmt::Debug;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ops::ControlFlow;
use fandango::visitor::kpath::KPathVisitor;
use libafl::inputs::Input;
use libafl_bolts::impl_serdeany;
use libafl_bolts::simd::{MaxReducer, MinReducer, Reducer};
use mappable_rc::Mrc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationTree<N> {
    node: N,
}

impl<N> Hash for DerivationTree<N>
where
    N: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state);
    }
}

impl<N> DerivationTree<N> {
    pub fn new(node: N) -> Self {
        Self { node }
    }

    pub fn node(&self) -> &N {
        &self.node
    }

    pub fn node_mut(&mut self) -> &mut N {
        &mut self.node
    }
}

impl<N> Input for DerivationTree<N> where
    N: Debug + Clone + Hash + Serialize + for<'a> Deserialize<'a>
{
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct NodeCountMetadata {
    count: NonZeroUsize,
}

impl NodeCountMetadata {
    pub fn count(&self) -> NonZeroUsize {
        self.count
    }

    pub fn count_mut(&mut self) -> &mut NonZeroUsize {
        &mut self.count
    }

    pub fn new(count: NonZeroUsize) -> Self {
        Self { count }
    }
}

impl_serdeany!(NodeCountMetadata);

pub struct KPathReducer<R> {
    value: usize,
    phantom: PhantomData<R>,
}

impl<R> KPathReducer<R> {
    fn new(value: usize) -> Self {
        Self {
            value,
            phantom: PhantomData,
        }
    }
}

impl KPathReducer<MinReducer> {
    pub fn min() -> Self {
        Self::new(usize::MAX)
    }
}

impl KPathReducer<MaxReducer> {
    pub fn max() -> Self {
        Self::new(0)
    }
}

impl<R> KPathVisitor for KPathReducer<R>
where
    R: Reducer<usize>,
{
    type Value = usize;
    type Break = Infallible;
    type Error = Infallible;

    fn visit_path(
        &mut self,
        count: usize,
        _path: &Mrc<[usize]>,
    ) -> Result<ControlFlow<Self::Break>, Self::Error> {
        self.value = R::reduce(self.value, count);
        Ok(ControlFlow::Continue(()))
    }

    fn value(self) -> Self::Value {
        self.value
    }
}
