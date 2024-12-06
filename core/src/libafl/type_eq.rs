use std::cell::Cell;
use std::marker::PhantomData;

/// Returns if the type `T` is equal to `U`, ignoring lifetimes.
#[inline] // this entire call gets optimized away :)
#[must_use]
pub fn type_eq<T: ?Sized, U: ?Sized>() -> bool {
    // decider struct: hold a cell (which we will update if the types are unequal) and some
    // phantom data using a function pointer to allow for Copy to be implemented
    struct W<'a, T: ?Sized, U: ?Sized>(&'a Cell<bool>, PhantomData<fn() -> (&'a T, &'a U)>);

    // default implementation: if the types are unequal, we will use the clone implementation
    impl<T: ?Sized, U: ?Sized> Clone for W<'_, T, U> {
        #[inline]
        fn clone(&self) -> Self {
            // indicate that the types are unequal
            // unfortunately, use of interior mutability (Cell) makes this not const-compatible
            // not really possible to get around at this time
            self.0.set(false);
            W(self.0, self.1)
        }
    }

    // specialized implementation: Copy is only implemented if the types are the same
    #[allow(clippy::mismatching_type_param_order)]
    impl<T: ?Sized> Copy for W<'_, T, T> {}

    let detected = Cell::new(true);
    // [].clone() is *specialized* in core.
    // Types which implement copy will have their copy implementations used, falling back to clone.
    // If the types are the same, then our clone implementation (which sets our Cell to false)
    // will never be called, meaning that our Cell's content remains true.
    let res = [W::<T, U>(&detected, PhantomData)].clone();
    res[0].0.get()
}
