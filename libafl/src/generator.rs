use fandango::generation::Generated;
use libafl::{generators::Generator, state::HasRand};

use crate::inputs::DerivationTree;

/// Generator for `DerivationTree`s.
pub struct FandangoGenerator<SM, G> {
    sampler: SM,
    generator: G,
}

impl<SM, G> FandangoGenerator<SM, G> {
    /// Creates a new `FandangoGenerator`
    #[must_use]
    pub fn new(sampler: SM, generator: G) -> Self {
        Self { sampler, generator }
    }
}

impl<N, S, SM, G> Generator<DerivationTree<N>, S> for FandangoGenerator<SM, G>
where
    N: Generated<SM, G>,
    S: HasRand,
{
    fn generate(&mut self, _state: &mut S) -> Result<DerivationTree<N>, libafl::Error> {
        Ok(DerivationTree::new(N::generate(
            &mut self.sampler,
            &mut self.generator,
            0,
        )))
    }
}

#[cfg(test)]
mod test {
    use fandango::Fandango;
    use libafl::{corpus::NopCorpus, generators::Generator, state::StdState};
    use libafl_bolts::rands::StdRand;
    use rand::{SeedableRng as _, rngs::StdRng};

    use crate::{generator::FandangoGenerator, inputs::DerivationTree};

    #[derive(Fandango)]
    #[fandango(grammar = "../tests/grammars/simple.fan", parse = false, serde = true)]
    #[allow(dead_code)]
    struct Simple;

    #[test]
    fn generator_works() {
        let sampler = StdRng::from_seed([0; 32]);
        let mut generator = FandangoGenerator::new(sampler, ());
        let mut state = StdState::<_, DerivationTree<nonterminal_start>, _, _>::new(
            StdRand::new(),
            NopCorpus::new(),
            NopCorpus::new(),
            &mut (),
            &mut (),
        )
        .unwrap();
        let _generated: DerivationTree<nonterminal_start> = generator.generate(&mut state).unwrap();
    }
}
