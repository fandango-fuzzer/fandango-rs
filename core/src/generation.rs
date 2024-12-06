use crate::type_eq::type_eq;
use crate::typing::Node;
use std::convert::Infallible;
use std::mem;
use std::mem::ManuallyDrop;

pub trait Generator<R> {
    type Error;
    type Supported;

    fn generate(&mut self, rng: &mut R) -> Result<Self::Supported, Self::Error>;
}

pub trait GeneratorTuple<R> {
    type Error;

    fn generate<N>(&mut self, rng: &mut R) -> Result<N, Self::Error>
    where
        N: Node;
}

impl<R> GeneratorTuple<R> for () {
    type Error = Infallible;

    fn generate<N>(&mut self, rng: &mut R) -> Result<N, Self::Error>
    where
        N: Node,
    {
        todo!()
    }
}

impl<Head, Tail, R> GeneratorTuple<R> for (Head, Tail)
where
    Head: Generator<R>,
    Tail: GeneratorTuple<R>,
    Tail::Error: Into<Head::Error>,
{
    type Error = Head::Error;

    fn generate<N>(&mut self, rng: &mut R) -> Result<N, Self::Error>
    where
        N: Node,
    {
        if type_eq::<N, Head::Supported>() {
            let generated = self.0.generate(rng)?;
            // Rust doesn't understand type_eq, so we have to rely on transmute_copy
            // This should be perfectly safe:
            //  - size and alignment are preserved because N == Head::Supported
            //  - destructors are not run because of `ManuallyDrop`
            //  - lifetimes are consistent between the supported and specified node type
            // I'm fairly confident the compiler is smart enough to skip the copy, too
            let generated = unsafe { mem::transmute_copy(&ManuallyDrop::new(generated)) };
            Ok(generated)
        } else {
            self.1.generate(rng).map_err(Into::into)
        }
    }
}
