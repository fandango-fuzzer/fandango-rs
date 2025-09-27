//! Here, we define the constraints for the csv.fan grammar, namely:
//! ```text,ignore
//! forall <r1> in <csv_record>:
//!     forall <r2> in <csv_record>:
//!         |<r1>.<csv_string_list>.<raw_field>| == |<r2>.<csv_string_list>.<raw_field>|
//! ;
//! ```
//!
//! Note that the definition here is erroneous and only counts the first field, making this
//! constraint trivially tautological.

#[cfg(not(feature = "static_defs"))]
mod defs {
    use core::convert::Infallible;
    use fandango::Fandango;

    /// Base for the CSV grammar stored in csv.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/csv.fan", parse = false, dynamic = true)]
    pub struct Csv(Infallible);
}

#[cfg(feature = "static_defs")]
mod defs {
    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use core::convert::Infallible;
    use core::mem;
    use core::ops::ControlFlow;
    use fandango::Fandango;
    use fandango::generation::Generated;
    use fandango::typing::{
        AsNodeMut, AsNodeRef, ChildAccessor, Downcast, Node, Nth, Opaque, OpaqueMut,
    };
    use fandango::visitor::{
        VisitMutResult, VisitResult, VisitableChildren, VisitableChildrenMut, Visitor, VisitorMut,
    };
    use fandango_runtime::measurement::Violations;
    use fandango_runtime::operators::Checker;
    use num_rational::Ratio;

    /// Base for the CSV grammar stored in csv.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/csv.fan", parse = false)]
    pub struct Csv(Infallible);

    /// A visitor which collects the violations of the constraints in the CSV grammar.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitor<const CORRECT: bool> {
        path: VecDeque<usize>,
        checked: usize,
        violations: Vec<VecDeque<usize>>,
    }

    impl ConstraintVisitor<false> {
        /// Construct this visitor in the form that was originally evaluated in FANDANGO.
        #[deprecated(note = "The CSV grammar originally incorrectly counts the number of fields.")]
        pub fn evaluated() -> Self {
            Self::default()
        }
    }

    impl ConstraintVisitor<true> {
        /// Construct this visitor in the form that produces correctly formatted data.
        pub fn corrected() -> Self {
            Self::default()
        }
    }

    impl<const FIXED: bool> Checker for ConstraintVisitor<FIXED> {
        fn violations(self) -> Violations {
            Violations::new(
                if self.checked != 0 { Ratio::new(self.checked - self.violations.len(), self.checked) } else { Default::default() },
                self.violations,
            )
        }
    }

    pub(crate) fn count_fields(mut list: &nonterminal_csv_string_list) -> usize {
        let mut count = 0;
        while let Some(f) = list.nth::<0>().nth::<1>() {
            list = f.nth::<2>();
            count += 1;
        }
        count
    }

    impl<T> Visitor<T> for ConstraintVisitor<true>
    where
        T: VisitableChildren<T> + AsNodeRef<nonterminal_csv_records>,
    {
        type Continue = Self;
        type Break = Infallible;
        type Error = Infallible;

        fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
        where
            N: Node<Type<'program> = T>,
            T: From<&'program N> + AsNodeRef<N>,
        {
            self.path.push_back(idx);
            let visited = node.opaque();
            if let Some(tree) = visited.downcast::<nonterminal_csv_records>()
                && let Some(seq) = tree.nth::<0>().nth::<0>()
            {
                self.checked += 1;
                let base = count_fields(seq.nth::<0>().nth::<0>().nth::<0>());
                // because this is a universal equality, we can just check this pairwise
                if let Some(seq) = seq.nth::<1>().nth::<0>().nth::<0>() {
                    let cmp = count_fields(seq.nth::<0>().nth::<0>().nth::<0>());
                    if base != cmp {
                        let mut violation = self.path.clone();
                        violation.extend([0, 0, 1, 0, 0, 0, 0, 0]); // interior path to actual node
                        self.violations.push(violation)
                    }
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

        fn visit<'program, N>(self, _node: &'program N, _idx: usize) -> VisitResult<Self, T>
        where
            N: Node<Type<'program> = T>,
            T: From<&'program N> + AsNodeRef<N>,
        {
            Ok(ControlFlow::Continue(self)) // csv constraints are trivially true
        }
    }

    /// A visitor which applies fixes based on the constraints in the CSV grammar.
    #[derive(Debug)]
    pub struct ConstraintFixer<'a, S, G, const CORRECT: bool> {
        sampler: &'a mut S,
        generator: &'a mut G,
    }

    impl<'a, S, G> ConstraintFixer<'a, S, G, false> {
        /// Construct this fixer in the form that was originally evaluated in FANDANGO.
        #[deprecated(note = "The CSV grammar originally incorrectly counts the number of fields.")]
        pub fn evaluated(sampler: &'a mut S, generator: &'a mut G) -> Self {
            Self { sampler, generator }
        }
    }

    impl<'a, S, G> ConstraintFixer<'a, S, G, true> {
        /// Construct this fixer in the form that ensures the correctness of generated inputs.
        pub fn corrected(sampler: &'a mut S, generator: &'a mut G) -> Self {
            Self { sampler, generator }
        }
    }

    impl<'a, S, G, T> VisitorMut<T> for ConstraintFixer<'a, S, G, true>
    where
        nonterminal_raw_field: Generated<S, G>,
        T: VisitableChildrenMut<T> + AsNodeMut<nonterminal_csv_records>,
    {
        type Continue = Self;
        type Break = Infallible;
        type Error = Infallible;

        fn visit_mut<'program, N>(
            self,
            node: &'program mut N,
            _idx: usize,
        ) -> VisitMutResult<Self, T>
        where
            N: Node<TypeMut<'program> = T>,
            T: From<&'program mut N> + AsNodeMut<N>,
        {
            let mut visited = node.opaque_mut();
            if let Some(tree) = AsNodeMut::<nonterminal_csv_records>::as_node_mut(&mut visited) {
                if let Some(seq) = tree.child_mut().nth_mut::<0>() {
                    let (base, mut remaining) = seq.children_mut();
                    let base = count_fields(base.nth::<0>().nth::<0>());

                    // simply: "truncate or extend as needed"
                    while let Some(seq) = remaining.child_mut().nth_mut::<0>() {
                        let (cmp, remainder) = seq.children_mut();
                        remaining = remainder;

                        let mut curr = 0;
                        let mut tmp = cmp.child_mut().nth_mut::<0>();
                        while curr < base {
                            if let Some(inplace) = tmp.child_mut().nth_mut::<0>() {
                                let mut new = nonterminal_csv_string_list_0_1::default();
                                *new.nth_mut::<2>().nth_mut::<0>() =
                                    nonterminal_csv_string_list_0::from_0th(
                                        nonterminal_raw_field::generate(
                                            self.sampler,
                                            self.generator,
                                            0,
                                        ),
                                    );

                                mem::swap(inplace, new.nth_mut::<0>());
                                *tmp.nth_mut::<0>() =
                                    nonterminal_csv_string_list_0::variant_1(new.into());
                            }
                            tmp = tmp
                                .child_mut()
                                .nth_mut::<1>()
                                .expect("Must be present by construction")
                                .nth_mut::<2>();
                            curr += 1;
                        }

                        if let Some(seq) = tmp.child_mut().nth_mut::<1>() {
                            *tmp.child_mut() = nonterminal_csv_string_list_0::from_0th(mem::take(
                                seq.nth_mut::<0>(),
                            ));
                        }
                    }
                }
                return Ok(ControlFlow::Continue(self)); // terminate; we have completed the fix
            }
            visited.visit_each_mut(self)
        }
    }

    impl<'a, S, G, T> VisitorMut<T> for ConstraintFixer<'a, S, G, false> {
        type Continue = Self;
        type Break = Infallible;
        type Error = Infallible;

        fn visit_mut<'program, N>(
            self,
            _node: &'program mut N,
            _idx: usize,
        ) -> VisitMutResult<Self, T>
        where
            N: Node<TypeMut<'program> = T>,
            T: From<&'program mut N> + AsNodeMut<N>,
        {
            Ok(ControlFlow::Continue(self)) // csv constraints are trivially true
        }
    }

    #[cfg(test)]
    mod test {
        use crate::csv;
        use alloc::boxed::Box;
        use core::error::Error;
        use core::ops::ControlFlow;
        use fandango::generation::Generated;
        use fandango::tuple_list::tuple_list;
        use fandango::typing::{ChildAccessor, Nth, Structured};
        use fandango::visitor::navigation::GoTo;
        use fandango::visitor::{Visitor, VisitorMut};
        use fandango_runtime::operators::DepthLimiter;
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        #[test]
        fn check_constraint() -> Result<(), Box<dyn Error>> {
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(csv::nonterminal_start::ROOT.inner(), 50));
            let mut diff_count = 0;
            for _ in 0..100_000 {
                let mut tree = csv::nonterminal_start::generate(&mut rng, &mut generators, 0);
                let Ok(ControlFlow::Continue(csv::ConstraintVisitor { violations, .. })) =
                    csv::ConstraintVisitor::corrected().visit(&tree, 0);

                for mut violation in violations {
                    violation.pop_front();
                    assert!(matches!(
                        tree.go_to(0, violation.clone())?,
                        csv::Type::nonterminal_csv_string_list(_)
                    ));
                    let len = violation.len();

                    violation.truncate(len - 8);

                    let csv::Type::nonterminal_csv_records(records) = tree.go_to(0, violation)?
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

                    assert_ne!(csv::count_fields(base_list), csv::count_fields(cmp_list));

                    diff_count += 1;
                }

                let _ =
                    csv::ConstraintFixer::corrected(&mut rng, &mut ()).visit_mut(&mut tree, 0)?;
                let ControlFlow::Continue(csv::ConstraintVisitor { violations, .. }) =
                    csv::ConstraintVisitor::corrected().visit(&tree, 0)?;
                assert_eq!(0, violations.len());
            }
            assert_ne!(0, diff_count);
            Ok(())
        }
    }
}

pub use defs::*;
