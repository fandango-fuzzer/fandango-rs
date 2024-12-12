use crate::generation::SpecificGenerator;
use crate::graph::{FandangoNode, IntoGraph};
use crate::typing::{Node, Structured};
use pest::Span;
use petgraph::graphmap::DiGraphMap;
use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Unflattenable;

impl Display for Unflattenable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("The structure could not be flattened.")
    }
}

impl Error for Unflattenable {}

pub struct Flattener<N> {
    choices: Vec<usize>,
    phantom: PhantomData<N>,
}

fn alternatives_within(
    graph: &DiGraphMap<FandangoNode<'static, 'static>, Span<'static>>,
    node: FandangoNode<'static, 'static>,
    alt_queue: &mut VecDeque<FandangoNode<'static, 'static>>,
    visited: &mut HashSet<FandangoNode<'static, 'static>>,
) -> Result<(), Unflattenable> {
    todo!()
}

impl<N> Flattener<N>
where
    N: Structured<FandangoType = FandangoNode<'static, 'static>>,
{
    pub fn new() -> Result<Self, Unflattenable> {
        if !matches!(N::STRUCTURE.inner(), FandangoNode::Alternative(_)) {
            return Err(Unflattenable);
        }

        let graph = N::STRUCTURE.inner().into_graph();

        todo!()
    }
}

impl<N, W, S> SpecificGenerator<W, S> for Flattener<N> {
    type Generated = N;

    fn generate(&mut self, with: &mut W, sampler: &mut S) -> Option<Self::Generated> {
        todo!()
    }
}
