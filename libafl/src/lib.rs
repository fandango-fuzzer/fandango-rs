#![no_std]

use core::fmt::Debug;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use fandango::typing::Node;
use libafl::inputs::Input;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct DerivationTree<N> {
    node: N,
}

impl<N> Input for DerivationTree<N> where N: Debug + Clone + Hash + Serialize + Deserialize {}

pub trait NodeChooser<N, R>
where
    N: Node,
{
    fn choose(&self, node: &mut N, rng: &mut R) -> N::TypeMut;
}

pub struct NodeMutator<C, S, G> {
    chooser: C,
    sampler: S,
    generator: G,
}

pub trait LibaflSampler {}
