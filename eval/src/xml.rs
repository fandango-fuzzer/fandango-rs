//! Here, we define the constraints for the scriptsizec.fan grammar, namely:
//! ```text,ignore
//! forall <tree> in <xml_tree>:
//!     <tree>.<xml_open_tag>.<id> == <tree>.<xml_close_tag>.<id>
//! ;
//!
//! forall <open_tag> in <xml_tree>.<xml_open_tag>:
//!     forall <xml_attribute_1> in <open_tag>..<xml_attribute>:
//!         forall <xml_attribute_2> in <open_tag>..<xml_attribute>:
//!             (<xml_attribute_1> != <xml_attribute_2> -> str(<xml_attribute_1>.<id>) != str(<xml_attribute_2>.<id>))
//! ;
//! ```

use crate::Checker;
use alloc::borrow::ToOwned;
use alloc::collections::{BTreeSet, VecDeque};
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::ControlFlow;
use fandango::Fandango;
use fandango::generation::Generated;
use fandango::typing::{AsNodeMut, AsNodeRef, Node};
use fandango::visitor::{VisitResult, VisitableChildren, Visitor};

/// Base for the XML grammar stored in xml.fan.
#[derive(Fandango)]
#[fandango(grammar = "grammars/xml.fan", parse = false)]
pub struct Xml(Infallible);

/// A visitor which collects the violations of the constraints in the XML grammar.
#[derive(Debug, Default)]
pub struct ConstraintVisitor {
    path: VecDeque<usize>,
    violations: Vec<VecDeque<usize>>,
}

impl ConstraintVisitor {
    /// Construct this visitor in the form that was originally evaluated in FANDANGO.
    pub fn evaluated() -> Self {
        ConstraintVisitor::default()
    }

    /// Construct this visitor in the form that produces correctly formatted data.
    pub fn corrected() -> Self {
        ConstraintVisitor::default()
    }
}

impl Checker for ConstraintVisitor {
    fn violations(self) -> Vec<VecDeque<usize>> {
        self.violations
    }
}

impl<T> Visitor<T> for ConstraintVisitor
where
    T: VisitableChildren<T>
        + AsNodeRef<nonterminal_xml_tree>
        + AsNodeRef<nonterminal_xml_attributes>,
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
        if let Some(tree) = AsNodeRef::<nonterminal_xml_tree>::as_node(&visited) {
            let (open, _, close) = tree.child_0.children();
            let id = match &open.child_0 {
                nonterminal_xml_open_tag_0::variant_0(n) => &n.child_1,
                nonterminal_xml_open_tag_0::variant_1(n) => &n.child_1,
            };
            if id != &close.child_0.child_1 {
                let mut violation = self.path.clone();
                violation.extend([0, 2, 0, 1]); // interior path to actual node
                self.violations.push(violation);
            }
        } else if let Some(tree) = AsNodeRef::<nonterminal_xml_attributes>::as_node(&visited) {
            if let nonterminal_xml_attributes_0::variant_1(seq) = &tree.child_0 {
                let (base, _, mut rest) = seq.children();
                loop {
                    let (cmp, maybe_rest) = match &rest.child_0 {
                        nonterminal_xml_attributes_0::variant_0(cmp) => (cmp, None),
                        nonterminal_xml_attributes_0::variant_1(seq) => {
                            let (cmp, _, rest) = seq.children();
                            (cmp, Some(rest))
                        }
                    };
                    if base == cmp {
                        let mut violation = self.path.clone();
                        violation.extend([0, 1, 0, 0, 0]); // interior path to actual node
                        self.violations.push(violation);
                    }
                    if let Some(actual) = maybe_rest {
                        rest = actual;
                    } else {
                        break;
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

/// A visitor which applies fixes based on the constraints in the XML grammar.
#[derive(Debug)]
pub struct ConstraintFixer<'a, S, G, const CORRECT: bool> {
    sampler: &'a mut S,
    generator: &'a mut G,
}

impl<'a, S, G> ConstraintFixer<'a, S, G, false> {
    /// Construct this fixer in the form that was originally evaluated in FANDANGO.
    #[deprecated(note = "This is an incomplete fixer, used for evaluation purposes.")]
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

impl<S, G, T> Visitor<T> for ConstraintFixer<'_, S, G, true>
where
    nonterminal_id: Generated<S, G>,
    T: VisitableChildren<T>
        + AsNodeMut<nonterminal_xml_tree>
        + AsNodeMut<nonterminal_xml_attributes>,
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
        if let Some(tree) = AsNodeMut::<nonterminal_xml_tree>::as_node_mut(&mut visited) {
            let (open, _, close) = tree.child_0.children_mut();
            let id = match &open.child_0 {
                nonterminal_xml_open_tag_0::variant_0(n) => &n.child_1,
                nonterminal_xml_open_tag_0::variant_1(n) => &n.child_1,
            };
            id.clone_into(&mut close.child_0.child_1);
        } else if let Some(tree) =
            AsNodeMut::<nonterminal_xml_attributes>::as_node_mut(&mut visited)
        {
            if let nonterminal_xml_attributes_0::variant_1(seq) = &mut tree.child_0 {
                let (base, _, mut rest) = seq.children_mut();
                let mut ids = BTreeSet::new();
                ids.insert(&mut base.child_0.child_0);
                loop {
                    let (cmp, maybe_rest) = match &mut rest.child_0 {
                        nonterminal_xml_attributes_0::variant_0(cmp) => (cmp, None),
                        nonterminal_xml_attributes_0::variant_1(seq) => {
                            let (cmp, _, rest) = seq.children_mut();
                            (cmp, Some(rest))
                        }
                    };

                    let cmp = &mut cmp.child_0.child_0;
                    while ids.contains(cmp) {
                        *cmp = nonterminal_id::generate(self.sampler, self.generator, 0);
                    }
                    ids.insert(cmp);

                    if let Some(actual) = maybe_rest {
                        rest = actual;
                    } else {
                        break;
                    }
                }
            }
            return Ok(ControlFlow::Continue(self)); // attributes are already fixed, so no need
        }
        visited.visit_each(self)
    }
}

impl<S, G, T> Visitor<T> for ConstraintFixer<'_, S, G, false>
where
    nonterminal_id: Generated<S, G>,
    T: VisitableChildren<T>
        + AsNodeMut<nonterminal_xml_tree>
        + AsNodeMut<nonterminal_xml_attributes>,
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
        if let Some(tree) = AsNodeMut::<nonterminal_xml_tree>::as_node_mut(&mut visited) {
            let (open, _, close) = tree.child_0.children_mut();
            let id = match &open.child_0 {
                nonterminal_xml_open_tag_0::variant_0(n) => &n.child_1,
                nonterminal_xml_open_tag_0::variant_1(n) => &n.child_1,
            };
            id.clone_into(&mut close.child_0.child_1);
        }
        visited.visit_each(self)
    }
}

#[cfg(test)]
mod test {
    use crate::operators::DepthLimiter;
    use crate::{crossover, xml};
    use alloc::boxed::Box;
    use alloc::collections::VecDeque;
    use alloc::vec;
    use core::error::Error;
    use core::ops::ControlFlow;
    use fandango::generation::Generated;
    use fandango::tuple_list::tuple_list;
    use fandango::typing::{Node, Structured};
    use fandango::visitor::Visitor;
    use fandango::visitor::navigation::GoTo;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn check_constraint() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut generators =
            tuple_list!(DepthLimiter::new(xml::nonterminal_start::ROOT.inner(), 50));
        let mut tag_diff_count = 0;
        let mut attr_diff_count = 0;
        for _ in 0..100_000 {
            let mut tree = xml::nonterminal_start::generate(&mut rng, &mut generators, 0);
            let Ok(ControlFlow::Continue(xml::ConstraintVisitor { violations, .. })) =
                xml::ConstraintVisitor::corrected().visit(&mut tree, 0);

            for mut violation in violations {
                violation.pop_front();
                assert!(matches!(
                    tree.go_to(0, violation.clone())?,
                    xml::TypeMut::nonterminal_id(_)
                ));
                let len = violation.len();

                if let xml::TypeMut::nonterminal_xml_tree(ref inner) =
                    tree.go_to(0, violation.iter().take(len - 4).copied().collect())?
                {
                    tag_diff_count += 1;
                    let inner = inner.child_0.as_ref();
                    let id = match &inner.child_0.child_0 {
                        xml::nonterminal_xml_open_tag_0::variant_0(n) => &n.child_1,
                        xml::nonterminal_xml_open_tag_0::variant_1(n) => &n.child_1,
                    };
                    assert_ne!(id, &inner.child_2.child_0.child_1);
                } else if let xml::TypeMut::nonterminal_xml_attributes(ref attrs) =
                    tree.go_to(0, violation.into_iter().take(len - 5).collect())?
                {
                    attr_diff_count += 1;
                    if let xml::nonterminal_xml_attributes_0::variant_1(seq) = &attrs.child_0 {
                        let (base, _, mut rest) = seq.children();
                        let diff_found = loop {
                            rest = match &rest.child_0 {
                                xml::nonterminal_xml_attributes_0::variant_0(cmp) => {
                                    if cmp == base {
                                        break true;
                                    }
                                    break false;
                                }
                                xml::nonterminal_xml_attributes_0::variant_1(seq) => {
                                    let (cmp, _, rest) = seq.children();
                                    if cmp == base {
                                        break true;
                                    }
                                    rest
                                }
                            };
                        };
                        assert!(diff_found);
                    } else {
                        unreachable!("This would need to be a sequence.")
                    }
                }
            }

            let _ = xml::ConstraintFixer::corrected(&mut rng, &mut ()).visit(&mut tree, 0)?;
            let ControlFlow::Continue(xml::ConstraintVisitor { violations, .. }) =
                xml::ConstraintVisitor::default().visit(&mut tree, 0)?;
            assert_eq!(0, violations.len());
        }
        assert_ne!(0, tag_diff_count);
        assert_ne!(0, attr_diff_count);
        Ok(())
    }

    #[test]
    fn mutate() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);

        let mut first = xml::nonterminal_start::generate(&mut rng, &mut (), 0);
        let second = first.clone();
        assert_eq!(first, second);

        let mut choices = vec![VecDeque::from([0])];
        let mutated =
            crate::operators::mutate(&mut first, &mut choices, &mut rng, &mut ())?.unwrap();
        assert!(matches!(mutated, xml::TypeMut::nonterminal_start(_)));
        assert!(choices.is_empty());
        assert_ne!(first, second);

        Ok(())
    }

    #[test]
    fn crossover() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);

        let mut first = xml::nonterminal_start::generate(&mut rng, &mut (), 0);
        let mut second = xml::nonterminal_start::generate(&mut rng, &mut (), 0);
        assert_ne!(first, second);

        let mut choices = vec![VecDeque::from([0])];
        let crossed = crossover!(
            xml::nonterminal_start,
            &mut first,
            &mut second,
            choices,
            &mut rng
        )?;
        assert!(crossed);
        assert!(choices.is_empty());
        assert_eq!(first, second);

        Ok(())
    }
}
