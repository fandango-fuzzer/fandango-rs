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

use alloc::borrow::ToOwned;
use alloc::collections::{BTreeSet, VecDeque};
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::ControlFlow;
use fandango::generation::Generated;
use fandango::typing::{AsNodeMut, AsNodeRef, Node};
use fandango::visitor::{VisitResult, VisitableChildren, Visitor};
use fandango::Fandango;

#[derive(Fandango)]
#[fandango(grammar = "grammars/xml.fan", parse = false)]
pub struct Xml;

#[derive(Debug, Default)]
pub struct XmlConstraintVisitor {
    path: VecDeque<usize>,
    violations: Vec<VecDeque<usize>>,
}

impl<T> Visitor<T> for XmlConstraintVisitor
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
        T: From<&'program mut N>,
    {
        self.path.push_back(idx);
        let mut visited = T::from(node);
        if let Some(tree) = AsNodeRef::<nonterminal_xml_tree>::as_node(&mut visited) {
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
        } else if let Some(tree) = AsNodeRef::<nonterminal_xml_attributes>::as_node(&mut visited) {
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

#[derive(Debug)]
pub struct XmlConstraintFixer<'a, S, G, const CORRECT: bool> {
    sampler: &'a mut S,
    generator: &'a mut G,
}

impl<'a, S, G> XmlConstraintFixer<'a, S, G, true> {
    pub fn corrected(sampler: &'a mut S, generator: &'a mut G) -> Self {
        Self { sampler, generator }
    }
}

impl<S, G, T> Visitor<T> for XmlConstraintFixer<'_, S, G, true>
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
        T: From<&'program mut N>,
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

impl<'a, S, G> XmlConstraintFixer<'a, S, G, false> {
    #[deprecated(note = "This is an incomplete fixer, used for evaluation purposes.")]
    pub fn evaluated(sampler: &'a mut S, generator: &'a mut G) -> Self {
        Self { sampler, generator }
    }
}

impl<S, G, T> Visitor<T> for XmlConstraintFixer<'_, S, G, false>
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
        T: From<&'program mut N>,
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
    use crate::xml;
    use crate::xml::{
        nonterminal_xml_attributes_0, nonterminal_xml_open_tag_0, nonterminal_xml_tree,
        XmlConstraintFixer, XmlConstraintVisitor,
    };
    use alloc::boxed::Box;
    use core::error::Error;
    use core::ops::ControlFlow;
    use fandango::generation::Generated;
    use fandango::tuple_list::tuple_list;
    use fandango::typing::Node;
    use fandango::visitor::navigation::GoTo;
    use fandango::visitor::Visitor;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn check_constraint() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut generators = tuple_list!(DepthLimiter::new::<xml::Type<'static>>(50));
        let mut tag_diff_count = 0;
        let mut attr_diff_count = 0;
        for _ in 0..100_000 {
            let mut tree = nonterminal_xml_tree::generate(&mut rng, &mut generators, 0);
            let Ok(ControlFlow::Continue(XmlConstraintVisitor { violations, .. })) =
                XmlConstraintVisitor::default().visit(&mut tree, 0);

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
                        nonterminal_xml_open_tag_0::variant_0(n) => &n.child_1,
                        nonterminal_xml_open_tag_0::variant_1(n) => &n.child_1,
                    };
                    assert_ne!(id, &inner.child_2.child_0.child_1);
                } else if let xml::TypeMut::nonterminal_xml_attributes(ref attrs) =
                    tree.go_to(0, violation.into_iter().take(len - 5).collect())?
                {
                    attr_diff_count += 1;
                    if let nonterminal_xml_attributes_0::variant_1(seq) = &attrs.child_0 {
                        let (base, _, mut rest) = seq.children();
                        let diff_found = loop {
                            rest = match &rest.child_0 {
                                nonterminal_xml_attributes_0::variant_0(cmp) => {
                                    if cmp == base {
                                        break true;
                                    }
                                    break false;
                                }
                                nonterminal_xml_attributes_0::variant_1(seq) => {
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

            let _ = XmlConstraintFixer::corrected(&mut rng, &mut ()).visit(&mut tree, 0)?;
            let ControlFlow::Continue(XmlConstraintVisitor { violations, .. }) =
                XmlConstraintVisitor::default().visit(&mut tree, 0)?;
            assert_eq!(0, violations.len());
        }
        assert_ne!(0, tag_diff_count);
        assert_ne!(0, attr_diff_count);
        Ok(())
    }
}
