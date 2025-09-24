// This file will be an attempt at using fandango-rs as a compiler tester. 

#[cfg(not(feature = "static_defs"))]
mod defs {
    use core::convert::Infallible;
    use fandango::Fandango;

    /// Base for the lang grammar stored in lang.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/lua_lang.fan", parse = false, dynamic = true)]
    pub struct LuaLang(Infallible);
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
    use num_rational::Ratio;
    
    /// Base for the lang grammar stored in lang.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/lua_lang.fan", parse = false)]
    pub struct LuaLang(Infallible);

    // ================= Def before use.
    // Constraint visitor.
    //

    // First, a visitor that collects declarations, which will be used by the main visitor.
    #[derive(Debug, Default)]
    pub struct DeclarationCollector {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The current scope depth, to track variable scopes.
        pub scope_depth: usize,
        /// The set of currently defined variables, mapping names to nodes.
        pub defined_vars: alloc::collections::BTreeSet<(nonterminal_var_name, usize)>,
    }

    impl<T> Visitor<T> for DeclarationCollector
    where 
        T: VisitableChildren<T> +
        AsNodeRef<nonterminal_decl> +
        AsNodeRef<nonterminal_var_name> +
        AsNodeRef<nonterminal_fn_def> + 
        AsNodeRef<nonterminal_param_name>,
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
            // First, check if we are in a situation where we need to increase scope depth.
            if let Some(_tree) = visited.downcast::<nonterminal_fn_def>() {
                self.scope_depth += 1;
                // Visit the function, then decrease depth.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                visitor.scope_depth -= 1;
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            } // Functions are currently the only scope-increasing construct.

            if let Some(decl_tree) = visited.downcast::<nonterminal_decl>() {
                let var_decl_name = decl_tree.nth::<0>().nth::<2>().clone();
                self.defined_vars.insert((var_decl_name, self.scope_depth));
            } /* Check now for param_name */ else if let Some(param_tree) = visited.downcast::<nonterminal_param_name>() {
                let var_name_inside = param_tree.nth::<0>().clone();
                // The parameters should be scoped properly.
                self.defined_vars.insert((var_name_inside, self.scope_depth));
            }
            let mut result = visited.visit_each(self);
            if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                visitor.path.pop_back();
            }
            result
        }
    }

    /// Basic def-before-use constraint visitor.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorDefUse {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The current scope depth, to track variable scopes.
        pub scope_depth: usize,
        /// The set of currently defined variables, mapping names to nodes.
        pub defined_vars: alloc::collections::BTreeSet<(nonterminal_var_name, usize)>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The list of places where violations _could_ have occurred, for computing the violation ratio.
        pub paths_to_all_accesses: Vec<VecDeque<usize>>,
    }
    
    impl Checker for ConstraintVisitorDefUse {
        fn violations(self) -> Violations {
            if self.paths_to_all_accesses.is_empty() {
                // No violations.
                Violations::new(Ratio::new(1, 1), self.violations)
            } else {
                Violations::new(Ratio::new(self.violations.len(), self.paths_to_all_accesses.len()), self.violations)
            }
        }
    }

    impl<T> Visitor<T> for ConstraintVisitorDefUse
    where 
        T: VisitableChildren<T> +
        AsNodeRef<nonterminal_var_access> +
        AsNodeRef<nonterminal_assignment> +
        AsNodeRef<nonterminal_decl> +
        AsNodeRef<nonterminal_fn_def> +
        AsNodeRef<nonterminal_var_name>,
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

            // Assume that defined_vars is correctly populated at the start of the visit.

            // Check if we are in a situation where we need to increase scope depth.
            if let Some(_tree) = visited.downcast::<nonterminal_fn_def>() {
                self.scope_depth += 1;
                // Visit the function, then decrease depth.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                visitor.scope_depth -= 1;
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            } // Functions are currently the only scope-increasing construct.

            if let Some(tree) = visited.downcast::<nonterminal_var_access>() {
                let var_name_accessed = tree.nth::<0>().clone();
                // Look for it in the defined vars.
                if !self.defined_vars.iter().any(|(name, depth)| {
                    name == &var_name_accessed && *depth <= self.scope_depth
                }) {
                    // We have a violation, the variable was not defined in the current or any outer scope.
                    self.violations.push(self.path.clone());
                }
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

    // ================= Functions called with correct number of arguments.

    // Constraint visitor.
    /// Basic function-arguments-count constraint visitor.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorFuncArgCount {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The current function definitions, mapping function names to their parameter counts.
        pub func_defs: alloc::collections::BTreeMap<nonterminal_fn_name, usize>,
    }   

    impl<T> Visitor<T> for ConstraintVisitorFuncArgCount
    where
        T: VisitableChildren<T> +
            AsNodeRef<nonterminal_fn_call> +
            AsNodeRef<nonterminal_fn_def> +
            AsNodeRef<nonterminal_arg_list> +
            AsNodeRef<nonterminal_param_list> +
            AsNodeRef<nonterminal_e> +
            AsNodeRef<nonterminal_param_list_e>,
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
            if let Some(tree) = visited.downcast::<nonterminal_fn_def>() {
                let fn_name = tree.nth::<0>().nth::<2>().clone();
                // Get the param_list_e.
                let param_list_e = tree.nth::<0>().nth::<4>();
                // Is param_list_e <e> or <param_list>?
                let param_count = if let Some(_e) = param_list_e.nth::<0>().nth::<1>() {
                    0
                } else {
                    // In this case, we know it's param_list.
                    let mut count = 0;
                    let mut current = param_list_e.nth::<0>().nth::<0>();
                    while let Some(pl) = current {
                        count += 1;
                        current = match pl.nth::<0>() {
                            nonterminal_param_list_0::variant_0(_) => None,
                            nonterminal_param_list_0::variant_1(seq) => {
                                let (_, _, _, rest) = seq.children();
                                Some(rest)
                            }
                        };
                    }
                    count
                };
                // Save the function definition using the node as key.
                self.func_defs.insert(fn_name, param_count);
            } else if let Some(tree) = visited.downcast::<nonterminal_fn_call>() {
                let fn_name = tree.nth::<0>().nth::<0>().clone();
                // Get the arg_list_e.
                let arg_list_e = tree.nth::<0>().nth::<2>();
                // Is arg_list_e <e> or <arg_list>?
                let arg_count = if let Some(_e) = arg_list_e.nth::<0>().nth::<1>() {
                    0
                } else {
                    // In this case, we know it's arg_list.
                    let mut count = 0;
                    let mut current = arg_list_e.nth::<0>().nth::<0>();
                    while let Some(al) = current {
                        count += 1;
                        current = match al.nth::<0>() {
                            nonterminal_arg_list_0::variant_0(_) => None,
                            nonterminal_arg_list_0::variant_1(seq) => {
                                let (_, _, _, rest) = seq.children();
                                Some(rest)
                            }
                        };
                    }
                    count
                };
                // Check if the function name is in the definitions.
                if let Some(expected_count) = self.func_defs.get(&fn_name) {
                    if *expected_count != arg_count {
                        self.violations.push(self.path.clone());
                    }
                } else {
                    // Function not defined, consider it a violation.
                    self.violations.push(self.path.clone());
                }
            }
            let mut result = visited.visit_each(self);
            if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                visitor.path.pop_back();
            }
            result
        }
    }

    // ================ end of Functions called with correct number of arguments.

    #[cfg(test)]
    mod test {
        use crate::lualang as lang;
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
            
            // Run this 100 times and print to see violations and fixes.
            let i = 0;
            for i in 0..100 {
                tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);

                std::println!("==============================");

                // First collect declarations.
                let Ok(ControlFlow::Continue(decl_visitor)) =
                    lang::DeclarationCollector::default().visit(&tree, 0);
                // let mut tree = tree;
                let defined_vars = decl_visitor.defined_vars;
                std::println!("Found {} defined variables.", defined_vars.len());

                // Now check for violations.   
                
                let mut def_use_visitor = lang::ConstraintVisitorDefUse::default();
                // Set the defined vars.
                def_use_visitor.defined_vars = defined_vars.clone();
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorDefUse { violations, .. })) =
                    def_use_visitor.visit(&tree, 0);
                std::println!("Program {i} has {} def-before-use violations.", violations.len());
                
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
                // let _ = lang::ConstraintFixerDefUse {
                //     sampler: &mut rng,
                //     generator: &mut generators,
                //     defined_vars: &mut alloc::collections::BTreeMap::new(),
                // }
                // .visit_mut(&mut tree, 0)?;
                // let Ok(ControlFlow::Continue(lang::ConstraintVisitorDefUse { violations, .. })) =
                //     lang::ConstraintVisitorDefUse::default().visit(&mut tree, 0);

                // std::println!("After fixing, found {} violations.", violations.len());
                // std::println!("Fixed program:\n{}", String::from_utf8(
                //     WriteVisitor::new(Vec::new())
                //         .visit(&tree, 0)
                //         .unwrap()
                //         .continue_value()
                //         .unwrap()
                //         .output(),
                // ).unwrap());
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

        #[test]
        fn check_fn_call_constraint() -> Result<(), Box<dyn Error>> {
            extern crate std;
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(lang::nonterminal_start::ROOT.inner(), 50));
            // Generate 50 programs and check for violations.
            for i in 0..200 {
                let tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorFuncArgCount { 
                    violations, .. 
                })) = lang::ConstraintVisitorFuncArgCount::default().visit(&tree, 0);
                std::println!("==============================");
                std::println!("Program {i} has {} fn-call-arg-count violations.", violations.len());
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