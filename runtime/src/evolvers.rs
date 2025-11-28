//! Evolvers which can act on [`fandango`]-specified grammars

pub mod basic;
pub mod multi;

/// Core trait over which evolution is performed
pub trait Evolver<G, S> {
    /// The population handled by this evolver (most likely a [`alloc::vec::Vec`])
    type Population;

    /// The error which is emitted in the case that anything failed
    type Error;

    /// Generate an initial population of inputs
    ///
    /// Note that you do not need to use the same generators or sampler as on the [`Evolver::step`]
    /// function.
    fn initial(
        &mut self,
        generators: &mut G,
        sampler: &mut S,
    ) -> Result<Self::Population, Self::Error>;

    /// Take the current population and produce a new population from it
    ///
    /// Note that it is also not necessary to use the same generator and sampler every time, but
    /// changing them may have unexpected impacts on the evolution strategy.
    fn step(
        &mut self,
        generators: &mut G,
        sampler: &mut S,
        population: Self::Population,
    ) -> Result<Self::Population, Self::Error>;
}

pub trait PopulationEvaluator<G, S, I>: Evolver<G, S> {
    fn evaluate_population(&mut self, to_evaluate: &[I]) -> Result<Self::Population, Self::Error>;
}
