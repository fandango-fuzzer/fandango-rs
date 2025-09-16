// This file will be an attempt at using fandango-rs as a compiler tester. 

#[cfg(not(feature = "static_defs"))]
mod defs {
    use core::convert::Infallible;
    use fandango::Fandango;

    /// Base for the lang grammar stored in lang.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/lang.fan", parse = false, dynamic = true)]
    pub struct Lang(Infallible);
}

#[cfg(feature = "static_defs")]
mod defs {
    use crate::Checker;
    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use core::convert::Infallible;
    use core::mem;
    use core::ops::ControlFlow;
    use fandango::Fandango;
    use fandango::generation::Generated;
    use fandango::typing::{AsNodeMut, AsNodeRef, ChildAccessor, Node, Nth};
    use fandango::visitor::{VisitResult, VisitableChildren, Visitor};
    
    /// Base for the lang grammar stored in lang.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/lang.fan", parse = false)]
    pub struct Lang(Infallible);
    
    /// A visitor which collects the violations of the constraints in the Lang grammar.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitor<const CORRECT: bool> {
        path: VecDeque<usize>,
        violations: Vec<VecDeque<usize>>,
    }
    
    impl ConstraintVisitor<true> {
        /// Construct this visitor in the form that produces correctly formatted data.
        pub fn corrected() -> Self {
            Self::default()
        }
    }
    
    impl<const FIXED: bool> Checker for ConstraintVisitor<FIXED> {
        fn violations(self) -> Vec<VecDeque<usize>> {
            self.violations
        }
    }

    // Let's try to make a constraint to... make sure that all the variable names are ... "x"?
    impl<T> Visitor<T> for ConstraintVisitor<true>
    where
        T: VisitableChildren<T> + AsNodeRef<nonterminal_var_name>,
    {
        type Continue = Self;
        type Break = Infallible;
        type Error = Infallible;

        fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
        where
            N: Node<TypeMut<'program> = T>,
            T: From<&'program mut N> + AsNodeMut<N>,
        {
            self.path.push_back(idx);
            let visited = T::from(node);
            if let Some(tree) = visited.as_node()
                && let Some(seq) = tree.nth::<0>().nth::<0>()
            {
                if seq.as_str() != "x" {
                    self.violations.push(self.path.clone());
                }                
            }
            let result = visited.visit_each(self);
            let Ok(ControlFlow::Continue(mut visitor)) = result;
            visitor.path.pop_back();
            Ok(ControlFlow::Continue(visitor))
        }
    }

    impl<T> Visitor<T> for ConstraintVisitor<false> {
        type Continue = Self;
        type Break = Infallible;
        type Error = Infallible;

        fn visit<'program, N>(self, _node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
        where
            N: Node<TypeMut<'program> = T>,
            T: From<&'program mut N> + AsNodeMut<N>,
        {
            Ok(ControlFlow::Continue(self)) // lang constraints are trivially true
        }
    }

    
    /// A visitor which applies fixes based on the constraints in the lang grammar.
    #[derive(Debug)]
    pub struct ConstraintFixer<'a, S, G, const CORRECT: bool> {
        sampler: &'a mut S,
        generator: &'a mut G,
    }
    
    impl<'a, S, G> ConstraintFixer<'a, S, G, true> {
        /// Construct this fixer in the form that ensures the correctness of generated inputs.
        pub fn corrected(sampler: &'a mut S, generator: &'a mut G) -> Self {
            Self { sampler, generator }
        }
    }
    
    //
    // checked to here
    //

    impl<'a, S, G, T> Visitor<T> for ConstraintFixer<'a, S, G, true>
    where
        nonterminal_var_name: Generated<S, G>,
        T: VisitableChildren<T> + AsNodeMut<nonterminal_var_name>,
    {
        type Continue = Self;
        type Break = Infallible;
        type Error = Infallible;

        fn visit<'program, N>(self, node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
        where
            N: Node<TypeMut<'program> = T>,
            T: From<&'program mut N> + AsNodeMut<N>,
        {
            let mut visited = T::from(node);
            if let Some(tree) = AsNodeMut::<nonterminal_var_name>::as_node_mut(&mut visited) {
                if let Some(seq) = tree.nth::<0>().nth::<0>() {
                    if seq.as_str() != "x" {
                        let new_name = nonterminal_var_name::generate(self.sampler, self.generator, 0);
                        *tree = new_name;
                    }
                }
                return Ok(ControlFlow::Continue(self)); // terminate; we have completed the fix
            }
            visited.visit_each(self)
        }
    }

    impl<'a, S, G, T> Visitor<T> for ConstraintFixer<'a, S, G, false> {
        type Continue = Self;
        type Break = Infallible;
        type Error = Infallible;

        fn visit<'program, N>(self, _node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
        where
            N: Node<TypeMut<'program> = T>,
            T: From<&'program mut N> + AsNodeMut<N>,
        {
            Ok(ControlFlow::Continue(self)) // ...lang constraints are trivially true?
        }
    }

    #[cfg(test)]
    mod test {
        use crate::lang;
        use crate::operators::DepthLimiter;
        use alloc::boxed::Box;
        use core::error::Error;
        use core::ops::ControlFlow;
        use fandango::generation::Generated;
        use fandango::tuple_list::tuple_list;
        use fandango::typing::{ChildAccessor, Nth, Structured};
        use fandango::visitor::Visitor;
        use fandango::visitor::navigation::GoTo;
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        #[test]
        fn check_constraint() -> Result<(), Box<dyn Error>> {
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(lang::nonterminal_start::ROOT.inner(), 50));
            let mut diff_count = 0;
            for _ in 0..100_000 {
                let mut tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
                let Ok(ControlFlow::Continue(lang::ConstraintVisitor { violations, .. })) =
                    lang::ConstraintVisitor::corrected().visit(&mut tree, 0);

                for mut violation in violations {
                    violation.pop_front();
                    assert!(matches!(
                        tree.go_to(0, violation.clone())?,
                        lang::TypeMut::nonterminal_lang_string_list(_)
                    ));
                    let len = violation.len();

                    violation.truncate(len - 8);

                    let lang::TypeMut::nonterminal_var_name(records) =
                        tree.go_to(0, violation)?
                    else {
                        unreachable!("We are inspecting the records directly.");
                    };
                    let base_list = records
                        .child()
                        .nth::<0>()
                        .expect("Must be present for a violation to be reported")
                        .nth::<0>()
                        .child()
                        .nth::<0>();
                    let cmp_list = records
                        .child()
                        .nth::<0>()
                        .expect("Must be present for a violation to be reported")
                        .nth::<1>()
                        .child()
                        .nth::<0>()
                        .expect("Must be present for a violation to be reported")
                        .nth::<0>()
                        .child()
                        .nth::<0>();

                    assert_ne!(lang::count_fields(base_list), lang::count_fields(cmp_list));

                    diff_count += 1;
                }

                let _ = lang::ConstraintFixer::corrected(&mut rng, &mut ()).visit(&mut tree, 0)?;
                let ControlFlow::Continue(lang::ConstraintVisitor { violations, .. }) =
                    lang::ConstraintVisitor::corrected().visit(&mut tree, 0)?;
                assert_eq!(0, violations.len());
            }
            assert_ne!(0, diff_count);
            Ok(())
        }
    }
}

pub use defs::*;