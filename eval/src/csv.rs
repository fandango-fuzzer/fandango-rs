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

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::mem;
use core::ops::ControlFlow;
use fandango::generation::Generated;
use fandango::typing::{AsNodeMut, AsNodeRef, Node};
use fandango::visitor::{VisitResult, VisitableChildren, Visitor};
use fandango::Fandango;

/// Base for the CSV grammar stored in csv.fan.
#[derive(Fandango)]
#[fandango(grammar = "grammars/csv.fan", parse = false)]
pub struct Csv(Infallible);

/// A visitor which collects the violations of the constraints in the CSV grammar.
#[derive(Debug, Default)]
pub struct ConstraintVisitor<const CORRECT: bool> {
    path: VecDeque<usize>,
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

fn count_fields(mut list: &nonterminal_csv_string_list) -> usize {
    let mut count = 0;
    loop {
        list = match &list.child_0 {
            nonterminal_csv_string_list_0::variant_0(_) => break,
            nonterminal_csv_string_list_0::variant_1(f) => {
                count += 1;
                &f.child_2
            }
        }
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

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        self.path.push_back(idx);
        let visited = T::from(node);
        if let Some(tree) = visited.as_node() {
            if let nonterminal_csv_records_0::variant_0(seq) = &tree.child_0 {
                let (base, rest) = seq.children();
                let base = count_fields(&base.child_0.child_0);
                // because this is a universal equality, we can just check this pairwise
                if let nonterminal_csv_records_0::variant_0(seq) = &rest.child_0 {
                    let cmp = count_fields(&seq.child_0.child_0.child_0);
                    if base != cmp {
                        let mut violation = self.path.clone();
                        violation.extend([0, 0, 1, 0, 0, 0, 0, 0]); // interior path to actual node
                        self.violations.push(violation)
                    }
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

    fn visit<'program, N>(self, _node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
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

impl<'a, S, G, T> Visitor<T> for ConstraintFixer<'a, S, G, true>
where
    nonterminal_raw_field: Generated<S, G>,
    T: VisitableChildren<T> + AsNodeMut<nonterminal_csv_records>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(self, node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        let mut visited = T::from(node);
        if let Some(tree) = visited.as_node_mut() {
            if let nonterminal_csv_records_0::variant_0(seq) = &mut tree.child_0 {
                // this is horrible, but the compiler should be smart enough to see this
                // is not needed and directly replace... hopefully
                let mut exchange = nonterminal_raw_field {
                    child_0: nonterminal_raw_field_0::variant_0(nonterminal_simple_field {
                        child_0: Box::new(nonterminal_simple_field_0 {
                            child_0: nonterminal_spaces {
                                child_0: nonterminal_spaces_0::variant_0(nonterminal_spaces_0_0),
                            },
                            child_1: nonterminal_simple_characters {
                                child_0: nonterminal_simple_characters_0::variant_1(
                                    nonterminal_simple_character {
                                        child_0: nonterminal_simple_character_0::variant_0(
                                            nonterminal_simple_character_0_0,
                                        ),
                                    },
                                ),
                            },
                            child_2: nonterminal_spaces {
                                child_0: nonterminal_spaces_0::variant_0(nonterminal_spaces_0_0),
                            },
                        }),
                    }),
                };

                let (base, mut remaining) = seq.children_mut();
                let base = count_fields(&base.child_0.child_0);

                // simply: "truncate or extend as needed"
                while let nonterminal_csv_records_0::variant_0(seq) = &mut remaining.child_0 {
                    let (cmp, remainder) = seq.children_mut();
                    remaining = remainder;

                    let mut curr = 0;
                    let mut tmp = &mut cmp.child_0.child_0;
                    while curr < base {
                        if let nonterminal_csv_string_list_0::variant_0(inplace) = &mut tmp.child_0
                        {
                            mem::swap(inplace, &mut exchange);
                            exchange = match mem::replace(
                                &mut tmp.child_0,
                                nonterminal_csv_string_list_0::variant_1(Box::new(
                                    nonterminal_csv_string_list_0_1 {
                                        child_0: exchange,
                                        child_1: nonterminal_csv_string_list_0_1_1,
                                        child_2: nonterminal_csv_string_list {
                                            child_0: nonterminal_csv_string_list_0::variant_0(
                                                nonterminal_raw_field::generate(
                                                    self.sampler,
                                                    self.generator,
                                                    0,
                                                ),
                                            ),
                                        },
                                    },
                                )),
                            ) {
                                nonterminal_csv_string_list_0::variant_0(inner) => inner,
                                _ => unreachable!("Impossible case by construction."),
                            };
                        }
                        tmp = match &mut tmp.child_0 {
                            nonterminal_csv_string_list_0::variant_1(seq) => &mut seq.child_2,
                            _ => unreachable!("Impossible case by construction."),
                        };
                        curr += 1;
                    }
                    if let nonterminal_csv_string_list_0::variant_1(seq) = &mut tmp.child_0 {
                        mem::swap(&mut seq.child_0, &mut exchange);
                        exchange = match mem::replace(
                            &mut tmp.child_0,
                            nonterminal_csv_string_list_0::variant_0(exchange),
                        ) {
                            nonterminal_csv_string_list_0::variant_0(inner) => inner,
                            nonterminal_csv_string_list_0::variant_1(inner) => inner.child_0,
                        }
                    };
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
        T: From<&'program mut N>,
    {
        Ok(ControlFlow::Continue(self)) // csv constraints are trivially true
    }
}

#[cfg(test)]
mod test {
    use crate::csv;
    use crate::operators::DepthLimiter;
    use alloc::boxed::Box;
    use core::error::Error;
    use core::ops::ControlFlow;
    use fandango::generation::Generated;
    use fandango::tuple_list::tuple_list;
    use fandango::visitor::navigation::GoTo;
    use fandango::visitor::Visitor;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn check_constraint() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut generators = tuple_list!(DepthLimiter::new::<csv::Type<'static>>(50));
        let mut diff_count = 0;
        for _ in 0..100_000 {
            let mut tree = csv::nonterminal_start::generate(&mut rng, &mut generators, 0);
            let Ok(ControlFlow::Continue(csv::ConstraintVisitor { violations, .. })) =
                csv::ConstraintVisitor::corrected().visit(&mut tree, 0);

            for mut violation in violations {
                violation.pop_front();
                assert!(matches!(
                    tree.go_to(0, violation.clone())?,
                    csv::TypeMut::nonterminal_csv_string_list(_)
                ));
                let len = violation.len();

                violation.truncate(len - 8);

                let csv::TypeMut::nonterminal_csv_records(csv::nonterminal_csv_records {
                    child_0: csv::nonterminal_csv_records_0::variant_0(record),
                }) = tree.go_to(0, violation)?
                else {
                    unreachable!("We are inspecting the records directly.");
                };
                let csv::nonterminal_csv_records_0_0 {
                    child_0:
                        csv::nonterminal_csv_record {
                            child_0:
                                csv::nonterminal_csv_record_0 {
                                    child_0: base_list, ..
                                },
                        },
                    child_1:
                        csv::nonterminal_csv_records {
                            child_0: csv::nonterminal_csv_records_0::variant_0(remainder),
                        },
                } = &**record
                else {
                    unreachable!("We are inspecting the records directly.");
                };
                let csv::nonterminal_csv_records_0_0 {
                    child_0:
                        csv::nonterminal_csv_record {
                            child_0:
                                csv::nonterminal_csv_record_0 {
                                    child_0: cmp_list, ..
                                },
                        },
                    ..
                } = &**remainder;

                assert_ne!(csv::count_fields(base_list), csv::count_fields(cmp_list));

                diff_count += 1;
            }

            let _ = csv::ConstraintFixer::corrected(&mut rng, &mut ()).visit(&mut tree, 0)?;
            let ControlFlow::Continue(csv::ConstraintVisitor { violations, .. }) =
                csv::ConstraintVisitor::corrected().visit(&mut tree, 0)?;
            assert_eq!(0, violations.len());
        }
        assert_ne!(0, diff_count);
        Ok(())
    }
}
