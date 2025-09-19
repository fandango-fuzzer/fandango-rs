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
    use fandango_runtime::measurement::Violations;
    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use alloc::string::String;
    use core::convert::Infallible;
    use core::ops::ControlFlow;
    use fandango::Fandango;
    use fandango::generation::Generated;
    use fandango::typing::{AsNodeMut, AsNodeRef, Node, Nth, Downcast, Opaque, DowncastMut};
    use fandango::visitor::{VisitMutResult, VisitResult, VisitableChildren, VisitableChildrenMut, Visitor, VisitorMut};
    use fandango::visitor::write::WriteVisitor;
    use fandango_runtime::operators::Checker;
    use rand::Rng;
    
    /// Base for the lang grammar stored in lang.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/lang.fan", parse = false)]
    pub struct Lang(Infallible);
    
    // ================= Combined def-use and at least one var access.
    // Constraint visitor.
    // Mainly for illustrative purposes; in the future, we will devise a strategy to 
    // have multiple independent constraints running at once.
    //
    /// Basic combined constraint visitor.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorAtLeastOneVarAlsoDefUse {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The count of variable accesses found so far.
        pub var_access_count: usize,
        /// The set of currently defined variables, mapping names to nodes.
        pub defined_vars: alloc::collections::BTreeSet<String>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
    }

    impl Checker for ConstraintVisitorAtLeastOneVarAlsoDefUse {
        fn violations(self) -> Violations {
            // If there are no variable accesses, or if there are def-before-use violations, we have violations.
            if self.var_access_count == 0 {
                let mut violations = self.violations;
                violations.push([0].into());
                Violations::new(violations.len(), violations)
            } else {
                Violations::new(self.violations.len(), self.violations)
            }
        }
    }

    impl<T> Visitor<T> for ConstraintVisitorAtLeastOneVarAlsoDefUse
    where 
        T: VisitableChildren<T> + 
        AsNodeRef<nonterminal_var_access> +
        AsNodeRef<nonterminal_assignment> +
        AsNodeRef<nonterminal_decl>
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
            let visited = T::from(node);
            if let Some(_tree) = visited.downcast::<nonterminal_var_access>() {
                self.var_access_count += 1;
                let var_name_str = String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(_tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output(),
                ).unwrap();
                if !self.defined_vars.contains(&var_name_str) {
                    self.violations.push(self.path.clone());
                }
            } else if let Some(decl_tree) = visited.downcast::<nonterminal_decl>() {
                let var_decl_name = String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(decl_tree.nth::<0>().nth::<2>(), 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output(),
                ).unwrap();
                self.defined_vars.insert(var_decl_name);
            }
            let mut result = visited.visit_each(self);
            if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                visitor.path.pop_back();
            }
            result
        }
    }

    // ================= Ensure sufficiently many variable accesses.
    // Constraint visitor.
    // Mostly to ensure that there are variable accesses in generated programs.
    // Note: No scoping, not perfect.
    //
    /// Basic counter visitor.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorVarAccess {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The count of variable accesses found so far.
        pub var_access_count: usize,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
    }

    impl Checker for ConstraintVisitorVarAccess {
        fn violations(self) -> Violations {
            // We consider it a violation if there are no variable accesses.
            let mut violations = self.violations;
            if self.var_access_count == 0 {
                violations.push(self.path);
            }
            Violations::new(violations.len(), violations)
        }
    }

    impl<T> Visitor<T> for ConstraintVisitorVarAccess
    where 
        T: VisitableChildren<T> + 
        AsNodeRef<nonterminal_var_access>
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
            let visited = T::from(node);
            if let Some(_tree) = visited.downcast::<nonterminal_var_access>() {
                self.var_access_count += 1;
            }
            let mut result = visited.visit_each(self);
            if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                visitor.path.pop_back();
            }
            result
        }
    }
    // ================= end of Ensure sufficiently many variable accesses.

    // ================= Def before use.
    // Constraint visitor.
    // Note: Not yet perfect, no scoping, also declarations like let a = a are allowed. 
    //
    /// Basic def-before-use constraint visitor.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorDefUse {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The set of currently defined variables, mapping names to nodes.
        pub defined_vars: alloc::collections::BTreeSet<String>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
    }
    
    impl Checker for ConstraintVisitorDefUse {
        fn violations(self) -> Violations {
            Violations::new(self.violations.len(), self.violations)
        }
    }

    impl<T> Visitor<T> for ConstraintVisitorDefUse
    where 
        T: VisitableChildren<T> +
        AsNodeRef<nonterminal_var_access> +
        AsNodeRef<nonterminal_assignment> +
        AsNodeRef<nonterminal_decl>
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
            let visited = T::from(node);
            
            // TODO Move decl thingy here.
            if let Some(tree) = visited.downcast::<nonterminal_var_access>() {
                let var_name_str = String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output(),
                ).unwrap();
                if !self.defined_vars.contains(&var_name_str) {
                    self.violations.push(self.path.clone());
                }
            } else if let Some(decl_tree) = visited.downcast::<nonterminal_decl>() {
                let var_decl_name = String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(decl_tree.nth::<0>().nth::<2>(), 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output(),
                ).unwrap();
                self.defined_vars.insert(var_decl_name);
            }
            let mut result = visited.visit_each(self);
            if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                visitor.path.pop_back();
            }
            result
        }
    }

    // ================= end of Def before use.

    // ================= Def before use fixer.
    // This fixer will attempt to fix def-before-use violations by replacing
    // variable accesses with defined variables, or with a random new name if none exist.
    //
    /// A fixer which applies fixes based on the def-before-use constraints in the lang grammar.
    #[derive(Debug)]
    pub struct ConstraintFixerDefUse<'a, S, G> {
        /// Sampler is an external random number generator.
        pub sampler: &'a mut S,
        /// Generator is the tuple-list based generator.
        pub generator: &'a mut G,
        /// The set of currently defined variables, mapping names to nodes.
        pub defined_vars: &'a mut alloc::collections::BTreeMap<String, nonterminal_var_name>,
    }

    // TODO: Test this and see if it helps.
    impl<'a, S, G, T> VisitorMut<T> for ConstraintFixerDefUse<'a, S, G>
    where
        nonterminal_var_access: Generated<S, G>,
        T: VisitableChildrenMut<T> + 
            AsNodeMut<nonterminal_var_access> + 
            AsNodeMut<nonterminal_decl> +
            AsNodeMut<nonterminal_var_name>,
    {
        type Continue = Self;
        type Break = Infallible;
        type Error = Infallible;

        fn visit_mut<'program, N>(self, node: &'program mut N, _idx: usize) -> VisitMutResult<Self, T>
        where
            N: Node<TypeMut<'program> = T>,
            T: From<&'program mut N> + AsNodeMut<N>,
        {
            let mut visited = T::from(node);
            if let Some(tree) = visited.downcast_mut::<nonterminal_var_access>() {
                let var_name_str = String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output(),
                ).unwrap();
                if !self.defined_vars.contains_key(&var_name_str) {
                    // We have a violation, we need to fix it.
                    if self.defined_vars.is_empty() {
                        // No defined variables, generate a new var access and hope that Fandango handles it.
                        let new_var: nonterminal_var_access = nonterminal_var_access::generate(self.sampler, self.generator, 0);
                        *tree = new_var;
                    } else {
                        // Pick a random defined variable to use.
                        let var_names: Vec<&String> = self.defined_vars.keys().collect();
                        let mut rng = rand::rng();
                        let choice_idx = rng.random_range(0..var_names.len());
                        let chosen_name = var_names[choice_idx];
                        let chosen_node = self.defined_vars.get(chosen_name).unwrap().clone();
                        *tree.nth_mut::<0>() = chosen_node;
                    }
                }
                // else it is defined, do nothing.
            } else if let Some(decl_tree) = visited.downcast_mut::<nonterminal_decl>() {
                let var_decl_name = String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(decl_tree.nth::<0>().nth::<2>(), 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output(),
                ).unwrap();
                let var_decl_name_node_clone = decl_tree.nth_mut::<0>().nth_mut::<2>().clone();
                self.defined_vars.insert( var_decl_name, var_decl_name_node_clone);
            }
            visited.visit_each_mut(self)
        }
    }
    // ================= end of Def before use fixer.

    // ================= Returns only inside functions.
    // Constraint visitor.
    /// Basic return-in-function-only constraint visitor.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorReturnInFunc {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The current function depth, to track if we are inside a function or not.
        pub func_depth: usize,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
    }

    // Visitor that checks for violations.
    impl<T> Visitor<T> for ConstraintVisitorReturnInFunc
    where
        T: VisitableChildren<T> +
            AsNodeRef<nonterminal_return_stmt> +
            AsNodeRef<nonterminal_fn_def>,
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
            if let Some(_tree) = visited.downcast::<nonterminal_return_stmt>() {
                if self.func_depth == 0 {
                    self.violations.push(self.path.clone());
                }
            } else if let Some(_tree) = visited.downcast::<nonterminal_fn_def>() {
                self.func_depth += 1;
                // Visit the function, then decrease depth.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                visitor.func_depth -= 1;
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            }
            let result = visited.visit_each(self);
            let Ok(ControlFlow::Continue(mut visitor)) = result;
            visitor.path.pop_back();
            Ok(ControlFlow::Continue(visitor))
        }
    }
    // ================= end of Returns only inside functions.

    #[cfg(test)]
    mod test {
        use crate::lang;
        use fandango_runtime::operators::DepthLimiter;
        use alloc::boxed::Box;
        use core::error::Error;
        use core::ops::ControlFlow;
        use fandango::generation::Generated;
        use fandango::tuple_list::tuple_list;
        use fandango::typing::{Structured};
        use fandango::visitor::{Visitor, VisitorMut};
        use fandango::visitor::navigation::GoTo;
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use alloc::string::String;
        use fandango::visitor::write::WriteVisitor;
        use alloc::vec::Vec;

        #[test]
        fn check_def_use_constraint_and_fix() -> Result<(), Box<dyn Error>> {
            extern crate std;
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(lang::nonterminal_start::ROOT.inner(), 50));
            let mut diff_count = 0;
            let mut tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
            
            // Run this 10 times and print to see violations and fixes.
            let i = 0;
            for i in 0..10 {
                tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorDefUse { violations, defined_vars, .. })) =
                    lang::ConstraintVisitorDefUse::default().visit(&mut tree, 0);

                let total_violations = violations.len();

                for mut violation in violations {
                    violation.pop_front();

                    assert!(matches!(
                        tree.go_to(0, violation.clone())?,
                        lang::Type::nonterminal_var_access(_)
                    ));

                    let lang::Type::nonterminal_var_access(records) =
                        tree.go_to(0, violation.clone())?
                    else {
                        unreachable!("We are inspecting the records directly.");
                    };
                    let var_name_str = String::from_utf8(
                        WriteVisitor::new(Vec::new())
                            .visit(records, 0)
                            .unwrap()
                            .continue_value()
                            .unwrap()
                            .output(),
                    ).unwrap();

                    assert!(!defined_vars.contains(&var_name_str), "at {violation:?}, found: {var_name_str}");

                    diff_count += 1;
                }

                std::println!("Found {total_violations} violations in iteration {i}, total so far {diff_count}.");
                std::println!("Program:\n{}", String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(&tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output(),
                ).unwrap());

                // Now fix the tree.
                let _ = lang::ConstraintFixerDefUse {
                    sampler: &mut rng,
                    generator: &mut generators,
                    defined_vars: &mut alloc::collections::BTreeMap::new(),
                }
                .visit_mut(&mut tree, 0)?;
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorDefUse { violations, .. })) =
                    lang::ConstraintVisitorDefUse::default().visit(&mut tree, 0);

                std::println!("After fixing, found {} violations.", violations.len());
                std::println!("Fixed program:\n{}", String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(&tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output(),
                ).unwrap());
            }

            // ok at this point we found a program that worked.
            std::println!("Found {diff_count} violations across {i} programs.");
            std::println!("Final program:\n{}", String::from_utf8(
                WriteVisitor::new(Vec::new())
                    .visit(&tree, 0)
                    .unwrap()
                    .continue_value()
                    .unwrap()
                    .output(),
            ).unwrap());
            Ok(())
        }

        #[test]
        fn check_ret_in_fn_constraint() -> Result<(), Box<dyn Error>> {
            extern crate std;
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(lang::nonterminal_start::ROOT.inner(), 50));
            // Generate 50 programs and check for violations.
            for i in 0..50 {
                let tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorReturnInFunc { 
                    violations, .. 
                })) = lang::ConstraintVisitorReturnInFunc::default().visit(&tree, 0);
                std::println!("==============================");
                std::println!("Program {i} has {} return-in-fn violations.", violations.len());
                // Print the program.
                std::println!("Program:\n{}", String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(&tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output(),
                ).unwrap());
            }
            Ok(())
        }
    }
}

pub use defs::*;