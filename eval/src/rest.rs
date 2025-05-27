//! Reimplements the constraints in the rest.fan grammar, namely:
//! ```text,ignore
//! forall <body> in <body_elements>:
//!     len(str(<body>.<body_element>.<section_title>.<title_text>)) <= len(str(<body>.<body_element>.<section_title>.<underline>))
//! ;
//!
//! forall <internal> in <paragraph_element>:
//!     exists <labeled_p> in <body_element>:
//!         str(<labeled_p>.<labeled_paragraph>.<label>.<id>) == str(<internal>.<internal_reference>.<id>)
//! ;
//!
//! forall <inter> in <internal_reference_nospace>:
//!     exists <labeled_p> in <body_element>:
//!         str(<labeled_p>.<labeled_paragraph>.<label>.<id>) == str(<inter>.<id>)
//! ;
//!
//! forall <l1> in <label>:
//!     exists <l2> in <label>:
//!         str(<l>.<id>) == str(<l2>.<id>) and <l> != <l2>
//! ;
//! ```
//!
//! Note that the last constraint is malformed and cannot be represented, so we ignore it.

use crate::Checker;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::ControlFlow;
use embedded_io::{ErrorType, Write};
use fandango::Fandango;
use fandango::typing::{AsNodeMut, AsNodeRef, Node};
use fandango::visitor::write::WriteVisitor;
use fandango::visitor::{VisitResult, VisitableChildren, Visitor};
use hashbrown::HashSet;

/// Base for the REST grammar stored in rest.fan.
#[derive(Fandango)]
#[fandango(grammar = "grammars/rest.fan", parse = false)]
pub struct Rest(Infallible);

/// A visitor which collects the violations of the constraints in the REST grammar.
#[derive(Debug, Default)]
pub struct ConstraintVisitor<const FIXED: bool> {
    path: VecDeque<usize>,
    violations: Vec<VecDeque<usize>>,
    labels: HashSet<nonterminal_id>,
}

impl ConstraintVisitor<false> {
    /// Construct this visitor in the form that was originally evaluated in FANDANGO.
    #[deprecated(note = "The REST grammar originally does not represent label deduplication.")]
    pub fn evaluated() -> Self {
        Self::default()
    }
}

impl<const FIXED: bool> Checker for ConstraintVisitor<FIXED> {
    fn violations(self) -> Vec<VecDeque<usize>> {
        self.violations
    }
}

#[derive(Debug, Default)]
struct RestConstraintContextVisitor<const FIXED: bool> {
    labels: HashSet<nonterminal_id>,
}

#[derive(Debug, Default)]
struct LengthCounter {
    count: usize,
}

impl ErrorType for LengthCounter {
    type Error = Infallible;
}

impl Write for LengthCounter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.count += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T> Visitor<T> for ConstraintVisitor<false>
where
    T: VisitableChildren<T>
        + AsNodeMut<nonterminal_body_elements>
        + AsNodeMut<nonterminal_section_title>
        + AsNodeRef<nonterminal_internal_reference>
        + AsNodeRef<nonterminal_internal_reference_nospace>,
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
        let mut visited = T::from(node);
        if self.labels.is_empty() {
            if let Some(elements) =
                AsNodeMut::<nonterminal_body_elements>::as_node_mut(&mut visited)
            {
                self.labels = RestConstraintContextVisitor::<false>::default()
                    .visit(elements, 0)?
                    .continue_value()
                    .unwrap()
                    .labels;
            }
        }

        if let Some(title) = AsNodeMut::<nonterminal_section_title>::as_node_mut(&mut visited) {
            if WriteVisitor::new(LengthCounter::default())
                .visit(&mut title.child_0.child_0, 0)?
                .continue_value()
                .unwrap()
                .output()
                .count
                > WriteVisitor::new(LengthCounter::default())
                    .visit(&mut title.child_0.child_2, 0)?
                    .continue_value()
                    .unwrap()
                    .output()
                    .count
            {
                let mut path = self.path.clone();
                path.extend([0, 2]);
                self.violations.push(path);
            }
        } else if let Some(internal) =
            AsNodeRef::<nonterminal_internal_reference>::as_node(&visited)
        {
            if !self.labels.contains(&internal.child_0.child_1) {
                let mut path = self.path.clone();
                path.extend([0, 1]);
                self.violations.push(path);
            }
        } else if let Some(internal) =
            AsNodeRef::<nonterminal_internal_reference_nospace>::as_node(&visited)
        {
            if !self.labels.contains(&internal.child_0.child_0) {
                let mut path = self.path.clone();
                path.extend([0, 0]);
                self.violations.push(path);
            }
        }
        let mut result = visited.visit_each(self);
        if let Ok(ControlFlow::Continue(visitor)) = &mut result {
            visitor.path.pop_back();
        }
        result
    }
}

impl<T> Visitor<T> for RestConstraintContextVisitor<false>
where
    T: VisitableChildren<T> + AsNodeRef<nonterminal_label>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        let visited = T::from(node);
        if let Some(label) = visited.as_node() {
            self.labels.insert(label.child_0.child_1.clone());
        }
        visited.visit_each(self)
    }
}

/// A visitor which applies fixes based on the constraints in the REST grammar.
#[derive(Debug, Default)]
pub struct RestConstraintFixer<const FIXED: bool>(());

impl<T> Visitor<T> for RestConstraintFixer<false> {
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(self, _node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N> + AsNodeMut<N>,
    {
        Ok(ControlFlow::Continue(self)) // no fixes available for original fandango
    }
}

#[cfg(test)]
mod test {
    use crate::operators::DepthLimiter;
    use crate::rest;
    use alloc::boxed::Box;
    use core::error::Error;
    use core::ops::ControlFlow;
    use fandango::generation::Generated;
    use fandango::tuple_list::tuple_list;
    use fandango::typing::Structured;
    use fandango::visitor::Visitor;
    use fandango::visitor::navigation::GoTo;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    #[allow(deprecated)]
    fn check_constraint() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut generators =
            tuple_list!(DepthLimiter::new(rest::nonterminal_start::ROOT.inner(), 50));
        let mut diff_count = 0;
        for _ in 0..100_000 {
            let mut tree = rest::nonterminal_start::generate(&mut rng, &mut generators, 0);
            let Ok(ControlFlow::Continue(rest::ConstraintVisitor { violations, .. })) =
                rest::ConstraintVisitor::evaluated().visit(&mut tree, 0);

            for mut violation in violations {
                let backup = violation.clone();
                violation.pop_front();
                let goto = tree.go_to(0, violation.clone())?;
                assert!(
                    matches!(
                        goto,
                        rest::TypeMut::nonterminal_id(_) | rest::TypeMut::nonterminal_underline(_)
                    ),
                    "at {backup:?}, found: {goto:?}"
                );
                diff_count += 1;
            }

            // cannot apply fixes; skip
        }
        assert_ne!(0, diff_count);
        Ok(())
    }
}
