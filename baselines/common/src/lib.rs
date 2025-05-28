//! Common definitions for benchmarking the baselines.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use fandango::lang::{FandangoNode, Program};
use fandango::tuple_list::tuple_list_type;
use fandango_targets::operators::DepthLimiter;
use hashbrown::HashMap;
use rand::rngs::StdRng;

/// The sampler to be used throughout the evaluation.
pub type StdSampler = StdRng;
/// The generator to be used throughout the evaluation.
pub type StdGenerator =
    tuple_list_type!(DepthLimiter<HashMap<FandangoNode<'static, 'static>, Vec<usize>>>);

/// A benchmark suite, but only the dynamic bits.
pub trait DynamicBenchmarkSuite {
    /// The name to associate with this benchmark.
    const NAME: &'static str;

    /// The static [`Program`] node which represents the grammar.
    fn program() -> &'static Program<'static>;
}

#[cfg(feature = "static_defs")]
mod static_defs {
    use crate::DynamicBenchmarkSuite;
    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use fandango::dynamic::{DynamicNode, DynamicSampler};

    /// Trait which each benchmark target will implement, for consistency.
    pub trait BenchmarkSuite<S, G>: DynamicBenchmarkSuite {
        /// The start node type (often, `nonterminal_start`) used by the benchmark.
        type Start;

        /// Generate a start node.
        fn generate(sampler: &mut S, generator: &mut G) -> Self::Start;

        /// Fix a given start node.
        ///
        /// Sampler and generator is provided to allow for mutation-based fixing.
        fn fix(item: &mut Self::Start, sampler: &mut S, generator: &mut G);

        /// Check a given start node's constraints and return any violations as paths.
        fn check(item: &mut Self::Start) -> Vec<VecDeque<usize>>;

        /// Mutate the given start node at the given points, where possible.
        fn mutate(
            item: &mut Self::Start,
            choices: &mut Vec<VecDeque<usize>>,
            sampler: &mut S,
            generator: &mut G,
        ) -> bool;

        /// Crossover the given start node at the given points with the provided base.
        fn crossover(
            item: &mut Self::Start,
            other: &mut Self::Start,
            choices: &mut Vec<VecDeque<usize>>,
            sampler: &mut S,
        ) -> bool;

        /// Crossover the given dynamic node at the given points with the provided base.
        ///
        /// Since the crossover targets are specific to each benchmark, we split these out.
        fn crossover_dynamic(
            item: &mut DynamicNode,
            other: &mut DynamicNode,
            choices: &mut Vec<VecDeque<usize>>,
            sampler: &mut DynamicSampler<S>,
        ) -> bool;
    }
}

#[cfg(feature = "static_defs")]
pub use static_defs::*;
