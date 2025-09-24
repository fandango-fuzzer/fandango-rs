// This file will be an attempt at using fandango-rs as a compiler tester. 

// General soundness TODOs:
// 1. Possible issue with def-use but only in the same scope. Sub-scopes work, but not same scope.

// General completeness TODOs:
// 1. Nested struct definitions (need to adjust grammar probably?)

#[cfg(not(feature = "static_defs"))]
mod defs {
    use core::convert::Infallible;
    use fandango::Fandango;

    /// Base for the lang grammar stored in lang.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/c_lang.fan", parse = false, dynamic = true)]
    pub struct CLang(Infallible);
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
    
    /// Base for the C language grammar stored in c_lang.fan.
    #[derive(Fandango)]
    #[fandango(grammar = "grammars/c_lang.fan", parse = false)]
    pub struct CLang(Infallible);
    
    // Helpful definitions.
    // First, a way to track lexical scope.
    // This will be a mapping of (var_name, scope_trace) -> var_type
    // where scope_trace is a Vec of usize indices representing the path to the scope.
    // This way, we can check if a variable is defined in the current scope or any outer scope.
    // Note: This is a simplified approach and may not cover all edge cases in a real compiler.
    // For now, we will use a BTreeMap for simplicity, but a more efficient data structure may be needed for large programs.
    type ScopeTrace = Vec<usize>;
    
    // Now, a sort of symbol table to track variable definitions.
    type VarSymbolTable = alloc::collections::BTreeMap<(nonterminal_var_name, ScopeTrace), nonterminal_type>;

    // A helper function to match two scope traces.
    // A scope trace A matches scope trace B if A is a prefix of B.
    fn scope_trace_matches(a: &ScopeTrace, b: &ScopeTrace) -> bool {
        if a.len() > b.len() {
            return false;
        }
        for (i, val) in a.iter().enumerate() {
            if *val != b[i] {
                return false;
            }
        }
        true
    }

    // A helper function to get the last definition of a variable in the current or outer scopes.
    fn get_var_definition<'a>(symbol_table: &'a VarSymbolTable, var_name: &nonterminal_var_name, current_scope: &ScopeTrace) -> Option<&'a nonterminal_type> {
        // We will look for the variable in the current scope and then in outer scopes.
        for scope_len in (0..=current_scope.len()).rev() {
            let scope_prefix = &current_scope[0..scope_len];
            if let Some(var_type) = symbol_table.get(&(var_name.clone(), scope_prefix.to_vec())) {
                return Some(var_type);
            }
        }
        None
    }

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
        AsNodeRef<nonterminal_decl> +
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
    //

    // First, a visitor that collects declarations, which will be used by the main visitors.
    /// Collects all variable declarations in the program (incl. parameters), pass this to any
    /// visitor that needs it.
    /// Also collects function definitions and their parameter counts, for use in function call checks.
    /// Also also collects struct definitions, mapping struct names to their field names and types.
    #[derive(Debug, Default)]
    pub struct DeclarationCollector {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The current scope_id, to track variable scopes.
        pub scope_id: usize,
        /// The current scope depth, to track variable scopes.
        pub scope_trace: Vec<usize>,
        /// The set of currently defined variables, (var_name, scope) -> var_type
        pub defined_vars: VarSymbolTable,
        /// The set of currently defined functions. (fn_name, scope) -> Vec<param_type>
        pub func_param_counts: alloc::collections::BTreeMap<(nonterminal_fn_name, ScopeTrace), Vec<nonterminal_type>>,
        /// The set of currently defined structs. struct_name -> Vec<(field_name, field_type)>
        pub struct_defs: alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
    }

    // TODO: Also collect struct names?
    impl<T> Visitor<T> for DeclarationCollector
    where 
        T: VisitableChildren<T> +
        AsNodeRef<nonterminal_decl> +
        AsNodeRef<nonterminal_var_name> +
        AsNodeRef<nonterminal_fn_def> + 
        AsNodeRef<nonterminal_fn_name> +
        AsNodeRef<nonterminal_param_name> +
        AsNodeRef<nonterminal_struct_def> +
        AsNodeRef<nonterminal_struct_name> +
        AsNodeRef<nonterminal_field_name> +
        AsNodeRef<nonterminal_type> +
        AsNodeRef<nonterminal_field_def_list> +
        AsNodeRef<nonterminal_field_def_list_e> +
        AsNodeRef<nonterminal_param_list> +
        AsNodeRef<nonterminal_param> +,
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
            // First, check if we are in a situation where we need to update the scope trace.
            if let Some(tree) = visited.downcast::<nonterminal_fn_def>() {
                // First, record the function name and its parameter count.
                let fn_name = tree.nth::<0>().nth::<4>().clone();
                // Get the param_list_e.
                let param_list_e = tree.nth::<0>().nth::<6>();
                // Is param_list_e <e> or <param_list>?
                let param_type_list = if let Some(_e) = param_list_e.nth::<0>().nth::<1>() {
                    Vec::new()
                } else {
                    // In this case, we know it's param_list.
                    let mut current = param_list_e.nth::<0>().nth::<0>();
                    let mut param_type_list_inner = Vec::new();
                    while let Some(pl) = current {
                        match pl.nth::<0>() {
                            // Variant 0, single param.
                            nonterminal_param_list_0::variant_0(param) => {
                                let param_type = param.nth::<0>().nth::<0>().clone();
                                param_type_list_inner.push(param_type);
                                current = None;
                            },
                            // Variant 1, param followed by more params.
                            nonterminal_param_list_0::variant_1(seq) => {
                                let (param, _, _, rest) = seq.children();
                                let param_type = param.nth::<0>().nth::<0>().clone();
                                param_type_list_inner.push(param_type);
                                current = Some(rest);
                            }
                        }
                    }
                    param_type_list_inner
                };
                // Save the function definition using the node as key.
                self.func_param_counts.insert((fn_name, self.scope_trace.clone()), param_type_list);
                // Update the scope trace.
                self.scope_id += 1;
                self.scope_trace.push(self.scope_id);
                // Visit the function, then decrease depth.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                visitor.scope_trace.pop();
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            } // Functions are currently the only scope-increasing construct.

            if let Some(decl_tree) = visited.downcast::<nonterminal_decl>() {
                let var_decl_name = decl_tree.nth::<0>().nth::<2>().clone();
                let var_decl_type = decl_tree.nth::<0>().nth::<0>().clone();
                self.defined_vars.insert((var_decl_name, self.scope_trace.clone()), var_decl_type);
            } /* Check now for param_name */
            else if let Some(param_tree) = visited.downcast::<nonterminal_param>() {
                let type_inside = param_tree.nth::<0>().nth::<0>().clone();
                let var_name_inside = param_tree.nth::<0>().nth::<2>().nth::<0>().clone();
                // The parameters should be scoped properly.
                self.defined_vars.insert((var_name_inside, self.scope_trace.clone()), type_inside);
            } /* Check now for struct definitions */
            else if let Some(struct_tree) = visited.downcast::<nonterminal_struct_def>() {
                let struct_name = struct_tree.nth::<0>().nth::<2>().clone();
                let mut fields = Vec::new();
                // Get the field_def_list.
                let field_def_list_e = struct_tree.nth::<0>().nth::<6>();
                // Is field_def_list_e <e> or <field_def_list>?
                if let Some(_e) = field_def_list_e.nth::<0>().nth::<1>() {
                    // No fields.
                } else {
                    // In this case, we know it's field_def_list.
                    let mut current = field_def_list_e.nth::<0>().nth::<0>();
                    while let Some(fdl) = current {
                        // Get the field name and type.
                        match fdl.nth::<0>() {
                            // Variant 0, single field.
                            nonterminal_field_def_list_0::variant_0(field_def) => {
                                let field_name = field_def.nth::<2>().clone();
                                let field_type = field_def.nth::<0>().clone();
                                fields.push((field_name, field_type));
                                current = None;
                            },
                            // Variant 1, field followed by more fields.
                            nonterminal_field_def_list_0::variant_1(seq) => {
                                let (field_type, _, field_name, _, _, rest) = seq.children();
                                fields.push((field_name.clone(), field_type.clone()));
                                current = Some(rest);
                            }
                        }
                        // This way doesn't work b/c nth can't quite statically resolve that 0 and 2 are always present
                        // regardless of variant.
                        // let field_type = fdl.nth::<0>().nth::<0>().clone();
                        // let field_name = fdl.nth::<0>().nth::<2>().clone();
                        // fields.push((field_name, field_type));
                        // current = match fdl.nth::<0>() {
                        //     nonterminal_field_def_list_0::variant_0(_) => None,
                        //     nonterminal_field_def_list_0::variant_1(seq) => {
                        //         let (_, _, _, _, _, rest) = seq.children();
                        //         Some(rest)
                        //     }
                        // };
                    }
                }
                self.struct_defs.insert(struct_name, fields);
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
        /// The current scope_id, to track variable scopes.
        pub scope_id: usize,
        /// The current scope depth, to track variable scopes.
        pub scope_trace: Vec<usize>,
        /// The set of currently defined variables, (var_name, scope) -> var_type
        pub defined_vars: VarSymbolTable,
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
                self.scope_id += 1;
                self.scope_trace.push(self.scope_id);
                // Visit the function, then decrease depth.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                visitor.scope_trace.pop();
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            } // Functions are currently the only scope-increasing construct.

            if let Some(tree) = visited.downcast::<nonterminal_var_access>() {
                let var_name_accessed = tree.nth::<0>().clone();
                if get_var_definition(&self.defined_vars, &var_name_accessed, &self.scope_trace).is_none() {
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

    // ================= Begin: Struct use must match declaration.
    // Constraint visitor.
    /// Basic struct-access constraint visitor.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorStructAccess {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The current scope level.
        pub scope_depth: usize,
        /// The current struct definitions, mapping struct names to their field names and types.
        /// This should be initialized by a prior pass of DeclarationCollector.
        pub struct_defs: alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
        /// The current variable definitions, mapping variable names to their types.
        pub var_defs: alloc::collections::BTreeMap<(nonterminal_var_name, usize), nonterminal_type>,
    }

    impl<T> Visitor<T> for ConstraintVisitorStructAccess
    where
        T: VisitableChildren<T> +
            AsNodeRef<nonterminal_struct_access> +
            AsNodeRef<nonterminal_var_access> +
            AsNodeRef<nonterminal_field_name> +
            AsNodeRef<nonterminal_struct_name> +
            AsNodeRef<nonterminal_expr> +
            AsNodeRef<nonterminal_var_name> +
            AsNodeRef<nonterminal_type> + 
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

            // First, check if we are in a situation where we need to increase scope depth.
            if let Some(_tree) = visited.downcast::<nonterminal_fn_def>() {
                self.scope_depth += 1;
                // Visit the function, then decrease depth.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                visitor.scope_depth -= 1;
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            }

            // Ok, actual logic.
            if let Some(tree) = visited.downcast::<nonterminal_struct_access>() {
                // Get the struct name.
                // First, the var name.
                let var_name = tree.nth::<0>().nth::<0>().clone();
                // Look up the var name in the var_defs to get its type.
                // Note: (Lowkey a TODO) This should look up the scope, not necessarily that it's the exact scope depth.
                let pot_var_type = self.var_defs.get(&(var_name, self.scope_depth)).cloned();

                match pot_var_type {
                    Some(var_type) => {
                        // We have a variable type, check if it's a struct type.
                        // Get the alternative.
                        let var_type_0 = var_type.nth::<0>();
                        if let nonterminal_type_0::variant_1(struct_type) = var_type_0 {
                            let struct_name = struct_type.nth::<0>().nth::<2>().clone();
                            // Now get the field name being accessed.
                            let field_name = tree.nth::<0>().nth::<2>().clone();
                            // Look up the struct definition to see if the field exists.
                            if let Some(fields) = self.struct_defs.get(&struct_name) {
                                if !fields.iter().any(|(fname, _ftype)| fname == &field_name) {
                                    // Field not found in struct definition, violation.
                                    self.violations.push(self.path.clone());
                                }
                            } else {
                                // Struct not found, violation.
                                self.violations.push(self.path.clone());
                            }
                        } else {
                            // Variable is not of struct type, violation.
                            self.violations.push(self.path.clone());
                        }
                    },
                    None => {
                        // Variable not found, violation.
                        self.violations.push(self.path.clone());
                    }
                }
            }
            let mut result = visited.visit_each(self);
            if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                visitor.path.pop_back();
            }
            result
        }
    }

    // ================= Functions called with correct number of arguments.
    // Constraint visitor.
    /// Basic function-arguments-count constraint visitor.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorFuncArgCount {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The current scope level.
        pub scope_depth: usize,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The current function definitions, mapping function names and scopes to their parameter counts.
        /// This should be initialized by a prior pass of DeclarationCollector.
        pub func_defs: alloc::collections::BTreeMap<(nonterminal_fn_name, usize), usize>,
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

            // First, check if we are in a situation where we need to increase scope depth.
            if let Some(_tree) = visited.downcast::<nonterminal_fn_def>() {
                self.scope_depth += 1;
                // Visit the function, then decrease depth.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                visitor.scope_depth -= 1;
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            }
            
            if let Some(tree) = visited.downcast::<nonterminal_fn_call>() {
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
                if let Some(expected_count) = self.func_defs.get(&(fn_name, self.scope_depth)) {
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

    // ================ Type checking.
    // Constraint visitor.
    /// Basic type-checking constraint visitor.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorTypeCheck {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The current scope path.
        pub scope_trace: Vec<usize>,
        /// The current scope depth.
        pub scope_depth: usize,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The current variable definitions, mapping variable names and scopes to their types.
        /// This should be initialized by a prior pass of DeclarationCollector.
        pub var_defs: alloc::collections::BTreeMap<(nonterminal_var_name, Vec<usize>), nonterminal_type>,
        /// The current function definitions, mapping function names and scopes to their return types.
        /// This should be initialized by a prior pass of DeclarationCollector.
        pub func_defs: alloc::collections::BTreeMap<(nonterminal_fn_name, Vec<usize>), nonterminal_type>,
        /// The current struct definitions, mapping struct names to their field names and types.
        /// This should be initialized by a prior pass of DeclarationCollector.
        pub struct_defs: alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
    }

    // impl<T> Visitor<T> for ConstraintVisitorTypeCheck
    // where
    //     T: VisitableChildren<T> +
    //         AsNodeRef<nonterminal_assignment> +
    //         AsNodeRef<nonterminal_expr> +
    //         AsNodeRef<nonterminal_var_access> +
    //         AsNodeRef<nonterminal_var_name> +
    //         AsNodeRef<nonterminal_decl> +
    //         AsNodeRef<nonterminal_type> +
    //         AsNodeRef<nonterminal_fn_def> +
    //         AsNodeRef<nonterminal_fn_name> +
    //         AsNodeRef<nonterminal_return_stmt> +
    //         AsNodeRef<nonterminal_struct_access> +
    //         AsNodeRef<nonterminal_field_name> +
    //         AsNodeRef<nonterminal_struct_name>,
    // {
    //     type Continue = Self;
    //     type Break = Infallible;
    //     type Error = Infallible;

    //     fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
    //     where
    //         N: Node<Type<'program> = T>,
    //         T: From<&'program N> + AsNodeRef<N>,
    //     {
    //         self.path.push_back(idx);
    //         let visited = node.opaque(); 

    //         // First, check if we are in a situation where we need to increase scope depth.
    //         if let Some(_tree) = visited.downcast::<nonterminal_fn_def>() {
    //             self.scope_depth += 1;
    //             self.scope_trace.push(self.scope_depth);
    //             // Visit the function, then decrease depth.
    //             let result = visited.visit_each(self);
    //             let Ok(ControlFlow::Continue(mut visitor)) = result;
    //             visitor.scope_depth -= 1;
    //             visitor.scope_trace.pop();
    //             visitor.path.pop_back();
    //             return Ok(ControlFlow::Continue(visitor));
    //         }

    //         if let Some(tree) = visited.downcast::<nonterminal_decl>() {
    //             let var_name = tree.nth::<0>().nth::<2>().clone();
    //             let var_type = tree.nth::<0>().nth::<0>().clone();
    //             self.var_defs.insert((var_name, self.scope_trace.clone()), var_type);
    //         } else if let Some(tree) = visited.downcast::<nonterminal_assignment>() {
    //             let var_access = tree.nth::<0>().nth::<0>();
    //             let expr = tree.nth::<0>().nth::<2>();
    //             // Get the variable name from the var_access.
    //             let var_name = var_access.nth::<0>().clone();
    //             // Look up the variable type.
    //             let var_type = self.var_defs.get(&(var_name, self.scope_trace.clone()));
    //             // Get the expression type.
    //             let expr_type = infer_expr_type(expr, &self.var_defs, &self.func_defs, &self.struct_defs, &self.scope_trace);
    //             // Compare types.
    //             if let (Some(vt), Some(et)) = (var_type, expr_type) {
    //                 if !types_compatible(vt, &et, &self.struct_defs) {
    //                     self.violations.push(self.path.clone());
    //                 }
    //             } else {
    //                 // Either variable or expression type could not be determined, consider it a violation.
    //                 self.violations.push(self.path.clone());
    //             }
    //         } else if let Some(tree) = visited.downcast::<nonterminal_return_stmt>() {
    //             let expr = tree.nth::<0>().nth::<1>();
    //             // Get the expression type.
    //             let expr_type = infer_expr_type(expr, &self.var_defs, &self.func_defs, &self.struct_defs, &self.scope_trace);
    //             // Get the function return type.
    //             if let Some((fn_name, _)) = self.path.iter().rev().find_map(|&i| {
    //                 let ancestor = visited.go_to(0, self.path.iter().cloned().take(i + 1).collect()).ok()?;
    //                 if let Some(fn_def) = ancestor.downcast::<nonterminal_fn_def>() {
    //                     Some((fn_def.nth::<0>().nth::<4>().clone(), ()))
    //                 } else {
    //                     None
    //                 }
    //             }) {
    //                 let fn_return_type = self.func_defs.get(&(fn_name, self.scope_trace.clone()));
    //                 // Compare types.
    //                 if let (Some(rt), Some(et)) = (fn_return_type, expr_type) {
    //                     if !types_compatible(rt, &et, &self.struct_defs) {
    //                         self.violations.push(self.path.clone());
    //                     }
    //                 } else {
    //                     // Either function return type or expression type could not be determined, consider it a violation.
    //                     self.violations.push(self.path.clone());
    //                 }
    //             } else {
    //                 // Return statement not inside a function, already handled by another visitor.
    //             }
    //         } else if let Some(tree) = visited.downcast::<nonterminal_struct_access>() {
    //             // Get the var name.
    //             let var_name = tree.nth::<0>().nth::<0>().clone();
    //             // Look up the variable type.
    //             let var_type = self.var_defs.get(&(var_name, self.scope_trace.clone()));
    //             // Get the field name being accessed.
    //             let field_name = tree.nth::<0>().nth::<2>().clone();
    //             // Check if the variable type is a struct and if it has the field.
    //             if let Some(vt) = var_type {
    //                 if let nonterminal_type_0::variant_1(struct_type) = vt.nth::<0>() {
    //                     let struct_name = struct_type.nth::<0>().nth::<2>().clone();
    //                     if let Some(fields) = self.struct_defs.get(&struct_name) {
    //                         if !fields.iter().any(|(fname, _ftype)| fname == &field_name) {
    //                             // Field not found in struct definition, violation.
    //                             self.violations.push(self.path.clone());
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    #[cfg(test)]
    mod test {
        use crate::clang as lang;
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
        fn check_def_use_constraint_and_fix_c() -> Result<(), Box<dyn Error>> {
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
        fn check_ret_in_fn_constraint_c() -> Result<(), Box<dyn Error>> {
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
        fn check_fn_call_constraint_c() -> Result<(), Box<dyn Error>> {
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

        #[test]
        fn check_struct_def_and_access_c() -> Result<(), Box<dyn Error>> {
            extern crate std;
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(lang::nonterminal_start::ROOT.inner(), 50));
            // Generate 50 programs and check for violations.
            for i in 0..200 {
                let tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
                let Ok(ControlFlow::Continue(lang::DeclarationCollector { 
                    struct_defs, .. 
                })) = lang::DeclarationCollector::default().visit(&tree, 0);

                std::println!("==============================");
                std::println!("Program {i} has {} struct definitions.", struct_defs.len());

                let Ok(ControlFlow::Continue(lang::ConstraintVisitorStructAccess { 
                    violations, .. 
                })) = lang::ConstraintVisitorStructAccess {
                    struct_defs,
                    ..Default::default()
                }.visit(&tree, 0);

                std::println!("Program {i} has {} struct-access violations.", violations.len());
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