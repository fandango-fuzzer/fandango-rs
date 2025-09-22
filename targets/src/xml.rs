//! Here, we define the constraints for the xml.fan grammar, namely:
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

#[cfg(not(feature = "static_defs"))]
mod defs {
    use core::convert::Infallible;
    use fandango::Fandango;

    /// Base for the XML grammar stored in xml.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/xml.fan", parse = false, dynamic = true)]
    pub struct Xml(Infallible);
}

#[cfg(feature = "static_defs")]
mod defs {
    use alloc::borrow::ToOwned;
    use alloc::collections::{BTreeSet, VecDeque};
    use alloc::vec::Vec;
    use anyhow::Error;
    use core::convert::Infallible;
    use core::ops::ControlFlow;
    use fandango::Fandango;
    use fandango::generation::Generated;
    use fandango::typing::{
        AsNodeMut, AsNodeRef, ChildAccessor, Downcast, DowncastMut, Node, Nth, Opaque, OpaqueMut,
    };
    use fandango::visitor::{
        VisitMutResult, VisitResult, VisitableChildren, VisitableChildrenMut, Visitor, VisitorMut,
    };
    use fandango_runtime::evolvers::basic::BasicHook;
    use fandango_runtime::measurement::Violations;
    use fandango_runtime::operators::Checker;
    use num_rational::Ratio;

    /// Base for the XML grammar stored in xml.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/xml.fan", parse = false)]
    pub struct Xml(Infallible);

    /// A visitor which collects the violations of the constraints in the XML grammar.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitor {
        path: VecDeque<usize>,
        checked: usize,
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
        fn violations(self) -> Violations {
            Violations::new(
                if self.checked != 0 { Ratio::new(self.checked - self.violations.len(), self.checked) } else { Default::default() },
                self.violations,
            )
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

        fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
        where
            N: Node<Type<'program> = T>,
            T: From<&'program N> + AsNodeRef<N>,
        {
            self.path.push_back(idx);
            let visited = node.opaque();
            if let Some(tree) = visited.downcast::<nonterminal_xml_tree>() {
                self.checked += 1;
                let (open, _, close) = tree.child().children();
                let id = match open.child() {
                    nonterminal_xml_open_tag_0::variant_0(n) => n.nth::<1>(),
                    nonterminal_xml_open_tag_0::variant_1(n) => n.nth::<1>(),
                };
                if id != close.child().nth::<1>() {
                    let mut violation = self.path.clone();
                    violation.extend([0, 2, 0, 1]); // interior path to actual node
                    self.violations.push(violation);
                }
            } else if let Some(tree) = visited.downcast::<nonterminal_xml_attributes>()
                && let Some(seq) = tree.child().nth::<1>()
            {
                let (base, _, mut rest) = seq.children();
                loop {
                    let (cmp, maybe_rest) = match rest.child() {
                        nonterminal_xml_attributes_0::variant_0(cmp) => (maybe_deref!(cmp), None),
                        nonterminal_xml_attributes_0::variant_1(seq) => {
                            let (cmp, _, rest) = seq.children();
                            (cmp, Some(rest))
                        }
                    };
                    self.checked += 1;
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

    impl<'a, S, G, const CORRECT: bool> ConstraintFixer<'a, S, G, CORRECT> {
        fn new(sampler: &'a mut S, generator: &'a mut G) -> Self {
            Self { sampler, generator }
        }
    }

    impl<'a, S, G> ConstraintFixer<'a, S, G, false> {
        /// Construct this fixer in the form that was originally evaluated in FANDANGO.
        #[deprecated(note = "The XML grammar fixer from FANDANGO is weaker than it could be.")]
        pub fn evaluated(sampler: &'a mut S, generator: &'a mut G) -> Self {
            Self::new(sampler, generator)
        }
    }

    impl<'a, S, G> ConstraintFixer<'a, S, G, true> {
        /// Construct this fixer in the form that ensures the correctness of generated inputs.
        pub fn corrected(sampler: &'a mut S, generator: &'a mut G) -> Self {
            Self::new(sampler, generator)
        }
    }

    impl<S, G, T> VisitorMut<T> for ConstraintFixer<'_, S, G, true>
    where
        nonterminal_id: Generated<S, G>,
        T: VisitableChildrenMut<T>
            + AsNodeMut<nonterminal_xml_tree>
            + AsNodeMut<nonterminal_xml_attributes>,
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
            if let Some(tree) = visited.downcast_mut::<nonterminal_xml_tree>() {
                let (open, _, close) = tree.child_mut().children_mut();
                let id = match open.child() {
                    nonterminal_xml_open_tag_0::variant_0(n) => n.nth::<1>(),
                    nonterminal_xml_open_tag_0::variant_1(n) => n.nth::<1>(),
                };
                id.clone_into(close.child_mut().nth_mut::<1>());
            } else if let Some(tree) = visited.downcast_mut::<nonterminal_xml_attributes>() {
                if let Some(seq) = tree.child_mut().nth_mut::<1>() {
                    let (base, _, mut rest) = seq.children_mut();
                    let mut ids = BTreeSet::new();
                    ids.insert(base.child_mut().nth_mut::<0>());
                    loop {
                        let (cmp, maybe_rest) = match rest.child_mut() {
                            nonterminal_xml_attributes_0::variant_0(cmp) => {
                                (maybe_deref_mut!(cmp), None)
                            }
                            nonterminal_xml_attributes_0::variant_1(seq) => {
                                let (cmp, _, rest) = seq.children_mut();
                                (cmp, Some(rest))
                            }
                        };

                        let cmp = cmp.child_mut().nth_mut::<0>();
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
            visited.visit_each_mut(self)
        }
    }

    impl<S, G, T> VisitorMut<T> for ConstraintFixer<'_, S, G, false>
    where
        nonterminal_id: Generated<S, G>,
        T: VisitableChildrenMut<T>
            + AsNodeMut<nonterminal_xml_tree>
            + AsNodeMut<nonterminal_xml_attributes>,
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
            if let Some(tree) = AsNodeMut::<nonterminal_xml_tree>::as_node_mut(&mut visited) {
                let (open, _, close) = tree.child_mut().children_mut();
                let id = match open.child() {
                    nonterminal_xml_open_tag_0::variant_0(n) => n.nth::<1>(),
                    nonterminal_xml_open_tag_0::variant_1(n) => n.nth::<1>(),
                };
                id.clone_into(close.child_mut().nth_mut::<1>());
            }
            visited.visit_each_mut(self)
        }
    }

    /// A [`BasicHook`] which performs automatic fixes over the [`Xml`] grammar
    pub struct XmlFixHook<const FIXED: bool>;

    impl XmlFixHook<false> {
        /// The fix hook with the fixes available from the original FANDANGO
        #[deprecated(note = "The XML grammar fixer from FANDANGO is weaker than it could be.")]
        pub fn evaluated() -> Self {
            Self
        }
    }

    impl XmlFixHook<true> {
        /// The fix hook with maximum possible fixes
        pub fn corrected() -> Self {
            Self
        }
    }

    impl<N, G, S, const FIXED: bool> BasicHook<N, G, S> for XmlFixHook<FIXED>
    where
        N: Node,
        for<'a, 'b> ConstraintFixer<'a, S, G, FIXED>: VisitorMut<N::TypeMut<'b>>,
    {
        fn individual_created(
            &mut self,
            node: &mut N,
            generators: &mut G,
            sampler: &mut S,
        ) -> Result<(), Error> {
            let fixer = ConstraintFixer::<_, _, FIXED>::new(sampler, generators);
            let _res = fixer.visit_mut(node, 0);
            Ok(())
        }
    }
}

pub use defs::*;
