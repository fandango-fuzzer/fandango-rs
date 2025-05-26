//! Here, we define the constraints for the scriptsizec.fan grammar, namely:
//! ```text,ignore
//! forall <use_id> in <statement>..<expr>..<id>:
//!     exists <dec> in <declaration>:
//!         str(<dec>.<id>) == str(<id>) and is_before(<start>, <dec>, <use_id>)
//! ;
//!
//! forall <decl1> in <declaration>:
//!     forall <decl2> in <declaration>:
//!         not(str(<decl1>.<id>)==str(<decl2>.<id>)) or <decl1>==<decl2>
//! ;
//! ```
//!
//! Note that this constraint set is erroneous; scope tracking is not correctly implemented.

use alloc::collections::{BTreeSet, VecDeque};
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::ControlFlow;
use fandango::typing::{AsNodeRef, Node};
use fandango::visitor::{VisitResult, VisitableChildren, Visitor};
use fandango::Fandango;

#[derive(Fandango)]
#[fandango(grammar = "grammars/scriptsizec.fan", parse = false)]
pub struct ScriptSizeC(());

#[derive(Debug, Default)]
pub struct SSCConstraintVisitor<S> {
    scope: S,
    path: VecDeque<usize>,
    violations: Vec<VecDeque<usize>>,
}

impl SSCConstraintVisitor<BTreeSet<nonterminal_id>> {
    #[deprecated(note = "This implements the incorrect version of scriptsizec.fan constraints.")]
    pub fn evaluated() -> Self {
        Self::default()
    }
}

impl<T> Visitor<T> for SSCConstraintVisitor<BTreeSet<nonterminal_id>>
where
    T: VisitableChildren<T> + AsNodeRef<nonterminal_declaration> + AsNodeRef<nonterminal_id>,
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
        if let Some(decl) = AsNodeRef::<nonterminal_declaration>::as_node(&visited) {
            let (id, path) = match &decl.child_0 {
                nonterminal_declaration_0::variant_0(nonterminal_declaration_0_0 {
                    child_1: id,
                    ..
                }) => (id, [0, 0, 1]),
                nonterminal_declaration_0::variant_1(nonterminal_declaration_0_1 {
                    child_1: id,
                    ..
                }) => (id, [0, 1, 1]),
            };
            if self.scope.contains(id) {
                let mut violation = self.path.clone();
                violation.extend(path);
                self.violations.push(violation);
            } else {
                self.scope.insert(id.clone());
            }
        } else if let Some(id) = AsNodeRef::<nonterminal_id>::as_node(&mut visited) {
            if !self.scope.contains(id) {
                self.violations.push(self.path.clone());
            }
        }
        let result = visited.visit_each(self);
        let Ok(ControlFlow::Continue(mut visitor)) = result;
        visitor.path.pop_back();
        Ok(ControlFlow::Continue(visitor))
    }
}

pub struct SSCConstraintFixer<S> {
    scope: S,
}

impl<T> Visitor<T> for SSCConstraintFixer<BTreeSet<nonterminal_id>> {
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(self, _node: &'program mut N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        Ok(ControlFlow::Continue(self)) // no fixes available for original fandango
    }
}

#[cfg(test)]
mod test {
    use crate::operators::DepthLimiter;
    use crate::scriptsizec;
    use crate::scriptsizec::{nonterminal_statement, SSCConstraintVisitor};
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
    #[allow(deprecated)]
    fn check_constraint() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut generators = tuple_list!(DepthLimiter::new::<scriptsizec::Type<'static>>(50));
        let mut diff_count = 0;
        for _ in 0..100_000 {
            let mut tree = nonterminal_statement::generate(&mut rng, &mut generators, 0);
            let Ok(ControlFlow::Continue(SSCConstraintVisitor { violations, .. })) =
                SSCConstraintVisitor::evaluated().visit(&mut tree, 0);

            for mut violation in violations {
                violation.pop_front();
                assert!(matches!(
                    tree.go_to(0, violation.clone())?,
                    scriptsizec::TypeMut::nonterminal_id(_)
                ));
                diff_count += 1;
            }

            // cannot apply fixes; skip
        }
        assert_ne!(0, diff_count);
        Ok(())
    }
}
