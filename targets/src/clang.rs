// This file will be an attempt at using fandango-rs as a compiler tester. 

// General soundness TODOs:
// 1. Possible issue with def-use but only in the same scope. Sub-scopes work, but not same scope.
<<<<<<< HEAD
// 2. Add constraint for no duplicate fields.
=======

// General completeness TODOs:
// 1. Nested struct definitions (need to adjust grammar probably?)
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)

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
<<<<<<< HEAD
=======
    use fandango_runtime::measurement::Violations;
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
    use rand::Rng;
    use alloc::vec;
    
    // For the experiments
    use fandango_runtime::evolvers::basic::BasicHook;
    use fandango_runtime::measurement::Violations;
    use fandango_runtime::operators::Checker;
    use num_rational::Ratio;
=======
    use fandango_runtime::operators::Checker;
    use rand::Rng;
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
    
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
    
<<<<<<< HEAD
    // TODO: These could easily be consolidated.
    // Now, a sort of symbol table to track variable definitions.
    type VarSymbolTable = alloc::collections::BTreeMap<(nonterminal_var_name, ScopeTrace), nonterminal_type>;

    // Also for function definitions, mapping (fn_name, scope_trace) -> Vec<nonterminal_type> (parameter types + return type)
    type FuncSymbolTable = alloc::collections::BTreeMap<(nonterminal_fn_name, ScopeTrace), Vec<nonterminal_type>>;

    // A helper function which gets the function name and type given a scope trace.
    // Should find the innermost function definition that matches the scope trace.
    fn get_current_function<'a>(symbol_table: &'a FuncSymbolTable, current_scope: &ScopeTrace) -> Option<(&'a nonterminal_fn_name, &'a Vec<nonterminal_type>)> {
        // We will look for the function in the current scope and then in outer scopes.
        for scope_len in (0..=current_scope.len()).rev() {
            let scope_prefix = current_scope[0..scope_len].to_vec();
            for ((fn_name, fn_scope), param_types) in symbol_table.iter() {
                if scope_trace_matches(fn_scope, &scope_prefix) {
                    return Some((fn_name, param_types));
                }
            }
        }
        None
        
    }

=======
    // Now, a sort of symbol table to track variable definitions.
    type VarSymbolTable = alloc::collections::BTreeMap<(nonterminal_var_name, ScopeTrace), nonterminal_type>;

>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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

<<<<<<< HEAD
    fn get_func_definition<'a>(symbol_table: &'a FuncSymbolTable, fn_name: &nonterminal_fn_name, current_scope: &ScopeTrace) -> Option<&'a Vec<nonterminal_type>> {
        // We will look for the function in the current scope and then in outer scopes.
        for scope_len in (0..=current_scope.len()).rev() {
            let scope_prefix = &current_scope[0..scope_len];
            if let Some(param_types) = symbol_table.get(&(fn_name.clone(), scope_prefix.to_vec())) {
                return Some(param_types);
            }
        }
        None
    }

    // Constraint (testing) for only one struct, at most 5 fields.
    pub struct KeepReasonableStructVisitor {
        pub structs_seen: usize,
        pub fields_seen: usize,
        /// Path
        pub path: VecDeque<usize>,
        /// Violations
        pub violations: Vec<VecDeque<usize>>,
        /// Paths that passed checks, for ratio calculation.
        pub paths_to_passed_checks: Vec<VecDeque<usize>>,
    }

    impl Checker for KeepReasonableStructVisitor {
        fn violations(self) -> Violations {
            if self.structs_seen == 0 && self.fields_seen == 0 {
                // No structs, no violations.
                return Violations::new(Ratio::new(1, 1), self.violations);
            }
            Violations::new(
                Ratio::new(self.violations.len(), self.structs_seen.max(1) + self.fields_seen.max(1)), // If no structs, no violations.
                self.violations,
            )
        }
    }

    impl Default for KeepReasonableStructVisitor {
        fn default() -> Self {
            Self {
                structs_seen: 0,
                fields_seen: 0,
                path: VecDeque::new(),
                violations: Vec::new(),
                paths_to_passed_checks: Vec::new(),
=======
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
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
            }
        }
    }

<<<<<<< HEAD
    impl<T> Visitor<T> for KeepReasonableStructVisitor
    where
        T: VisitableChildren<T> +
            AsNodeRef<nonterminal_struct_def> +
            AsNodeRef<nonterminal_field_def_list> +
            AsNodeRef<nonterminal_field_def_list_e>,
=======
    impl<T> Visitor<T> for ConstraintVisitorAtLeastOneVarAlsoDefUse
    where 
        T: VisitableChildren<T> + 
        AsNodeRef<nonterminal_var_access> +
        AsNodeRef<nonterminal_assignment> +
        AsNodeRef<nonterminal_decl> +
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
            if let Some(struct_def) = visited.downcast::<nonterminal_struct_def>() {
                self.structs_seen += 1;
                if self.structs_seen > 1 {
                    self.violations.push(self.path.clone());
                } else {
                    self.paths_to_passed_checks.push(self.path.clone());
                }
                // Now count fields.
                let field_def_list_e = struct_def.nth::<0>().nth::<6>();
                if let Some(_e) = field_def_list_e.nth::<0>().nth::<1>() {
                    // No fields.
                } else {
                    // In this case, we know it's field_list.
                    let mut current = field_def_list_e.nth::<0>().nth::<0>();
                    while let Some(fdl) = current {
                        self.fields_seen += 1;
                        if self.fields_seen > 5 {
                            self.violations.push(self.path.clone());
                        } else {
                            self.paths_to_passed_checks.push(self.path.clone());
                        }
                        match fdl.nth::<0>() {
                            // Variant 0, single field.
                            nonterminal_field_def_list_0::variant_0(_) => {
                                current = None;
                            },
                            // Variant 1, field followed by more fields.
                            nonterminal_field_def_list_0::variant_1(seq) => {
                                let (_, _, _, _, _, rest) = seq.children();
                                current = Some(rest);
                            }
                        }
                    }
                }
            }
            self.path.pop_back();   
            let mut result = visited.visit_each(self);
=======
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
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
            result
        }
    }

<<<<<<< HEAD
=======
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

>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
        pub var_defs: VarSymbolTable,
        /// The set of currently defined functions. (fn_name, scope) -> Vec<param_type>
        pub func_defs: alloc::collections::BTreeMap<(nonterminal_fn_name, ScopeTrace), Vec<nonterminal_type>>,
=======
        pub defined_vars: VarSymbolTable,
        /// The set of currently defined functions. (fn_name, scope) -> Vec<param_type>
        pub func_param_counts: alloc::collections::BTreeMap<(nonterminal_fn_name, ScopeTrace), Vec<nonterminal_type>>,
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
                // Need to also record the return type.
                // <fn_def> ::= <type> <sep> <fn_kwd> <sep> <fn_name> "(" <param_list_e> ")" <sep> "{" <sep> <fn_body_e> <sep> "}" ;
                let return_type = tree.nth::<0>().nth::<0>().clone();
                // Have it be the last element in the param_type_list.
                let mut param_type_list = param_type_list;
                param_type_list.push(return_type);
                // Save the function definition using the node as key.
                self.func_defs.insert((fn_name, self.scope_trace.clone()), param_type_list);
=======
                // Save the function definition using the node as key.
                self.func_param_counts.insert((fn_name, self.scope_trace.clone()), param_type_list);
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
                self.var_defs.insert((var_decl_name, self.scope_trace.clone()), var_decl_type);
=======
                self.defined_vars.insert((var_decl_name, self.scope_trace.clone()), var_decl_type);
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
            } /* Check now for param_name */
            else if let Some(param_tree) = visited.downcast::<nonterminal_param>() {
                let type_inside = param_tree.nth::<0>().nth::<0>().clone();
                let var_name_inside = param_tree.nth::<0>().nth::<2>().nth::<0>().clone();
                // The parameters should be scoped properly.
<<<<<<< HEAD
                self.var_defs.insert((var_name_inside, self.scope_trace.clone()), type_inside);
=======
                self.defined_vars.insert((var_name_inside, self.scope_trace.clone()), type_inside);
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
    #[derive(Debug)]
    pub struct ConstraintVisitorDefUse<'a> {
=======
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorDefUse {
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The current scope_id, to track variable scopes.
        pub scope_id: usize,
        /// The current scope depth, to track variable scopes.
        pub scope_trace: Vec<usize>,
<<<<<<< HEAD
        /// These next three should be initialized by a prior pass of DeclarationCollector.
        /// The set of currently defined variables, (var_name, scope) -> var_type
        pub defined_vars: &'a VarSymbolTable,
        /// The set of currently defined functions. (fn_name, scope) -> Vec<param_type>
        pub defined_fns: &'a FuncSymbolTable,
        /// The set of currently defined structs. struct_name -> Vec<(field_name, field_type)>
        pub defined_structs: &'a alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The list of places where violations _could_ have occurred, for computing the violation ratio.
        pub paths_to_passed_checks: Vec<VecDeque<usize>>,
    }

    impl<'a> ConstraintVisitorDefUse<'a> {
        /// Create a new ConstraintVisitorDefUse with the given defined variables/functions/structs.
        pub fn new(
            defined_vars: &'a VarSymbolTable,
            defined_fns: &'a FuncSymbolTable,
            defined_structs: &'a alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
        ) -> Self {
            Self {
                path: VecDeque::new(),
                scope_trace: Vec::new(),
                scope_id: 0,
                violations: Vec::new(),
                paths_to_passed_checks: Vec::new(),
                defined_vars,
                defined_fns,
                defined_structs,
            }
        }
    }

    impl<T> Visitor<T> for ConstraintVisitorDefUse<'_>
    where
=======
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
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
        T: VisitableChildren<T> +
        AsNodeRef<nonterminal_var_access> +
        AsNodeRef<nonterminal_assignment> +
        AsNodeRef<nonterminal_decl> +
        AsNodeRef<nonterminal_fn_def> +
<<<<<<< HEAD
        AsNodeRef<nonterminal_var_name> +
        AsNodeRef<nonterminal_fn_call> +
        AsNodeRef<nonterminal_type> +
        AsNodeRef<nonterminal_struct_type>,
=======
        AsNodeRef<nonterminal_var_name>,
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
                // First, check parameters for undefined struct types.
                // <fn_def> ::= <type> <sep> <fn_kwd> <sep> <fn_name> "(" <param_list_e> ")" <sep> "{" <sep> <fn_body_e> <sep> "}" ;
                // let param_list_e = _tree.nth::<0>().nth::<6>();
                // // Is param_list_e <e> or <param_list>?
                // if let Some(_e) = param_list_e.nth::<0>().nth::<1>() {
                //     // No parameters.
                // } else {
                //     // In this case, we know it's param_list.
                //     let mut current = param_list_e.nth::<0>().nth::<0>();
                //     while let Some(pl) = current {
                //         match pl.nth::<0>() {
                //             // Variant 0, single param.
                //             nonterminal_param_list_0::variant_0(param) => {
                //                 let param_type = param.nth::<0>().nth::<0>().clone();
                //                 // If the type is a struct type, check if it's defined.
                //                 if let nonterminal_type_0::variant_1(struct_type) = param_type.nth::<0>() {
                //                     let struct_name = struct_type.nth::<0>().nth::<2>().clone();
                //                     if !self.defined_structs.contains_key(&struct_name) {
                //                         // Struct type not defined, violation.
                //                         self.violations.push(self.path.clone());
                //                     } else {
                //                         // Struct type defined, passed check.
                //                         self.paths_to_passed_checks.push(self.path.clone());
                //                     }
                //                 }
                //                 current = None;
                //             },
                //             // Variant 1, param followed by more params.
                //             nonterminal_param_list_0::variant_1(seq) => {
                //                 let (param, _, _, rest) = seq.children();
                //                 let param_type = param.nth::<0>().nth::<0>().clone();
                //                 // If the type is a struct type, check if it's defined.
                //                 if let nonterminal_type_0::variant_1(struct_type) = param_type.nth::<0>() {
                //                     let struct_name = struct_type.nth::<0>().nth::<2>().clone();
                //                     if !self.defined_structs.contains_key(&struct_name) {
                //                         // Struct type not defined, violation.
                //                         self.violations.push(self.path.clone());
                //                     } else {
                //                         // Struct type defined, passed check.
                //                         self.paths_to_passed_checks.push(self.path.clone());
                //                     }
                //                 }
                //                 current = Some(rest);
                //             }
                //         }
                //     }
                // }
=======
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
                self.scope_id += 1;
                self.scope_trace.push(self.scope_id);
                // Visit the function, then decrease depth.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                visitor.scope_trace.pop();
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            } // Functions are currently the only scope-increasing construct.

<<<<<<< HEAD

            if let Some(tree) = visited.downcast::<nonterminal_type>() {
                // If the type is a struct type, check if it's defined.
                if let nonterminal_type_0::variant_1(struct_type) = tree.nth::<0>() {
                    let struct_name = struct_type.nth::<0>().nth::<2>().clone();
                    if !self.defined_structs.contains_key(&struct_name) {
                        // Struct type not defined, violation.
                        self.violations.push(self.path.clone());
                    } else {
                        // Struct type defined, passed check.
                        self.paths_to_passed_checks.push(self.path.clone());
                    }
                }
            } else if let Some(tree) = visited.downcast::<nonterminal_var_access>() {
                let var_name_accessed = tree.nth::<0>().clone();
                if get_var_definition(&self.defined_vars, &var_name_accessed, &self.scope_trace).is_none() {
                    self.violations.push(self.path.clone());
                } else {
                    // It is defined, so this is a passed check.
                    self.paths_to_passed_checks.push(self.path.clone());
                }
            } else if let Some(decl_tree) = visited.downcast::<nonterminal_decl>() {
                let var_decl_name = decl_tree.nth::<0>().nth::<2>().clone();
                let var_decl_type = decl_tree.nth::<0>().nth::<0>().clone();
                // Is the variable already defined in the current scope?
                if get_var_definition(&self.defined_vars, &var_decl_name, &self.scope_trace).is_none() {
                    // Not defined, that's ok.
                    self.paths_to_passed_checks.push(self.path.clone());
                } else {
                    // Already defined, violation.
                    self.violations.push(self.path.clone());
                }
                // Is the type a struct type? If so, check if it's defined.
                // if let nonterminal_type_0::variant_1(struct_type) = var_decl_type.nth::<0>() {
                //     let struct_name = struct_type.nth::<0>().nth::<2>().clone();
                //     if !self.defined_structs.contains_key(&struct_name) {
                //         // Struct type not defined, violation.
                //         self.violations.push(self.path.clone());
                //     } else {
                //         // Struct type defined, passed check.
                //         self.paths_to_passed_checks.push(self.path.clone());
                //     }
                // }
            } else if let Some(fn_call) = visited.downcast::<nonterminal_fn_call>() {
                // Get name and check if function is defined.
                let fn_name = fn_call.nth::<0>().nth::<0>().clone();
                if get_func_definition(&self.defined_fns, &fn_name, &self.scope_trace).is_none() {
                    self.violations.push(self.path.clone());
                } else {
                    // It is defined, so this is a passed check.
                    self.paths_to_passed_checks.push(self.path.clone());
                }
            }
=======
            if let Some(tree) = visited.downcast::<nonterminal_var_access>() {
                let var_name_accessed = tree.nth::<0>().clone();
                if get_var_definition(&self.defined_vars, &var_name_accessed, &self.scope_trace).is_none() {
                    self.violations.push(self.path.clone());
                }
            } 
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
        /// The list of places where violations _could_ have occurred, for computing the violation ratio.
        pub paths_to_passed_checks: Vec<VecDeque<usize>>,
=======
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
                } else {
                    // Valid return statement, but still counts as a place where a violation could have occurred.
                    self.paths_to_passed_checks.push(self.path.clone());
=======
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
    #[derive(Debug)]
    pub struct ConstraintVisitorStructAccess<'a> {
=======
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorStructAccess {
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
<<<<<<< HEAD
        /// The list of places where violations did not occur, for computing the violation ratio.
        pub paths_to_passed_checks: Vec<VecDeque<usize>>,
=======
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
        /// The current scope level.
        pub scope_depth: usize,
        /// The current struct definitions, mapping struct names to their field names and types.
        /// This should be initialized by a prior pass of DeclarationCollector.
<<<<<<< HEAD
        pub struct_defs: &'a alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
        /// The current variable definitions, mapping variable names to their types.
        pub var_defs: &'a VarSymbolTable,
    }

    impl<'a> ConstraintVisitorStructAccess<'a> {
        /// Create a new ConstraintVisitorStructAccess with the given struct and variable definitions.
        pub fn new(
            struct_defs: &'a alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
            var_defs: &'a VarSymbolTable,
        ) -> Self {
            Self {
                path: VecDeque::new(),
                scope_depth: 0,
                violations: Vec::new(),
                paths_to_passed_checks: Vec::new(),
                var_defs,
                struct_defs,
            }
        }
    }

    impl<T> Visitor<T> for ConstraintVisitorStructAccess<'_>
=======
        pub struct_defs: alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
        /// The current variable definitions, mapping variable names to their types.
        pub var_defs: alloc::collections::BTreeMap<(nonterminal_var_name, usize), nonterminal_type>,
    }

    impl<T> Visitor<T> for ConstraintVisitorStructAccess
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
                let pot_var_type = get_var_definition(&self.var_defs, &var_name, &vec![self.scope_depth]);
=======
                // Note: (Lowkey a TODO) This should look up the scope, not necessarily that it's the exact scope depth.
                let pot_var_type = self.var_defs.get(&(var_name, self.scope_depth)).cloned();

>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
                                } else {
                                    // Field found, no violation.
                                    self.paths_to_passed_checks.push(self.path.clone());
                                }
                            } else {
                                // Struct not found, violation. But this should be caught by def-before-use.
                                // self.violations.push(self.path.clone());
                            }
                        } else {
                            // Variable is not of struct type, violation. But this should be caught by the type checker.
                            // self.violations.push(self.path.clone());
                        }
                    },
                    None => {
                        // Variable not found, violation. But this should be caught by def-before-use.
                        // self.violations.push(self.path.clone());
=======
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
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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

<<<<<<< HEAD
    // ================ Type checking.
    // Helpers

    // TODO [Type Checking] Could implement a path cache so we can save any type inference work we have already done.

    // Ok, we need to make a light layer above nonterminal_type, specifically to handle struct expressions since they
    // can be anonymous and thus have no name to look up.
    /// Wrapper around nonterminal_type to handle struct types with field names and types. Idea is to have most of the 
    /// types represented as they are in the language, but esp. structural types have no equivalent in the grammar.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NonterminalTypeExtended {
        /// The base nonterminal_type.
        /// If struct_fields is supplied, this should be a struct type.
        /// If struct_fields_positional is supplied, this should be an empty struct type.
        pub base: nonterminal_type,
        /// If this is a struct type, we need to store the field names and types.
        /// This will be None if the type is not a struct.
        pub struct_fields: Option<alloc::collections::BTreeMap<nonterminal_field_name, nonterminal_type>>,
        /// Alternatively, positional fields could be used.
        pub struct_fields_positional: Option<Vec<NonterminalTypeExtended>>,
    }

    // Allows for easy conversion into and out of nonterminal_type.
    impl Into<nonterminal_type> for NonterminalTypeExtended {
        fn into(self) -> nonterminal_type {
            self.base
        }
    }

    // See above.
    impl Into<NonterminalTypeExtended> for nonterminal_type {
        fn into(self) -> NonterminalTypeExtended {
            NonterminalTypeExtended {
                base: self,
                struct_fields: None,
                struct_fields_positional: None,
            }
        }
    }

    fn coercion_and_subtyping_possible(t1: &nonterminal_type, t2: &nonterminal_type) -> bool {
        // For simplicity, we will consider coercion and subtyping possible only for numeric types.
        // A more complete implementation would handle more complex rules.
        let numeric_types = vec![
            nonterminal_basic_type::new(nonterminal_basic_type_0::from_0th(nonterminal_basic_type_0_0)), // int
            nonterminal_basic_type::new(nonterminal_basic_type_0::from_1th(nonterminal_basic_type_0_1)), // float
            nonterminal_basic_type::new(nonterminal_basic_type_0::from_2th(nonterminal_basic_type_0_2)), // double
        ];
        if let (nonterminal_type_0::variant_0(bt1), nonterminal_type_0::variant_0(bt2)) = (t1.nth::<0>(), t2.nth::<0>()) {
            return numeric_types.contains(&bt1) && numeric_types.contains(&bt2);
        }
        false
    }

    fn types_compatible(t1: &NonterminalTypeExtended, t2: &NonterminalTypeExtended) -> bool {
        // For simplicity, we will consider types compatible if they are exactly the same.
        // A more complete implementation would handle type coercion, subtyping, etc.
        // Ok there are a few cases; let's implement them.
        if t1.base == t2.base || coercion_and_subtyping_possible(&t1.base, &t2.base) {
            return true;
        } 

        // Check if both are struct types.
        if let (Some(fields1), Some(fields2)) = (&t1.struct_fields, &t2.struct_fields) {
            // If both types are structs, check if their fields are compatible.
            return fields1.iter().all(|(name, typ1)| {
                fields2.get(name).map_or(false, |typ2| types_compatible(&typ1.clone().into(), &typ2.clone().into()))
            });
        }

        // Check if both are struct types with positional fields.
        if let (Some(fields1), Some(fields2)) = (&t1.struct_fields_positional, &t2.struct_fields_positional) {
            // If both types are structs with positional fields, check if their fields are compatible.
            return fields1.iter().zip(fields2).all(|(typ1, typ2)| types_compatible(typ1, typ2));
        }

        // Check if positional struct fields match named struct fields.
        if let (Some(pos_fields), Some(named_fields)) = (&t1.struct_fields_positional, &t2.struct_fields) {
            if pos_fields.len() == named_fields.len() {
                return pos_fields.iter().zip(named_fields.values()).all(|(typ1, typ2)| types_compatible(typ1, &typ2.clone().into()));
            }
        }   

        // Also other way around.
        if let (Some(pos_fields), Some(named_fields)) = (&t2.struct_fields_positional, &t1.struct_fields) {
            if pos_fields.len() == named_fields.len() {
                return pos_fields.iter().zip(named_fields.values()).all(|(typ1, typ2)| types_compatible(typ1, &typ2.clone().into()));
            }
        }   

        false
    }

    fn type_resulting_from_binop(t1: &NonterminalTypeExtended, t2: &NonterminalTypeExtended, binop: &nonterminal_binop_op) -> Option<NonterminalTypeExtended> {
        // For simplicity, we will consider the resulting type to be the same as the operand types if they are compatible.
        // A more complete implementation would handle type coercion, subtyping, etc.
        if types_compatible(t1, t2) {
            Some(t1.clone())
        } else {
            None
        }
    }

    fn infer_expr_unit_type(
        expr: &nonterminal_expr_unit,
        var_defs: &alloc::collections::BTreeMap<(nonterminal_var_name, Vec<usize>), nonterminal_type>,
        func_defs: &alloc::collections::BTreeMap<(nonterminal_fn_name, Vec<usize>), Vec<nonterminal_type>>,
        struct_defs: &alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
        scope_trace: &Vec<usize>,
    ) -> Option<NonterminalTypeExtended> {
        // Working with expr_unit here.
        // <expr_unit> ::= <var_access> | <value> | <fn_call> | <struct_expr> | <struct_access>
        match expr.nth::<0>() {
            nonterminal_expr_unit_0::variant_0(var_access) => {
                // <var_access> ::= <var_name> ;
                let var_name = var_access.nth::<0>().clone();
                let var_def = get_var_definition(var_defs, &var_name, scope_trace)
                    .cloned();
                match var_def {
                    Some(var_type) => Some(var_type.into()),
                    None => None, // Variable not found.
                }
            },
            nonterminal_expr_unit_0::variant_1(value) => {
                // <value> ::= <bool_val> | <num_val> | <string_val> ;
                // <basic_type> ::= "int" | "float" | "double" | "bool" | "char" | "void" ;
                match value.nth::<0>() {
                    nonterminal_value_0::variant_0(_) => {
                        // bool_val
                        Some(
                            nonterminal_type::new(
                                nonterminal_type_0::from_0th(
                                    nonterminal_basic_type::new(
                                        nonterminal_basic_type_0::from_3th(
                                            nonterminal_basic_type_0_3)))).into())
                    },
                    nonterminal_value_0::variant_2(_) => {
                        // string_val
                        Some(
                            nonterminal_type::new(
                                nonterminal_type_0::from_0th(
                                    nonterminal_basic_type::new(
                                        nonterminal_basic_type_0::from_4th(
                                            nonterminal_basic_type_0_4)))).into())
                    },
                    nonterminal_value_0::variant_1(num_val) => {
                        match num_val.nth::<0>() {
                            nonterminal_num_val_0::variant_0(_) => {
                                // int
                                Some(
                                    nonterminal_type::new(
                                        nonterminal_type_0::from_0th(
                                            nonterminal_basic_type::new(
                                                nonterminal_basic_type_0::from_0th(
                                                    nonterminal_basic_type_0_0)))).into())
                            },
                            nonterminal_num_val_0::variant_1(_) => {
                                // float
                                Some(
                                    nonterminal_type::new(
                                        nonterminal_type_0::from_0th(
                                            nonterminal_basic_type::new(
                                                nonterminal_basic_type_0::from_1th(
                                                    nonterminal_basic_type_0_1)))).into())
                            },
                            nonterminal_num_val_0::variant_2(_) => {
                                // double
                                Some(
                                    nonterminal_type::new(
                                        nonterminal_type_0::from_0th(
                                            nonterminal_basic_type::new(
                                                nonterminal_basic_type_0::from_2th(
                                                    nonterminal_basic_type_0_2)))).into())
                            },
                        }
                    },
                }
            },
            nonterminal_expr_unit_0::variant_2(fn_call) => {
                // <fn_call> ::= <fn_name> "(" <arg_list_e> ")" ;
                let fn_name = fn_call.nth::<0>().nth::<0>().clone();
                if let Some(param_types) = get_func_definition(func_defs, &fn_name, scope_trace) {
                    // The return type is the last type in the param_types list.
                    if let Some(return_type) = param_types.last() {
                        Some(return_type.clone().into())
                    } else {
                        None // Function has no return type defined.
                    }
                } else {
                    None // Function not found.
                }
            },
            nonterminal_expr_unit_0::variant_3(struct_expr) => {
                // Specifically:
                // <struct_expr> ::= "{" "\n" <expr_list_e> "\n" "}" ;
                // <expr_list_e> ::= <expr_list> | <e> ;
                // <expr_list> ::= <expr> | <expr> "," <sep> <expr_list> ;
                let expr_list_e = struct_expr.nth::<0>().nth::<2>();
                // What are the types of the expressions in expr_list?
                let mut expr_types = Vec::new();
                // Go through expr_list and infer types.
                let mut current = None;
                // First, is it empty or not?
                match expr_list_e.nth::<0>().nth::<1>() {
                    Some(_e) => {
                        // Empty list.
                        return Some(NonterminalTypeExtended {
                            // In the interest of time, just put <type>.<basic_type>."void" here
                            base: nonterminal_type::new(
                                nonterminal_type_0::from_0th(
                                    nonterminal_basic_type::new(
                                        nonterminal_basic_type_0::from_5th(
                                            nonterminal_basic_type_0_5)))),
                            struct_fields: None,
                            struct_fields_positional: Some(vec![]),
                        });
                    },
                    None => {
                        // Non-empty list.
                        current = expr_list_e.nth::<0>().nth::<0>();
                    }
                }
                while let Some(el) = current {
                    match el.nth::<0>() {
                        // Variant 0, single expr.
                        nonterminal_expr_list_0::variant_0(expr) => {
                            if let Some(t) = infer_expr_type(&expr, var_defs, func_defs, struct_defs, scope_trace) {
                                expr_types.push(t);
                            } else {
                                return None; // Could not infer type of expression.
                            }
                            current = None;
                        },
                        // Variant 1, expr followed by more exprs.
                        nonterminal_expr_list_0::variant_1(seq) => {
                            let (expr, _, _, rest) = seq.children();
                            if let Some(t) = infer_expr_type(&expr, var_defs, func_defs, struct_defs, scope_trace) {
                                expr_types.push(t);
                            } else {
                                return None; // Could not infer type of expression.
                            }
                            current = Some(rest);
                        }
                    }
                }
                // Now, we have the types of the expressions in expr_types.
                // We will create a new NonterminalTypeExtended representing an anonymous struct.
                Some(NonterminalTypeExtended {
                    // In the interest of time, just put <type>.<basic_type>."void" here
                    base: nonterminal_type::new(
                        nonterminal_type_0::from_0th(
                            nonterminal_basic_type::new(
                                nonterminal_basic_type_0::from_5th(
                                    nonterminal_basic_type_0_5)))),
                    struct_fields: None,
                    struct_fields_positional: Some(expr_types),
                })
            },
            nonterminal_expr_unit_0::variant_4(struct_access) => {
                // <struct_access> ::= <var_access> "." <field_name> ;
                let var_name = struct_access.nth::<0>().nth::<0>();
                let field_name = struct_access.nth::<0>().nth::<2>().clone();
                // Does the var being accessed exist and is it a struct?
                // Get the var name.
                let var_def = get_var_definition(var_defs, var_name, scope_trace)
                    .cloned();
                match var_def {
                    Some(var_type) => {
                        // We have a variable type, check if it's a struct type.
                        // Get the alternative.
                        let var_type_0 = var_type.nth::<0>();
                        if let nonterminal_type_0::variant_1(struct_type) = var_type_0 {
                            let struct_name = struct_type.nth::<0>().nth::<2>().clone();
                            // Look up the struct definition to see if the field exists.
                            if let Some(fields) = struct_defs.get(&struct_name) {
                                if let Some((_, field_type)) = fields.iter().find(|(fname, _ftype)| fname == &field_name) {
                                    Some(field_type.clone().into())
                                } else {
                                    // Note: The way this is currently, this will double count some violations.
                                    None // Field not found in struct definition.
                                }
                            } else {
                                None // Struct not found.
                            }
                        } else {
                            None // Variable is not of struct type.
                        }
                    },                    
                    None => None // Variable not found.
                }
            },
        }
    }

    // A helper function to infer the type of an expression.
    fn infer_expr_type(
        expr: &nonterminal_expr,
        var_defs: &alloc::collections::BTreeMap<(nonterminal_var_name, Vec<usize>), nonterminal_type>,
        func_defs: &alloc::collections::BTreeMap<(nonterminal_fn_name, Vec<usize>), Vec<nonterminal_type>>,
        struct_defs: &alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
        scope_trace: &Vec<usize>,
    ) -> Option<NonterminalTypeExtended> {
        // This is being done at the <expr> level, so there are two variants: arith_expr and expr_unit.
        match expr.nth::<0>() {
            nonterminal_expr_0::variant_0(arith_expr) => {
                // <arith_expr> ::= <binop> | <unop> ;
                // Which case is it?
                match arith_expr.nth::<0>() {
                    nonterminal_arith_expr_0::variant_0(binop) => {
                        // <binop> ::= <expr_unit> <sep> <binop_op> <sep> <expr> ;
                        let left_expr_unit = binop.nth::<0>().nth::<0>();
                        let right_expr = binop.nth::<0>().nth::<4>();
                        let left_type = infer_expr_unit_type(left_expr_unit, var_defs, func_defs, struct_defs, scope_trace);
                        let right_type = infer_expr_type(&right_expr, var_defs, func_defs, struct_defs, scope_trace);
                        let binop_op = binop.nth::<0>().nth::<2>();
                        // Are types compatible for the binary operator?
                        if let (Some(lt), Some(rt)) = (left_type, right_type) {
                            if let Some(bop_type) = type_resulting_from_binop(&lt, &rt, &binop_op) {
                                Some(bop_type)
                            } else {
                                None // Types are not compatible.
                            }
                        } else {
                            None // Could not infer types.
                        }
                    },
                    nonterminal_arith_expr_0::variant_1(unop) => {
                        // <unop> ::= <unop_op> <expr> ;
                        let right_expr = unop.nth::<0>().nth::<1>();
                        let expr_type_nte = infer_expr_type(&right_expr, var_defs, func_defs, struct_defs, scope_trace);
                        // Check if the type is valid for the unary operator.
                        let unop_op = unop.nth::<0>().nth::<0>();
                        // <unop_op> ::= "-" | <sep> "not" <sep> ;
                        // If expr_type (a NonterminalTypeExtended) has struct fields, it cannot be used with "-" or "not".
                        if let Some(t) = &expr_type_nte {
                            if t.struct_fields.is_some() || t.struct_fields_positional.is_some() {
                                return None; // Invalid type for unary operator.
                            }
                            let expr_type = &t.base;
                            match unop_op.nth::<0>() {
                                nonterminal_unop_op_0::variant_0(_) => {
                                    // "-" operator, valid for numeric types.
                                    match expr_type.nth::<0>() {
                                        nonterminal_type_0::variant_0(basic_type) => {
                                            match basic_type.nth::<0>() {
                                                nonterminal_basic_type_0::variant_0(_) | // int
                                                nonterminal_basic_type_0::variant_1(_) | // float
                                                nonterminal_basic_type_0::variant_2(_)   // double
                                                => Some(t.clone()),
                                                _ => None, // Invalid type for "-"
                                            }
                                        },
                                        _ => None, // Invalid type for "-"
                                    }
                                },
                                nonterminal_unop_op_0::variant_1(_) => {
                                    // "not" operator, valid for bool type.
                                    match expr_type.nth::<0>() {
                                        nonterminal_type_0::variant_0(basic_type) => {
                                            match basic_type.nth::<0>() {
                                                nonterminal_basic_type_0::variant_3(_) // bool
                                                => Some(t.clone()),
                                                _ => None, // Invalid type for "not"
                                            }
                                        },
                                        _ => None, // Invalid type for "not"
                                    }
                                },
                                _ => None, // Should not happen
                            }
                        } else {
                            return None; // Could not infer type
                        }
                    },
                }
            },
            nonterminal_expr_0::variant_1(expr_unit) => {
                infer_expr_unit_type(expr_unit, var_defs, func_defs, struct_defs, scope_trace)
            }
        }
    }

    // Constraint visitor: No void declarations or parameters.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorNoVoidDecls {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The list of places where violations did not occur, for computing the violation ratio.
        pub paths_to_passed_checks: Vec<VecDeque<usize>>,
    }

    impl<T> Visitor<T> for ConstraintVisitorNoVoidDecls
    where
        T: VisitableChildren<T> +
            AsNodeRef<nonterminal_decl> +
            AsNodeRef<nonterminal_param> +
            AsNodeRef<nonterminal_type> +
            AsNodeRef<nonterminal_basic_type> +
            AsNodeRef<nonterminal_field_def_list>,
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
            if let Some(_tree) = visited.downcast::<nonterminal_decl>() {
                // <decl> ::= <type> <sep> <var_name> | <type> <sep> <var_name> "=" <sep> <expr> ;
                // TODO: Update this to handle the alternation.
                let var_type = _tree.nth::<0>().nth::<0>().clone();
                if let nonterminal_type_0::variant_0(basic_type) = var_type.nth::<0>() {
                    if let nonterminal_basic_type_0::variant_5(_) = basic_type.nth::<0>() {
                        // Void type in declaration, violation.
                        self.violations.push(self.path.clone());
                    } else {
                        // Valid declaration type.
                        self.paths_to_passed_checks.push(self.path.clone());
                    }
                }
            } else if let Some(_tree) = visited.downcast::<nonterminal_param>() {
                // <param> ::= <type> <sep> <var_name> ;
                let param_type = _tree.nth::<0>().nth::<0>().clone();
                if let nonterminal_type_0::variant_0(basic_type) = param_type.nth::<0>() {
                    if let nonterminal_basic_type_0::variant_5(_) = basic_type.nth::<0>() {
                        // Void type in parameter, violation.
                        self.violations.push(self.path.clone());
                    } else {
                        // Valid parameter type.
                        self.paths_to_passed_checks.push(self.path.clone());
                    }
                }
            } else if let Some(_tree) = visited.downcast::<nonterminal_field_def_list>() {
                // <field_def_list> ::= <field_def> | <field_def> "," <sep> <field_def_list> ;
                // Check each field_def for void type.
                let mut current = Some(_tree);
                while let Some(field_def_list) = current {
                    match field_def_list.nth::<0>() {
                        nonterminal_field_def_list_0::variant_0(field_def) => {
                            // Single field_def.
                            let field_type = field_def.nth::<0>().nth::<0>().clone();
                            if let nonterminal_type_0::variant_0(basic_type) = field_type {
                                if let nonterminal_basic_type_0::variant_5(_) = basic_type.nth::<0>() {
                                    // Void type in field definition, violation.
                                    self.violations.push(self.path.clone());
                                } else {
                                    // Valid field type.
                                    self.paths_to_passed_checks.push(self.path.clone());
                                }
                            }
                            current = None;
                        },
                        nonterminal_field_def_list_0::variant_1(seq) => {
                            // field_def followed by more field_defs.
                            // <type> <sep> <field_name> "," "\n" <field_def_list>
                            let (field_type, _, _, _, _, rest) = seq.children();
                            if let nonterminal_type_0::variant_0(basic_type) = field_type.nth::<0>() {
                                if let nonterminal_basic_type_0::variant_5(_) = basic_type.nth::<0>() {
                                    // Void type in field definition, violation.
                                    self.violations.push(self.path.clone());
                                } else {
                                    // Valid field type.
                                    self.paths_to_passed_checks.push(self.path.clone());
                                }
                            }
                            current = Some(rest);
                        }
                    }
                }
            }
            let mut result = visited.visit_each(self);
            if let Ok(ControlFlow::Continue(visitor)) = & mut result {
                visitor.path.pop_back();
            }
            result
        }
    }
    // ================= end of No void declarations or parameters.

    // Constraint: Struct expr only on RHS of decl.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorStructExprRHSOfDecl {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The list of places where violations did not occur, for computing the violation ratio.
        pub paths_to_passed_checks: Vec<VecDeque<usize>>,
        /// Are we currently inside a decl?
        pub inside_decl: bool,
    }

    impl<T> Visitor<T> for ConstraintVisitorStructExprRHSOfDecl
    where
        T: VisitableChildren<T> +
            AsNodeRef<nonterminal_struct_expr> +
            AsNodeRef<nonterminal_decl> +
            AsNodeRef<nonterminal_expr>,
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
            // Check if we are entering or leaving a decl.
            if let Some(_tree) = visited.downcast::<nonterminal_decl>() {
                self.inside_decl = true;
                // Visit the decl, then set inside_decl back to false.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                visitor.inside_decl = false;
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            }
            
            // Now, actual logic.
            if let Some(_tree) = visited.downcast::<nonterminal_struct_expr>() {
                // We are in a struct_expr.
                if !self.inside_decl {
                    // Violation, struct expr not on RHS of decl.
                    self.violations.push(self.path.clone());
                } else {
                    // Valid usage.
                    self.paths_to_passed_checks.push(self.path.clone());
                }
            }

            let mut result = visited.visit_each(self);
            if let Ok(ControlFlow::Continue(visitor)) = & mut result {
                visitor.path.pop_back();
            }
            result
        }
    }

    // Constraint: No empty structs.
    #[derive(Debug, Default)]
    pub struct ConstraintVisitorNoEmptyStructs {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The list of places where violations did not occur, for computing the violation ratio.
        pub paths_to_passed_checks: Vec<VecDeque<usize>>,
    }

    impl<T> Visitor<T> for ConstraintVisitorNoEmptyStructs
    where
        T: VisitableChildren<T> +
            AsNodeRef<nonterminal_struct_def> +
            AsNodeRef<nonterminal_field_def_list_e> +
            AsNodeRef<nonterminal_struct_expr> +
            AsNodeRef<nonterminal_expr_list_e>,
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
            if let Some(_tree) = visited.downcast::<nonterminal_struct_def>() {
                // <struct_def> ::= "struct" <sep> <struct_name> <sep> "{" <sep> <field_list_e> <sep> "}" ";" ;
                let field_list_e = _tree.nth::<0>().nth::<6>();
                match field_list_e.nth::<0>() {
                    nonterminal_field_def_list_e_0::variant_1(_) => {
                        // Empty field list, violation.
                        self.violations.push(self.path.clone());
                    },
                    nonterminal_field_def_list_e_0::variant_0(_) => {
                        // Non-empty field list, valid.
                        self.paths_to_passed_checks.push(self.path.clone());
                    }
                }
            } /* also catch empty struct exprs */
            else if let Some(_tree) = visited.downcast::<nonterminal_struct_expr>() {
                // <struct_expr> ::= "{" "\n" <expr_list_e> "\n" "}" ;
                // <expr_list_e> ::= <expr_list> | <e> ;
                // <expr_list> ::= <expr> | <expr> "," "\n" <expr_list> ;
                let expr_list_e = _tree.nth::<0>().nth::<2>();
                match expr_list_e.nth::<0>() {
                    nonterminal_expr_list_e_0::variant_1(_) => {
                        // Empty expr list, violation.
                        self.violations.push(self.path.clone());
                    },
                    nonterminal_expr_list_e_0::variant_0(_) => {
                        // Non-empty expr list, valid.
                        self.paths_to_passed_checks.push(self.path.clone());
                    }
                }
            }
            let mut result = visited.visit_each(self);
            if let Ok(ControlFlow::Continue(visitor)) = & mut result {
                visitor.path.pop_back();
            }
            result
        }
    }

    // Constraint visitor.
    /// Basic type-checking constraint visitor.
    #[derive(Debug)]
    pub struct ConstraintVisitorTypeCheck<'a> {
        /// The current path, to be used by the visitor when saving violations.
        pub path: VecDeque<usize>,
        /// The current scope path.
        pub scope_trace: Vec<usize>,
        /// The current scope id.
        pub scope_id: usize,
        /// The list of violations found so far.
        pub violations: Vec<VecDeque<usize>>,
        /// The list of locations where violations could have occurred.
        pub paths_to_passed_checks: Vec<VecDeque<usize>>,
        /// The current variable definitions, mapping variable names and scopes to their types.
        /// This should be initialized by a prior pass of DeclarationCollector.
        pub var_defs: &'a VarSymbolTable,
        /// The current function definitions, mapping function names and scopes to their types.
        /// This should be initialized by a prior pass of DeclarationCollector.
        pub func_defs: &'a FuncSymbolTable,
        /// The current struct definitions, mapping struct names to their field names and types.
        /// This should be initialized by a prior pass of DeclarationCollector.
        pub struct_defs: &'a alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
    }

    impl<'a> ConstraintVisitorTypeCheck<'a> {
        /// Creates a new ConstraintVisitorTypeCheck.
        pub fn new(
            var_defs: &'a VarSymbolTable,
            func_defs: &'a FuncSymbolTable,
            struct_defs: &'a alloc::collections::BTreeMap<nonterminal_struct_name, Vec<(nonterminal_field_name, nonterminal_type)>>,
        ) -> Self {
            Self {
                path: VecDeque::new(),
                scope_trace: Vec::new(),
                scope_id: 0,
                violations: Vec::new(),
                paths_to_passed_checks: Vec::new(),
                var_defs,
                func_defs,
                struct_defs,
            }
        }
    }

    impl<T> Visitor<T> for ConstraintVisitorTypeCheck<'_>
    where
        T: VisitableChildren<T> +
            AsNodeRef<nonterminal_assignment> +
            AsNodeRef<nonterminal_expr> +
            AsNodeRef<nonterminal_var_access> +
            AsNodeRef<nonterminal_var_name> +
            AsNodeRef<nonterminal_decl> +
            AsNodeRef<nonterminal_type> +
            AsNodeRef<nonterminal_fn_def> +
            AsNodeRef<nonterminal_fn_name> +
            AsNodeRef<nonterminal_param_list> +
            AsNodeRef<nonterminal_param_list_e> +
            AsNodeRef<nonterminal_fn_call> +
            AsNodeRef<nonterminal_return_stmt> +
            AsNodeRef<nonterminal_struct_access> +
            AsNodeRef<nonterminal_field_name> +
            AsNodeRef<nonterminal_expr_stmt> +
            AsNodeRef<nonterminal_struct_expr> +
            AsNodeRef<nonterminal_struct_name>,
=======
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
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
                // Before doing anything else, let's grab the return type.
                // If it's not void, we need to make sure there's at least one return statement in the function body.
                // <fn_def> ::= <type> <sep> <fn_kwd> <sep> <fn_name> "(" <param_list_e> ")" <sep> "{" <sep> <fn_body_e> <sep> "}" ;
                let fn_return_type = _tree.nth::<0>().nth::<0>().clone();
                let fn_return_type_is_void = if let nonterminal_type_0::variant_0(basic_type) = fn_return_type.nth::<0>() {
                    if let nonterminal_basic_type_0::variant_5(_) = basic_type.nth::<0>() {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                let fn_body_e = _tree.nth::<0>().nth::<11>();
                // Is there a body?
                // <fn_body_e> ::= <statements> | <e> ;
                match fn_body_e.nth::<0>() {
                    nonterminal_fn_body_e_0::variant_0(statements) => {
                        // We have statements, grab references to all return statements in the body.
                        // We will do this by traversing the statements.
                        let mut return_statements = Vec::new();
                        let mut current = Some(statements);
                        while let Some(stmts) = current {
                            match stmts.nth::<0>() {
                                // Variant 0, single statement.
                                nonterminal_statements_0::variant_0(stmt) => {
                                    // <stmt> ::= <decl> ";" | <assignment> ";" | <fn_def> | <struct_def> | <expr_stmt> ";" | <return_stmt> ";" ;
                                    match stmt.nth::<0>() {
                                        nonterminal_stmt_0::variant_5(return_stmt) => {
                                            // We have a return statement.
                                            return_statements.push(return_stmt);
                                        },
                                        _ => { /* Not a return statement, continue. */ }
                                    }
                                    current = None;
                                },
                                // Variant 1, statement followed by more statements.
                                nonterminal_statements_0::variant_1(seq) => {
                                    let (stmt, _, rest) = seq.children();
                                    match stmt.nth::<0>() {
                                        nonterminal_stmt_0::variant_5(return_stmt) => {
                                            // We have a return statement.
                                            return_statements.push(return_stmt);
                                        },
                                        _ => { /* Not a return statement, continue. */ }
                                    }
                                    current = Some(rest);
                                }
                            }
                        }
                        // Now we know if there's a return statement or not.
                        if !fn_return_type_is_void && return_statements.is_empty() {
                            // Function return type is not void, but no return statement found.
                            self.violations.push(self.path.clone());
                        } else if fn_return_type_is_void && return_statements.is_empty() {
                            // Function is void and has no return statements, all good.
                            self.paths_to_passed_checks.push(self.path.clone());
                        } else if !fn_return_type_is_void && !return_statements.is_empty() {
                            // Function is not void and has return statements.
                            // Go through all return statements and check their types.
                            for return_stmt in return_statements {
                                // <return_stmt> ::= <return_kwd> | <return_kwd> <sep> <expr> ;
                                let return_stmt_0 = return_stmt.nth::<0>();
                                match return_stmt_0.nth::<0>() {
                                    nonterminal_return_stmt_0::variant_0(_) => {
                                        // Void return in non-void function, violation.
                                        self.violations.push(self.path.clone());
                                    },
                                    nonterminal_return_stmt_0::variant_1(seq) => {
                                        let expr = seq.nth::<2>();
                                        let expr_type = infer_expr_type(expr, &self.var_defs, &self.func_defs, &self.struct_defs, &self.scope_trace);
                                        if let Some(et) = expr_type {
                                            if !types_compatible(&fn_return_type.clone().into(), &et) {
                                                self.violations.push(self.path.clone());
                                            } else {
                                                // Types match, all good.
                                                self.paths_to_passed_checks.push(self.path.clone()); 
                                            }
                                        } else {
                                            // Could not infer expression type, consider it a violation.
                                            self.violations.push(self.path.clone());
                                        }
                                    }
                                }
                            }
                        } else {
                            // Function is void and has return statements.
                            // Are they all empty returns?
                            let mut all_void_returns = true;
                            for return_stmt in return_statements {
                                let return_stmt_0 = return_stmt.nth::<0>();
                                match return_stmt_0.nth::<0>() {
                                    nonterminal_return_stmt_0::variant_0(_) => { /* Void return, all good. */ },
                                    nonterminal_return_stmt_0::variant_1(_) => {
                                        // Non-void return in void function, violation.
                                        all_void_returns = false;
                                        self.violations.push(self.path.clone());
                                    }
                                }
                            }
                            if all_void_returns {
                                // All returns are void, all good.
                                self.paths_to_passed_checks.push(self.path.clone());
                            }
                        }
                    },
                    nonterminal_fn_body_e_0::variant_1(_) => {
                        // No body, so no return statements.
                        if !fn_return_type_is_void {
                            // Function return type is not void, but no return statement found.
                            self.violations.push(self.path.clone());
                        } else {
                            // Function is void and has no body, all good.
                            self.paths_to_passed_checks.push(self.path.clone());
                        }
                    },
                }
                self.scope_trace.push(self.scope_id);
                self.scope_id += 1;
                // Visit the function.
                let result = visited.visit_each(self);
                let Ok(ControlFlow::Continue(mut visitor)) = result;
                // Pop off the scope trace.
                visitor.scope_trace.pop();
                visitor.path.pop_back();
                return Ok(ControlFlow::Continue(visitor));
            }

            if let Some(tree) = visited.downcast::<nonterminal_decl>() {
                // Get the type.
                let var_type = tree.nth::<0>().nth::<0>().clone();
                // Does the RHS match the type?
                let rhs_e = tree.nth::<0>().nth::<4>();
                if let Some(decl_rhs) = rhs_e.nth::<0>().nth::<0>() {
                    // We have an RHS expression, infer its type.
                    // <decl_rhs> ::= "=" <sep> <expr> ;
                    let rhs_expr = decl_rhs.nth::<0>().nth::<2>();
                    let expr_type = infer_expr_type(rhs_expr, &self.var_defs, &self.func_defs, &self.struct_defs, &self.scope_trace);
                    if let Some(et) = expr_type {
                        if !types_compatible(&var_type.into(), &et) {
                            self.violations.push(self.path.clone());
                        } else {
                            // Types match, all good.
                            // Add this as a valid use.
                            self.paths_to_passed_checks.push(self.path.clone());
                        }
                    } else {
                        // Could not infer expression type, consider it a violation.
                        self.violations.push(self.path.clone());
                    }
                } else { /* Nothing to check if no RHS. */ }
                // Declaration processed; continue visiting.
                // No need to add to var_defs here, should be done by DeclarationCollector.
            } else if let Some(tree) = visited.downcast::<nonterminal_assignment>() {
                // <assignment> ::= <var_access> <sep> <assign_op> <sep> <expr> ;
                let var_access = tree.nth::<0>().nth::<0>();
                let expr = tree.nth::<0>().nth::<4>();
                // Get the variable name from the var_access.
                let var_name = var_access.nth::<0>().clone();
                // Look up the variable type.
                let var_type = get_var_definition(&self.var_defs, &var_name, &self.scope_trace).cloned();
                // Get the expression type.
                let expr_type = infer_expr_type(expr, &self.var_defs, &self.func_defs, &self.struct_defs, &self.scope_trace);
                // Compare types.
                if let (Some(vt), Some(et)) = (var_type, expr_type) {
                    if !types_compatible(&vt.into(), &et) {
                        self.violations.push(self.path.clone());
                    } else {
                        // Types match, all good.
                        // Add this as a valid use.
                        self.paths_to_passed_checks.push(self.path.clone());
                    }
                } else {
                    // Either variable or expression type could not be determined, consider it a violation.
                    self.violations.push(self.path.clone());
                }
            } else if let Some(tree) = visited.downcast::<nonterminal_return_stmt>() {
                // First, what is the function we are in?
                if self.scope_trace.is_empty() {
                    // Not inside a function, so just keep going. This violation is caught by another visitor.
                    let mut result = visited.visit_each(self);
                    if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                        visitor.path.pop_back();
                    }
                    return result;
                }
                // Get the function name and type from the scope trace.
                if let Some((fn_name, fn_type)) = get_current_function(&self.func_defs, &self.scope_trace) {
                    // We have a function name and type.
                    // <return_stmt> ::= <return_kwd> | <return_kwd> <sep> <expr> ;
                    // What kind of return statement?
                    match tree.nth::<0>() {
                        nonterminal_return_stmt_0::variant_0(_) => {
                            // Void return.
                            // Check if function return type is void; get last elt from fn_type (a Vec)
                            // ... There has got to be a way to do this with unwrap_or_else ...
                            let some_last = fn_type.last();
                            let fn_return_type = match some_last {
                                Some(t) => t,
                                None => {
                                    // No return type found? This should never happen, but assume void just in case something is wrong with the grammar.
                                    &nonterminal_type::new(nonterminal_type_0::from_0th(nonterminal_basic_type::new(nonterminal_basic_type_0::from_5th(nonterminal_basic_type_0_5))))
                                }
                            };
                            match fn_return_type.nth::<0>() {
                                nonterminal_type_0::variant_0(bt) => {
                                    // Function return type is a basic type. Is that one void?
                                    match bt.nth::<0>() {
                                        // <basic_type> ::= "int" | "float" | "double" | "bool" | "char" | "void" ;
                                        nonterminal_basic_type_0::variant_5(_) => {
                                            // Basic type is void. All good.
                                            self.paths_to_passed_checks.push(self.path.clone());
                                        },
                                        _ => {
                                            // Basic type is not void. Violation.
                                            self.violations.push(self.path.clone());
                                        }
                                    }
                                },
                                nonterminal_type_0::variant_1(_) => {
                                    // Function return type is a struct type, not void. Violation.
                                    self.violations.push(self.path.clone());
                                },
                                _ => {
                                    // This should never happen. Violation just in case.
                                    self.violations.push(self.path.clone());
                                }
                            }
                        },
                        nonterminal_return_stmt_0::variant_1(ret_seq) => {
                            // Return with expression.
                            let ret_expr = ret_seq.nth::<2>();
                            let expr_type = infer_expr_type(ret_expr, &self.var_defs, &self.func_defs, &self.struct_defs, &self.scope_trace);
                            let some_last = fn_type.last();
                            let fn_return_type = match some_last {
                                Some(t) => t,
                                None => {
                                    // No return type found, assume void.
                                    &nonterminal_type::new(nonterminal_type_0::from_0th(nonterminal_basic_type::new(nonterminal_basic_type_0::from_5th(nonterminal_basic_type_0_5))))
                                }
                            };
                            if let Some(et) = expr_type {
                                if types_compatible(&fn_return_type.clone().into(), &et) {
                                    // Types match, all good.
                                    self.paths_to_passed_checks.push(self.path.clone());
                                } else {
                                    // Types do not match, violation.
                                    self.violations.push(self.path.clone());
                                }
                            } else {
                                // Could not infer expression type, consider it a violation.
                                self.violations.push(self.path.clone());
                            }
                        },
                    }
                } else {
                    // Could not determine current function, just continue; this violation is caught by another visitor.
                    let mut result = visited.visit_each(self);
                    if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                        visitor.path.pop_back();
                    }
                    return result;
                }
            } else if let Some(tree) = visited.downcast::<nonterminal_struct_access>() {
                // Get the var name.
                let var_name = tree.nth::<0>().nth::<0>().clone();
                // Look up the variable type.
                let var_type = get_var_definition(&self.var_defs, &var_name, &self.scope_trace);
                // Get the field name being accessed.
                let field_name = tree.nth::<0>().nth::<2>().clone();
                // Check if the variable type is a struct and if it has the field.
                if let Some(vt) = var_type {
                    if let nonterminal_type_0::variant_1(struct_type) = vt.nth::<0>() {
                        let struct_name = struct_type.nth::<0>().nth::<2>().clone();
                        if let Some(fields) = self.struct_defs.get(&struct_name) {
                            if !fields.iter().any(|(fname, _ftype)| fname == &field_name) {
                                // Field not found in struct definition, violation.
                                self.violations.push(self.path.clone());
                            } else {
                                // Field found, all good.
                                self.paths_to_passed_checks.push(self.path.clone());
                            }
                        }
                    }
                }
            } else if let Some(tree) = visited.downcast::<nonterminal_fn_call>() {
                // <fn_call> ::= <fn_name> "(" <arg_list_e> ")" ;
                // <arg_list_e> ::= <arg_list> | <e> ; 
                // <arg_list> ::= <arg> | <arg> "," <sep> <arg_list> ;
                let fn_name = tree.nth::<0>().nth::<0>().clone();
                // Look up the function definition.
                if let Some(param_types) = get_func_definition(&self.func_defs, &fn_name, &self.scope_trace) {
                    // We have the function definition.
                    // Now check the argument types.
                    let arg_list_e = tree.nth::<0>().nth::<2>();
                    // Is arg_list_e <e> or <arg_list>?
                    let mut arg_types = Vec::new();
                    if let Some(_e) = arg_list_e.nth::<0>().nth::<1>() {
                        // No arguments.
                        // If the function has parameters, this is a violation.
                        if param_types.len() > 1 {
                            // Function has parameters, but no arguments provided, violation.
                            self.violations.push(self.path.clone());
                        } else {
                            // No parameters, all good.
                            self.paths_to_passed_checks.push(self.path.clone());
                        }
                    } else {
                        // In this case, we know it's arg_list.
                        let mut current = arg_list_e.nth::<0>().nth::<0>();
                        while let Some(al) = current {
                            match al.nth::<0>() {
                                nonterminal_arg_list_0::variant_0(arg) => {
                                    // Single expression.
                                    let expr = arg.nth::<0>();
                                    if let Some(at) = infer_expr_type(&expr, &self.var_defs, &self.func_defs, &self.struct_defs, &self.scope_trace) {
                                        arg_types.push(at);
                                        // Successfully inferred argument type, a passed check.
                                        self.paths_to_passed_checks.push(self.path.clone());
                                    } else {
                                        // Could not infer argument type, consider it a violation.
                                        self.violations.push(self.path.clone());
                                    }
                                    current = None;
                                },
                                nonterminal_arg_list_0::variant_1(seq) => {
                                    // expr , sep , arg_list
                                    let arg = seq.nth::<0>();
                                    let expr = arg.nth::<0>();
                                    if let Some(at) = infer_expr_type(&expr, &self.var_defs, &self.func_defs, &self.struct_defs, &self.scope_trace) {
                                        arg_types.push(at);
                                        // Successfully inferred argument type, a passed check.
                                        self.paths_to_passed_checks.push(self.path.clone());
                                    } else {
                                        // Could not infer argument type, consider it a violation.
                                        self.violations.push(self.path.clone());
                                    }
                                    let (_, _, _, rest) = seq.children();
                                    current = Some(rest);
                                }
                            }
                        }
                    }
                    // Now compare arg_types with param_types (excluding the last type which is the return type).
                    if param_types.len() > 1 {
                        let expected_param_types = &param_types[..param_types.len()-1];
                        if expected_param_types.len() != arg_types.len() {
                            // Argument count mismatch, violation.
                            self.violations.push(self.path.clone());
                        } else {
                            let mut mismatch_found = false;
                            for (expected, actual) in expected_param_types.iter().zip(arg_types.iter()) {
                                if !types_compatible(&expected.clone().into(), actual) {
                                    // Type mismatch, violation.
                                    self.violations.push(self.path.clone());
                                    mismatch_found = true;
                                    break;
                                }
                            }
                            // If we reach here, all argument types match.
                            if !mismatch_found {
                                self.paths_to_passed_checks.push(self.path.clone());
                            }
                        }
                    } else {
                        // Function has no parameters, but arguments were provided, violation.
                        if !arg_types.is_empty() {
                            self.violations.push(self.path.clone());
                        } else {
                            // No arguments, all good.
                            self.paths_to_passed_checks.push(self.path.clone());
                        }
                    }
                } else {
                    // Function not found, but that's a def-use violation, not a typing violation.
                    let mut result = visited.visit_each(self);
                    if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                        visitor.path.pop_back();
                    }
                    return result;
                }
            } else if let Some(tree) = visited.downcast::<nonterminal_expr_stmt>() {
                // <expr_stmt> ::= <expr> ";" ;
                let expr = tree.nth::<0>();
                // Just infer the type to see if it can be inferred.
                if infer_expr_type(expr, &self.var_defs, &self.func_defs, &self.struct_defs, &self.scope_trace).is_some() {
                    // Type could be inferred, all good.
                    self.paths_to_passed_checks.push(self.path.clone());
                } else {
                    // Could not infer type, consider it a violation.
                    self.violations.push(self.path.clone());
                }
            }

            // Finally, continue visiting children.
            let mut result = visited.visit_each(self);
                if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                    visitor.path.pop_back();
                }
            return result;
        }
    }

    // Ok, now let's make one ConstraintVisitor that combines all the previous ones.
    /// Combined constraint visitor that checks for return-in-fn,
    /// struct-access, def-use, and type-checking constraints.
    #[derive(Debug, Default)]
    pub struct CombinedConstraintVisitor {
        /// The collected violations.
        pub violation_list: Vec<VecDeque<usize>>,
        /// The collected non-violations.
        pub paths_to_passed_checks: Vec<VecDeque<usize>>,
    }

    impl Checker for CombinedConstraintVisitor {
        fn violations(self) -> Violations {
            Violations::new(
                // A bit of a verbose calculation, condense.
                if self.violation_list.is_empty() && self.paths_to_passed_checks.is_empty() {
                    // No checks were performed, return default ratio.
                    Default::default()
                } else if !self.violation_list.is_empty() && self.paths_to_passed_checks.is_empty() {
                    // All checks failed.
                    Ratio::new(0, self.violation_list.len())
                } else if self.violation_list.is_empty() && !self.paths_to_passed_checks.is_empty() {
                    // All checks passed.
                    Ratio::new(self.paths_to_passed_checks.len(), self.paths_to_passed_checks.len())
                } else {
                    // Some checks passed, some failed.
                    Ratio::new(self.paths_to_passed_checks.len(), self.violation_list.len() + self.paths_to_passed_checks.len())
                },
                self.violation_list,
            )
        }
    }

    impl <T> Visitor<T> for CombinedConstraintVisitor
    where
        T: VisitableChildren<T> +
            AsNodeRef<nonterminal_start> +
            AsNodeRef<nonterminal_decl> +
            AsNodeRef<nonterminal_var_name> +
            AsNodeRef<nonterminal_type> +
            AsNodeRef<nonterminal_fn_def> +
            AsNodeRef<nonterminal_struct_def> +
            AsNodeRef<nonterminal_assignment> +
            AsNodeRef<nonterminal_expr> +
            AsNodeRef<nonterminal_var_access> +
            AsNodeRef<nonterminal_fn_name> +
            AsNodeRef<nonterminal_param_list> +
            AsNodeRef<nonterminal_param_list_e> +
            AsNodeRef<nonterminal_fn_call> +
            AsNodeRef<nonterminal_return_stmt> +
            AsNodeRef<nonterminal_struct_access> +
            AsNodeRef<nonterminal_field_name> +
            AsNodeRef<nonterminal_expr_stmt> + 
            AsNodeRef<nonterminal_basic_type> +
            AsNodeRef<nonterminal_struct_type> +
            AsNodeRef<nonterminal_param_name> + 
            AsNodeRef<nonterminal_struct_name> +
            AsNodeRef<nonterminal_arith_expr> +
            AsNodeRef<nonterminal_binop> +
            AsNodeRef<nonterminal_binop_op> +
            AsNodeRef<nonterminal_unop> +
            AsNodeRef<nonterminal_unop_op> +
            AsNodeRef<nonterminal_field_def_list> +
            AsNodeRef<nonterminal_field_def_list_e> + 
            AsNodeRef<nonterminal_param> +
            AsNodeRef<nonterminal_struct_expr> +
            AsNodeRef<nonterminal_expr_list_e>,
    {
        type Continue = Self;
        type Break = Infallible;
        type Error = Infallible;

        fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
        where
            N: Node<Type<'program> = T>,
            T: From<&'program N> + AsNodeRef<N>,
        {
            let visited = node.opaque(); 
            if let Some(_tree) = visited.downcast::<nonterminal_start>() {
                // First, collect declarations.
                let Ok(ControlFlow::Continue(decl_collector)) = DeclarationCollector::default().visit(node, idx);
                // Now run each constraint visitor in sequence, passing along the collected definitions.
                let var_defs = &decl_collector.var_defs;
                let func_defs = &decl_collector.func_defs;
                let struct_defs = &decl_collector.struct_defs;
                
                // No empty struct visitor.
                let Ok(ControlFlow::Continue(empty_struct_visitor)) = ConstraintVisitorNoEmptyStructs::default().visit(node, idx);
                self.violation_list.extend(empty_struct_visitor.violations);
                self.paths_to_passed_checks.extend(empty_struct_visitor.paths_to_passed_checks);

                // No void decls or params visitor. This one has Default implemented.
                let Ok(ControlFlow::Continue(void_decl_visitor)) = ConstraintVisitorNoVoidDecls::default().visit(node, idx);
                self.violation_list.extend(void_decl_visitor.violations);
                self.paths_to_passed_checks.extend(void_decl_visitor.paths_to_passed_checks);

                // Return-in-fn constraint visitor. This one has Default implemented.
                let Ok(ControlFlow::Continue(ret_in_fn_visitor)) = ConstraintVisitorReturnInFunc::default().visit(node, idx);
                self.violation_list.extend(ret_in_fn_visitor.violations);
                self.paths_to_passed_checks.extend(ret_in_fn_visitor.paths_to_passed_checks);
                
                // Struct-access constraint visitor. This one needs struct_defs and var_defs.
                let Ok(ControlFlow::Continue(struct_access_visitor)) = ConstraintVisitorStructAccess::new(struct_defs, var_defs).visit(node, idx);
                self.violation_list.extend(struct_access_visitor.violations);
                self.paths_to_passed_checks.extend(struct_access_visitor.paths_to_passed_checks);

                // Def-use constraint visitor. This one needs var_defs, func_defs, struct_defs.
                let Ok(ControlFlow::Continue(def_use_visitor)) = ConstraintVisitorDefUse::new(var_defs, func_defs, struct_defs).visit(node, idx);
                self.violation_list.extend(def_use_visitor.violations);
                self.paths_to_passed_checks.extend(def_use_visitor.paths_to_passed_checks);

                // Limit usage of struct expressions.
                let Ok(ControlFlow::Continue(struct_expr_visitor)) = ConstraintVisitorStructExprRHSOfDecl::default().visit(node, idx);
                self.violation_list.extend(struct_expr_visitor.violations);
                self.paths_to_passed_checks.extend(struct_expr_visitor.paths_to_passed_checks);

                // Type-checking constraint visitor. This one needs var_defs, func_defs, struct_defs.
                let Ok(ControlFlow::Continue(type_check_visitor)) = ConstraintVisitorTypeCheck::new(var_defs, func_defs, struct_defs).visit(node, idx);
                self.violation_list.extend(type_check_visitor.violations);
                self.paths_to_passed_checks.extend(type_check_visitor.paths_to_passed_checks);

                // KeepReasonableStructVisitor
                // let Ok(ControlFlow::Continue(keep_structs_reasonable_visitor)) = KeepReasonableStructVisitor::default().visit(node, idx);
                // self.violation_list.extend(keep_structs_reasonable_visitor.violations);
                // self.paths_to_passed_checks.extend(keep_structs_reasonable_visitor.paths_to_passed_checks);
            }
            // There's no reason to continue, that should be the only node we visit.
            Ok(ControlFlow::Continue(self))
        }
    }
=======
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
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)

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
<<<<<<< HEAD
        use fandango::visitor::{Visitor};
=======
        use fandango::visitor::{Visitor, VisitorMut};
        use fandango::visitor::navigation::GoTo;
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use alloc::string::String;
        use fandango::visitor::write::WriteVisitor;
        use alloc::vec::Vec;
<<<<<<< HEAD
        use fandango_runtime::measurement::Violations;
        use num_rational::Ratio;
=======

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
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)

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
<<<<<<< HEAD
=======
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
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
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
<<<<<<< HEAD
                })) = lang::ConstraintVisitorStructAccess ::new(&struct_defs, &Default::default()).visit(&tree, 0);
=======
                })) = lang::ConstraintVisitorStructAccess {
                    struct_defs,
                    ..Default::default()
                }.visit(&tree, 0);
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)

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
<<<<<<< HEAD

        // Def-use
        #[test]
        fn check_def_use_constraint_c() -> Result<(), Box<dyn Error>> {
            extern crate std;
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(lang::nonterminal_start::ROOT.inner(), 50));
            // Generate 200 programs and check for violations.
            for i in 0..200 {
                let tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
                let Ok(ControlFlow::Continue(lang::DeclarationCollector { 
                    var_defs, func_defs, struct_defs, ..
                })) = lang::DeclarationCollector::default().visit(&tree, 0);    
                std::println!("==============================");
                std::println!("Program {i} has {} variable definitions.", var_defs.len());
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorDefUse { 
                    violations, .. 
                })) = lang::ConstraintVisitorDefUse::new(&var_defs, &func_defs, &struct_defs).visit(&tree, 0);
                std::println!("Program {i} has {} def-before-use violations.", violations.len());
                // Print the program.
                std::println!("Program:\n{}", String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(&tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap().output(),
                ).unwrap());
            }
            Ok(())
        }

        // Ok, let's test type checking.
        #[test]
        fn check_type_checking_c() -> Result<(), Box<dyn Error>> {
            extern crate std;
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(lang::nonterminal_start::ROOT.inner(), 50));
            // Generate 200 programs and check for violations.
            for i in 0..200 {
                let tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
                let Ok(ControlFlow::Continue(lang::DeclarationCollector { 
                    var_defs, func_defs, struct_defs, .. 
                })) = lang::DeclarationCollector::default().visit(&tree, 0);    
                std::println!("==============================");
                std::println!("Program {i} has {} variable definitions, {} function definitions, and {} struct definitions.", var_defs.len(), func_defs.len(), struct_defs.len());
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorTypeCheck {
                    violations, ..  
                })) = lang::ConstraintVisitorTypeCheck::new(&var_defs, &func_defs, &struct_defs).visit(&tree, 0);
                std::println!("Program {i} has {} type-checking violations.", violations.len());
                // Print the program.
                std::println!("Program:\n{}", String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(&tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap().output(),
                ).unwrap());
            }
            Ok(())
        }

        // Test all constraints together.
        #[test]
        fn check_all_constraints_c() -> Result<(), Box<dyn Error>> {
            extern crate std;
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(lang::nonterminal_start::ROOT.inner(), 50));
            // Generate 200 programs and check for violations.
            for i in 0..200 {
                let tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
                let Ok(ControlFlow::Continue(lang::DeclarationCollector { 
                    var_defs, func_defs, struct_defs, .. 
                })) = lang::DeclarationCollector::default().visit(&tree, 0);    
                std::println!("==============================");
                std::println!("Program {i} has {} variable definitions, {} function definitions, and {} struct definitions.", var_defs.len(), func_defs.len(), struct_defs.len());
                let mut total_violations = 0;
                // Def-before-use
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorDefUse { 
                    violations, .. 
                })) = lang::ConstraintVisitorDefUse::new(&var_defs, &func_defs, &struct_defs).visit(&tree, 0);
                std::println!("Program {i} has {} def-before-use violations.", violations.len());
                total_violations += violations.len();   
                // Fn-call arg count
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorReturnInFunc { 
                    violations, .. 
                })) = lang::ConstraintVisitorReturnInFunc::default().visit(&tree, 0);
                std::println!("Program {i} has {} return-in-fn violations.", violations.len());
                total_violations += violations.len();
                // Struct access
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorStructAccess { 
                    violations, .. 
                })) = lang::ConstraintVisitorStructAccess::new(&struct_defs, &var_defs).visit(&tree, 0);
                std::println!("Program {i} has {} struct-access violations.", violations.len());
                total_violations += violations.len();
                // Type checking
                let Ok(ControlFlow::Continue(lang::ConstraintVisitorTypeCheck { 
                    violations, .. 
                })) = lang::ConstraintVisitorTypeCheck::new(&var_defs, &func_defs, &struct_defs).visit(&tree, 0);
                std::println!("Program {i} has {} type-checking violations.", violations.len());
                total_violations += violations.len();
                std::println!("Program {i} has a total of {} violations.", total_violations);
                // Print the program.
                std::println!("Program:\n{}", String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(&tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap().output(),
                ).unwrap());
            }
            Ok(())
        }

        // Test the combined visitor.
        #[test]
        fn check_combined_constraints_c() -> Result<(), Box<dyn Error>> {
            extern crate std;
            let mut rng = StdRng::seed_from_u64(0);
            let mut generators =
                tuple_list!(DepthLimiter::new(lang::nonterminal_start::ROOT.inner(), 50));
            // Generate 200 programs and check for violations.
            for i in 0..200 {
                let tree = lang::nonterminal_start::generate(&mut rng, &mut generators, 0);
                std::println!("==============================");
                let combined_constraint_visitor = lang::CombinedConstraintVisitor::default();
                let Ok(ControlFlow::Continue(lang::CombinedConstraintVisitor {
                    violation_list, paths_to_passed_checks, ..
                })) = combined_constraint_visitor.visit(&tree, 0);
                std::println!("Program {i} has {} violations and {} passed checks.", violation_list.len(), paths_to_passed_checks.len());
                // Print the program.
                std::println!("Program:\n{}", String::from_utf8(
                    WriteVisitor::new(Vec::new())
                        .visit(&tree, 0)
                        .unwrap()
                        .continue_value()
                        .unwrap().output(),
                ).unwrap());    

                // Also print the violations returned by the Checker impl.
                // Just copy the implementation here to check.
                let violations = Violations::new(
                    if violation_list.is_empty() && paths_to_passed_checks.is_empty() {
                        // No checks were performed, return default ratio.
                        Default::default()
                    } else if !violation_list.is_empty() && paths_to_passed_checks.is_empty() {
                        // All checks failed.
                        Ratio::new(0, violation_list.len())
                    } else if violation_list.is_empty() && !paths_to_passed_checks.is_empty() {
                        // All checks passed.
                        Ratio::new(paths_to_passed_checks.len(), paths_to_passed_checks.len())
                    } else {
                        // Some checks passed, some failed.
                        Ratio::new(paths_to_passed_checks.len(), violation_list.len() + paths_to_passed_checks.len())
                    },
                    violation_list,
                );
                std::println!("Checker reports violations with ratio {:?}.", violations.pass_rate());
            }
            Ok(())
        }
=======
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
    }
}

pub use defs::*;