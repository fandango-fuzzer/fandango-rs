use alloc::borrow::ToOwned;
use alloc::collections::VecDeque;
use alloc::vec;
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
pub struct XmlConstraintVisitor;

impl<T> Visitor<T> for XmlConstraintVisitor
where
    T: VisitableChildren<T>,
    T: AsNodeRef<nonterminal_xml_tree> + AsNodeRef<nonterminal_xml_attributes>,
{
    type Continue = Self;
    type Break = VecDeque<usize>;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        let mut visited = T::from(node);
        if let Some(tree) = AsNodeRef::<nonterminal_xml_tree>::as_node(&mut visited) {
            let (open, _, close) = tree.child_0.children();
            let id = match &open.child_0 {
                nonterminal_xml_open_tag_0::variant_0(n) => &n.child_1,
                nonterminal_xml_open_tag_0::variant_1(n) => &n.child_1,
            };
            if id != &close.child_0.child_1 {
                let mut path = VecDeque::new();
                path.push_front(idx);
                return Ok(ControlFlow::Break(path));
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
                        let mut path = VecDeque::new();
                        path.push_front(idx);
                        return Ok(ControlFlow::Break(path));
                    }
                    if let Some(actual) = maybe_rest {
                        rest = actual;
                    } else {
                        break;
                    }
                }
            }
        }
        match visited.visit_each(self) {
            Ok(ControlFlow::Break(mut path)) => {
                path.push_front(idx);
                Ok(ControlFlow::Break(path))
            }
            v => v,
        }
    }
}

#[derive(Debug)]
pub struct XmlConstraintFixer<'a, S, G> {
    sampler: &'a mut S,
    generator: &'a mut G,
}

impl<'a, S, G> XmlConstraintFixer<'a, S, G> {
    pub fn new(sampler: &'a mut S, generator: &'a mut G) -> Self {
        Self { sampler, generator }
    }
}

impl<S, G, T> Visitor<T> for XmlConstraintFixer<'_, S, G>
where
    nonterminal_id: Generated<S, G>,
    T: VisitableChildren<T>,
    T: AsNodeMut<nonterminal_xml_tree> + AsNodeMut<nonterminal_xml_attributes>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
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
                let mut ids = vec![&mut base.child_0.child_0];
                loop {
                    let (cmp, maybe_rest) = match &mut rest.child_0 {
                        nonterminal_xml_attributes_0::variant_0(cmp) => (cmp, None),
                        nonterminal_xml_attributes_0::variant_1(seq) => {
                            let (cmp, _, rest) = seq.children_mut();
                            (cmp, Some(rest))
                        }
                    };
                    ids.push(&mut cmp.child_0.child_0);
                    if let Some(actual) = maybe_rest {
                        rest = actual;
                    } else {
                        break;
                    }
                }
                let mut needs_mutation = true;
                while needs_mutation {
                    needs_mutation = false;
                    ids.sort_unstable();
                    for i in 0..(ids.len() - 1) {
                        if ids[i] == ids[i + 1] {
                            *ids[i] = nonterminal_id::generate(self.sampler, self.generator);
                            needs_mutation = true;
                        }
                    }
                }
            }
            return Ok(ControlFlow::Continue(self)); // attributes are already fixed, so no need
        }
        visited.visit_each(self)
    }
}

#[cfg(test)]
mod test {
    use crate::xml::{
        nonterminal_xml_attributes, nonterminal_xml_attributes_0, nonterminal_xml_open_tag_0,
        nonterminal_xml_tree, TypeMut, XmlConstraintFixer, XmlConstraintVisitor,
    };
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::error::Error;
    use core::ops::ControlFlow;
    use fandango::generation::Generated;
    use fandango::typing::Node;
    use fandango::visitor::navigation::GoTo;
    use fandango::visitor::write::WriteVisitor;
    use fandango::visitor::{VisitWith, Visitor};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn check_constraint() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut tag_diff_count = 0;
        let mut attr_diff_count = 0;
        for _ in 0..1000 {
            let mut tree = nonterminal_xml_tree::generate(&mut rng, &mut ());
            let invalid = match XmlConstraintVisitor::default().visit(&mut tree, 0)? {
                ControlFlow::Continue(_) => continue,
                ControlFlow::Break(mut path) => {
                    path.pop_front();
                    tree.go_to(0, path).unwrap()
                }
            };

            if let TypeMut::nonterminal_xml_tree(ref inner) = invalid {
                tag_diff_count += 1;
                let inner = inner.child_0.as_ref();
                let id = match &inner.child_0.child_0 {
                    nonterminal_xml_open_tag_0::variant_0(n) => &n.child_1,
                    nonterminal_xml_open_tag_0::variant_1(n) => &n.child_1,
                };
                assert_ne!(id, &inner.child_2.child_0.child_1);

                drop(invalid);

                let _ = XmlConstraintFixer::new(&mut rng, &mut ()).visit(&mut tree, 0)?;
                if let ControlFlow::Break(mut path) =
                    XmlConstraintVisitor::default().visit(&mut tree, 0)?
                {
                    path.pop_front();
                    let ser = String::from_utf8(
                        tree.go_to(0, path)
                            .unwrap()
                            .visit_with(WriteVisitor::new(Vec::new()), 0)?
                            .continue_value()
                            .unwrap()
                            .output(),
                    )?;
                    panic!("Did not successfully correct: {ser}");
                }
            }
        }
        for _ in 0..100000 {
            let mut tree = nonterminal_xml_attributes::generate(&mut rng, &mut ());
            let invalid = match XmlConstraintVisitor::default().visit(&mut tree, 0)? {
                ControlFlow::Continue(_) => continue,
                ControlFlow::Break(mut path) => {
                    path.pop_front();
                    tree.go_to(0, path).unwrap()
                }
            };
            attr_diff_count += 1;

            if let TypeMut::nonterminal_xml_attributes(ref attrs) = invalid {
                match &attrs.child_0 {
                    nonterminal_xml_attributes_0::variant_1(seq) => {
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
                    }
                    _ => unreachable!("This would need to be a sequence."),
                }
            }
            drop(invalid);

            let _ = XmlConstraintFixer::new(&mut rng, &mut ()).visit(&mut tree, 0)?;
            let ser = String::from_utf8(
                WriteVisitor::new(Vec::new())
                    .visit(&mut tree, 0)?
                    .continue_value()
                    .unwrap()
                    .output(),
            )?;
            if let ControlFlow::Break(mut path) =
                XmlConstraintVisitor::default().visit(&mut tree, 0)?
            {
                path.pop_front();
                let ser = String::from_utf8(
                    tree.go_to(0, path)
                        .unwrap()
                        .visit_with(WriteVisitor::new(Vec::new()), 0)?
                        .continue_value()
                        .unwrap()
                        .output(),
                )?;
                panic!("Did not successfully correct: {ser}");
            }
        }
        assert_ne!(0, tag_diff_count);
        assert_ne!(0, attr_diff_count);
        Ok(())
    }
}
