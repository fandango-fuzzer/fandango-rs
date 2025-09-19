pub mod basic;

use crate::population::Individual;
use ::fandango::typing::Node;

pub trait Evolver<I, G, S> {
    type Population;

    type Error;

    fn initial(
        &mut self,
        generators: &mut G,
        sampler: &mut S,
    ) -> Result<Self::Population, Self::Error>;
    fn step(
        &mut self,
        generators: &mut G,
        sampler: &mut S,
        population: Self::Population,
    ) -> Result<Self::Population, Self::Error>;
}
