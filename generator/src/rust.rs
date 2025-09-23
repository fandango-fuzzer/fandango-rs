use fandango_core::graph::shortest_path;
use fandango_core::lang::FandangoNode;
use fandango_core::lang::Operator;
use hashbrown::hash_map::Entry;
use hashbrown::{HashMap, HashSet};
use pest::Span;
use petgraph::graph::DiGraph;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::{Direction, algo, graph};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use std::convert::Infallible;

/// Produces a Rust source tree using the provided context.
pub trait IntoRustSource<C> {
    /// The error type which is encountered as a result of trying to emit the source code.
    type OutputError;

    /// Emits the corresponding Rust code for this structure.
    fn emit_rust(&self, ctx: C, output: &mut TokenStream) -> Result<(), Self::OutputError>;
}

fn from_boilerplate(name: &Ident) -> TokenStream {
    quote! {
        impl<'program> ::core::convert::From<&'program #name> for Type<'program> {
            fn from(node: &'program #name) -> Type<'program> {
                Type::#name(node)
            }
        }

        impl<'program> ::core::convert::From<&'program mut #name> for Type<'program> {
            fn from(node: &'program mut #name) -> Type<'program> {
                Type::#name(node)
            }
        }

        impl<'program> ::core::convert::From<&'program mut #name> for TypeMut<'program> {
            fn from(node: &'program mut #name) -> TypeMut<'program> {
                TypeMut::#name(node)
            }
        }

        impl<'program> ::core::convert::From<&'program ::alloc::boxed::Box<#name>> for Type<'program> {
            fn from(node: &'program ::alloc::boxed::Box<#name>) -> Type<'program> {
                Type::#name(&**node)
            }
        }

        impl<'program> ::core::convert::From<&'program mut ::alloc::boxed::Box<#name>> for Type<'program> {
            fn from(node: &'program mut ::alloc::boxed::Box<#name>) -> Type<'program> {
                Type::#name(&**node)
            }
        }

        impl<'program> ::core::convert::From<&'program mut ::alloc::boxed::Box<#name>> for TypeMut<'program> {
            fn from(node: &'program mut ::alloc::boxed::Box<#name>) -> TypeMut<'program> {
                TypeMut::#name(&mut **node)
            }
        }
    }
}

impl<'graph, 'program, 'source>
    IntoRustSource<(
        &'graph mut HashMap<FandangoNode<'program, 'source>, Ident>,
        bool,
        bool,
        Span<'source>,
        Span<'source>,
    )> for DiGraph<FandangoNode<'program, 'source>, Span<'source>>
where
    'program: 'graph,
{
    type OutputError = Infallible;

    fn emit_rust(
        &self,
        (mapped_names, emit_parse_glue, serde, decl, definition): (
            &'graph mut HashMap<FandangoNode<'program, 'source>, Ident>,
            bool,
            bool,
            Span<'source>,
            Span<'source>,
        ),
        output: &mut TokenStream,
    ) -> Result<(), Self::OutputError> {
        let start_node = self
            .node_indices()
            .find(|&n| matches!(self.node_weight(n).unwrap(), FandangoNode::Nonterminal(nt) if nt.name() == "start"))
            .expect("No start node?");

        let needs_indirection = if cfg!(no_opt_indirect) {
            self.node_references()
                .filter_map(|(n, weight)| match weight {
                    FandangoNode::Nonterminal(_) => Some(n),
                    _ => None,
                })
                .flat_map(|n| self.edges_directed(n, Direction::Incoming))
                .map(|e| {
                    (
                        *self.node_weight(e.source()).unwrap(),
                        *self.node_weight(e.target()).unwrap(),
                    )
                })
                .collect()
        } else {
            let vec_pruned = self.filter_map(
                |_n, w| Some(*w),
                |e, w| {
                    self.edge_endpoints(e).and_then(|(n1, _)| {
                        (!matches!(
                            self.node_weight(n1).unwrap(),
                            FandangoNode::Operator(Operator::Kleene(_))
                                | FandangoNode::Operator(Operator::Plus(_))
                                | FandangoNode::Operator(Operator::Repeat(_, _, _))
                        ))
                        .then_some(*w)
                    })
                },
            );
            algo::greedy_feedback_arc_set(&vec_pruned)
                .map(|e| {
                    (
                        *vec_pruned.node_weight(e.source()).unwrap(),
                        *vec_pruned.node_weight(e.target()).unwrap(),
                    )
                })
                .collect::<HashSet<_>>()
        };

        let node_weight = *self.node_weight(start_node).unwrap();

        let FandangoNode::Nonterminal(nt) = node_weight else {
            unimplemented!("Can only transforms non-terminals into source code.")
        };

        let pest_name = format_ident!("{}", nt.name());
        let name = format_ident!("nonterminal_{}", nt.name());

        let derives = if serde {
            quote! {
                #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, ::serde::Serialize, ::serde::Deserialize)]
            }
        } else {
            quote! {
                #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
            }
        };

        let shortest_paths = shortest_path(self);

        if emit_parse_glue {
            output.extend(quote! {
                pub type ParseError = ::alloc::boxed::Box<::pest::error::Error<Rule>>;
            });
        }

        start_node.emit_rust(
            (
                name,
                pest_name,
                node_weight,
                decl,
                definition,
                mapped_names,
                self,
                &needs_indirection,
                emit_parse_glue,
                &derives,
                &shortest_paths,
                true,
            ),
            output,
        )
    }
}

type FandangoGenContext<'names, 'graph, 'program, 'source> = (
    Ident,
    Ident,
    FandangoNode<'program, 'source>,
    Span<'source>,
    Span<'source>,
    &'names mut HashMap<FandangoNode<'program, 'source>, Ident>,
    &'graph DiGraph<FandangoNode<'program, 'source>, Span<'source>>,
    &'graph HashSet<(
        FandangoNode<'program, 'source>,
        FandangoNode<'program, 'source>,
    )>,
    bool,
    &'names TokenStream,
    &'names HashMap<FandangoNode<'program, 'source>, Vec<usize>>,
    bool,
);

impl<'program, 'source> IntoRustSource<FandangoGenContext<'_, '_, 'program, 'source>>
    for graph::NodeIndex
{
    type OutputError = Infallible;

    fn emit_rust(
        &self,
        ctx: FandangoGenContext<'_, '_, 'program, 'source>,
        output: &mut TokenStream,
    ) -> Result<(), Self::OutputError> {
        #[allow(unused_assignments)]
        let (
            name,
            pest_name,
            node_weight,
            mut span,
            mut last_nonterminal,
            mapped_names,
            graph,
            needs_indirection,
            emit_parse_glue,
            derives,
            shortest_paths,
            starting_symbol,
        ) = ctx;
        match mapped_names.entry(node_weight) {
            Entry::Occupied(_) => return Ok(()),
            Entry::Vacant(e) => {
                e.insert(name.clone());
            }
        }

        let mut local_output = TokenStream::new();

        let from = from_boilerplate(&name);

        let mut children = graph
            .edges(*self)
            .map(|e| {
                (
                    e.source(),
                    e.target(),
                    graph.node_weight(e.target()).copied().unwrap(),
                    *e.weight(),
                )
            })
            .collect::<Vec<_>>();
        children.sort_by_key(|(_, _, _, w)| w.start());
        let pest_child_names = children
            .iter()
            .enumerate()
            .map(|(i, (_, _, child, _))| {
                if let FandangoNode::Nonterminal(nt) = child {
                    format_ident!("{}", nt.name())
                } else {
                    format_ident!("{pest_name}_{i}")
                }
            })
            .collect::<Vec<_>>();
        let child_types = children
            .iter()
            .enumerate()
            .map(|(i, (_, _, child, _))| {
                if let FandangoNode::Nonterminal(nt) = child {
                    format_ident!("nonterminal_{}", nt.name())
                } else {
                    format_ident!("{name}_{i}")
                }
            })
            .collect::<Vec<_>>();

        let (
            child_ref_types,
            child_mut_types,
            child_field_types,
            ref_visit_prefixes,
            visit_prefixes,
            generate_prefixes,
        ) = children
            .iter()
            .zip(&child_types)
            .map(|((_, _, child, _), name)| {
                let base = quote! { #name };
                match node_weight {
                    FandangoNode::Operator(op) => match op {
                        Operator::Kleene(_) | Operator::Plus(_) | Operator::Repeat(_, _, _) => (
                            quote! { ::core::option::Option<&'a #base> },
                            quote! { ::core::option::Option<&'a mut #base> },
                            quote! { ::alloc::vec::Vec<#base> },
                            quote! {},
                            quote! {},
                            quote! {},
                        ),
                        Operator::Option(_) => {
                            if needs_indirection.contains(&(node_weight, *child)) {
                                (
                                    quote! { ::core::option::Option<&'a #base> },
                                    quote! { ::core::option::Option<&'a mut #base> },
                                    quote! { ::core::option::Option<::alloc::boxed::Box<#base>> },
                                    quote! { ::core::ops::Deref::deref },
                                    quote! { ::core::ops::DerefMut::deref_mut },
                                    quote! { ::alloc::boxed::Box::new },
                                )
                            } else {
                                (
                                    quote! { ::core::option::Option<&'a #base> },
                                    quote! { ::core::option::Option<&'a mut #base> },
                                    quote! { ::core::option::Option<#base> },
                                    quote! {},
                                    quote! {},
                                    quote! {},
                                )
                            }
                        }
                        Operator::Symbol(_) => {
                            unimplemented!("Unexpected symbol; should be elided.")
                        }
                    },
                    _ => {
                        if needs_indirection.contains(&(node_weight, *child)) {
                            (
                                quote! { &'a #base },
                                quote! { &'a mut #base },
                                quote! { ::alloc::boxed::Box<#base> },
                                quote! { ::core::ops::Deref::deref },
                                quote! { ::core::ops::DerefMut::deref_mut },
                                quote! { ::alloc::boxed::Box::new },
                            )
                        } else {
                            (
                                quote! { &'a #base },
                                quote! { &'a mut #base },
                                base,
                                quote! {},
                                quote! {},
                                quote! {},
                            )
                        }
                    }
                }
            })
            .collect::<(Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>)>();

        match node_weight {
            FandangoNode::String(orig) => {
                let s = Literal::byte_string(orig.inner());
                let parse_routine = if let Ok(parsed) = core::str::from_utf8(orig.inner()) {
                    quote! {
                        let span = value.as_span();
                        debug_assert_eq!(span.as_str(), #parsed);

                        Ok(Self)
                    }
                } else {
                    quote! { unimplemented!("Pest currently does not support byte-like grammars: https://github.com/pest-parser/pest/issues/244") }
                };
                local_output.extend(quote! {
                    #derives
                    #[derive(Default)]
                    #[allow(missing_docs)]
                    pub struct #name;

                    impl ::fandango::typing::Node for #name {
                        type Type<'program> = Type<'program>;
                        type TypeMut<'program> = TypeMut<'program>;
                        type ChildrenRef<'program> = (&'static [u8],);
                        type ChildrenRefMut<'program> = (&'static [u8],);

                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> { (#s.as_slice(),) }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> { (#s.as_slice(),) }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildren<Type<'program>> for &'program #name
                    {
                        fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V> {
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                        {
                            self.visit_each(visitor)
                        }

                        fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                        {
                            self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                        }

                        fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                        {
                            self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                        }

                        fn visit_nth<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>> {
                            Err(visitor)
                        }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildrenMut<TypeMut<'program>> for &'program mut #name
                    {
                        fn visit_each_mut<V>(self, visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V> {
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_mut<V>(self, visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                        {
                            self.visit_each_mut(visitor)
                        }

                        fn visit_each_reverse_mut_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                        {
                            self.visit_nth_mut(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                        }

                        fn visit_each_mut_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                        {
                            self.visit_nth_mut(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                        }

                        fn visit_nth_mut<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>> {
                            Err(visitor)
                        }
                    }

                    impl<S, G> ::fandango::generation::DefaultGenerated<S, G> for #name {
                        fn generate_default(sampler: &mut S, with: &mut G, _: usize) -> Self {
                            Self
                        }
                    }

                    #from
                });
                if emit_parse_glue {
                    local_output.extend(quote! {
                        impl ::core::convert::TryFrom<::pest::iterators::Pair<'_, Rule>> for #name {
                            type Error = ParseError;

                            fn try_from(value: ::pest::iterators::Pair<'_, Rule>) -> Result<Self, Self::Error> {
                                #parse_routine
                            }
                        }
                    });
                }
            }
            FandangoNode::Alternative(_) => {
                let child_variants = (0..children.len())
                    .map(|i| format_ident!("variant_{i}"))
                    .collect::<Vec<_>>();
                let indices = (0..children.len()).collect::<Vec<_>>();
                let count = children.len();
                let child_range_docs = (0usize..).map(|v| v.to_string());
                let child_range = 0usize..;

                let shortest = shortest_paths.get(&node_weight).unwrap();
                let default_choice = shortest[0];
                let default = &child_variants[default_choice];
                local_output.extend(quote! {
                    #derives
                    #[allow(missing_docs)]
                    pub enum #name {
                        #(
                            #[doc = concat!(#child_range_docs, "th variant of [`", stringify!(#name), "`] which maps to [`", stringify!(#child_types), "`]")]
                            #child_variants ( #child_field_types )
                        ),*
                    }

                    impl Default for #name {
                        fn default() -> Self {
                            Self::#default(Default::default())
                        }
                    }

                    impl ::fandango::typing::Node for #name {
                        type Type<'program> = Type<'program>;
                        type TypeMut<'program> = TypeMut<'program>;
                        type ChildrenRef<'program> = &'program Self;
                        type ChildrenRefMut<'program> = &'program mut Self;

                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
                            self
                        }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
                            self
                        }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildren<Type<'program>> for &'program #name
                    {
                        fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V> {
                            match self {
                                #(#name::#child_variants(n) => visitor.visit(#ref_visit_prefixes(n), #indices)),*
                            }
                        }

                        fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                        {
                            self.visit_each(visitor)
                        }

                        fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                        {
                            match self {
                                #(#name::#child_variants(n) if idx >= #indices => visitor.visit(#ref_visit_prefixes(n), idx)),*,
                                _ => Ok(::core::ops::ControlFlow::Continue(visitor))
                            }
                        }

                        fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                        {
                            match self {
                                #(#name::#child_variants(n) if idx <= #indices => visitor.visit(#ref_visit_prefixes(n), idx)),*,
                                _ => Ok(::core::ops::ControlFlow::Continue(visitor))
                            }
                        }

                        fn visit_nth<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>> {
                            match self {
                                #(#name::#child_variants(n) if idx == #indices => Ok(visitor.visit(#ref_visit_prefixes(n), idx))),*,
                                _ => Err(visitor)
                            }
                        }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildrenMut<TypeMut<'program>> for &'program mut #name
                    {
                        fn visit_each_mut<V>(self, visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V> {
                            match self {
                                #(#name::#child_variants(n) => visitor.visit_mut(#visit_prefixes(n), #indices)),*
                            }
                        }

                        fn visit_each_reverse_mut<V>(self, visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                        {
                            self.visit_each_mut(visitor)
                        }

                        fn visit_each_reverse_mut_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                        {
                            match self {
                                #(#name::#child_variants(n) if idx >= #indices => visitor.visit_mut(#visit_prefixes(n), idx)),*,
                                _ => Ok(::core::ops::ControlFlow::Continue(visitor))
                            }
                        }

                        fn visit_each_mut_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                        {
                            match self {
                                #(#name::#child_variants(n) if idx <= #indices => visitor.visit_mut(#visit_prefixes(n), idx)),*,
                                _ => Ok(::core::ops::ControlFlow::Continue(visitor))
                            }
                        }

                        fn visit_nth_mut<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>> {
                            match self {
                                #(#name::#child_variants(n) if idx == #indices => Ok(visitor.visit_mut(#visit_prefixes(n), idx))),*,
                                _ => Err(visitor)
                            }
                        }
                    }

                    impl<S, G> ::fandango::generation::DefaultGenerated<S, G> for #name
                    where
                        S: TypeSampler,
                        G: TypeGenerator<S>,
                    {
                        fn generate_default(sampler: &mut S, with: &mut G, depth: usize) -> Self {
                            match <S as ::fandango::generation::Sampler<Self>>::sample_alternative(sampler, #count) {
                                #(#indices => Self::#child_variants(#generate_prefixes(::fandango::generation::Generated::generate(sampler, with, depth + 1)))),*,
                                _ => unreachable!()
                            }
                        }
                    }

                    #(
                        impl ::fandango::typing::ChildAccessor<#child_range> for #name {
                            type Child<'a> = Option<#child_ref_types>;
                            type ChildMut<'a> = Option<#child_mut_types>;

                            fn child(&self) -> Self::Child<'_> {
                                match self {
                                    #name::#child_variants(n) => Some(#ref_visit_prefixes(n)),
                                    _ => None,
                                }
                            }

                            fn child_mut(&mut self) -> Self::ChildMut<'_> {
                                match self {
                                    #name::#child_variants(n) => Some(#visit_prefixes(n)),
                                    _ => None,
                                }
                            }
                        }
                    )*

                    #from
                });
                if emit_parse_glue {
                    local_output.extend(quote! {
                        impl ::core::convert::TryFrom<::pest::iterators::Pair<'_, Rule>> for #name {
                            type Error = ParseError;

                            fn try_from(value: ::pest::iterators::Pair<'_, Rule>) -> Result<Self, Self::Error> {
                                debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                                let mut children = value.into_inner();
                                let child_0 = children.next().expect("Expected exactly one descendant.");
                                debug_assert!(children.next().is_none(), "Expected exactly one descendant.");

                                Ok(match child_0.as_rule() {
                                    #(Rule::#pest_child_names => #name::#child_variants(
                                        #child_types::try_from(child_0)?.into()
                                    )),*,
                                    _ => unimplemented!("Not a child of this alternative.")
                                })
                            }
                        }
                    });
                }
            }
            FandangoNode::Operator(op) => {
                // TODO auto-resolve through box so the user doesn't have to deal with indirection
                assert_eq!(children.len(), 1);
                let ref_prefix = &ref_visit_prefixes[0];
                let prefix = &visit_prefixes[0];

                let range_check_fail = match op {
                    Operator::Repeat(_, start, end) => {
                        quote! {
                            if children.len() < #start || #end < children.len() {
                                todo!()
                            }
                        }
                    }
                    Operator::Option(_) | Operator::Kleene(_) | Operator::Plus(_) => {
                        TokenStream::new()
                    }
                    Operator::Symbol(_) => {
                        unimplemented!("Unexpected symbol; should be elided.")
                    }
                };
                let child_type = match op {
                    Operator::Repeat(_, _, _) | Operator::Kleene(_) | Operator::Plus(_) => {
                        quote! { ::alloc::vec::Vec<#(#child_types),*> }
                    }
                    Operator::Option(_) => {
                        quote! { ::core::option::Option<#(#child_types),*> }
                    }
                    Operator::Symbol(_) => {
                        unimplemented!("Unexpected symbol; should be elided.")
                    }
                };
                let sampler = match op {
                    Operator::Kleene(_) => {
                        quote! {
                            (0..<S as ::fandango::generation::Sampler<Self>>::sample_kleene(sampler)).map(|_| ::fandango::generation::Generated::generate(sampler, with, depth + 1)).collect()
                        }
                    }
                    Operator::Plus(_) => {
                        quote! {
                            (0..<S as ::fandango::generation::Sampler<Self>>::sample_plus(sampler)).map(|_| ::fandango::generation::Generated::generate(sampler, with, depth + 1)).collect()
                        }
                    }
                    Operator::Option(_) => {
                        quote! {
                            <S as ::fandango::generation::Sampler<Self>>::sample_optional(sampler).then(|| #(#generate_prefixes)*(::fandango::generation::Generated::generate(sampler, with, depth + 1)))
                        }
                    }
                    Operator::Repeat(_, start, end) => {
                        quote! {
                            (0..<S as ::fandango::generation::Sampler<Self>>::sample_repetition(sampler, #start, #end)).map(|_| ::fandango::generation::Generated::generate(sampler, with, depth + 1)).collect()
                        }
                    }
                    Operator::Symbol(_) => {
                        unimplemented!("Unexpected symbol; should be elided.")
                    }
                };
                let default = match op {
                    Operator::Repeat(_, start, _) => {
                        quote! {
                            (0..#start).map(|_| Default::default()).collect()
                        }
                    }
                    Operator::Plus(_) => {
                        quote! {
                            (0..1).map(|_| Default::default()).collect()
                        }
                    }
                    _ => {
                        quote! {
                            Default::default()
                        }
                    }
                };
                let child_accessor = match op {
                    Operator::Kleene(_) | Operator::Plus(_) | Operator::Repeat(_, _, _) => {
                        quote! {
                            #(
                                impl<const N: usize> ::fandango::typing::ChildAccessor<N> for #name {
                                    type Child<'a> = #child_ref_types;
                                    type ChildMut<'a> = #child_mut_types;

                                    fn child(&self) -> Self::Child<'_> {
                                        #ref_visit_prefixes(self.child_0.get(N))
                                    }

                                    fn child_mut(&mut self) -> Self::ChildMut<'_> {
                                        #visit_prefixes(self.child_0.get_mut(N))
                                    }
                                }
                            )*
                        }
                    }
                    Operator::Option(_) => {
                        quote! {
                            #(
                                impl ::fandango::typing::ChildAccessor<0> for #name {
                                    type Child<'a> = #child_ref_types;
                                    type ChildMut<'a> = #child_mut_types;

                                    fn child(&self) -> Self::Child<'_> {
                                        #ref_visit_prefixes(self.child_0.as_ref(N))
                                    }

                                    fn child_mut(&mut self) -> Self::ChildMut<'_> {
                                        #visit_prefixes(self.child_0.as_mut(N))
                                    }
                                }
                            )*
                        }
                    }
                    Operator::Symbol(_) => {
                        unimplemented!("Unexpected symbol; should be elided.")
                    }
                };

                local_output.extend(quote! {
                    #derives
                    #[allow(missing_docs)]
                    pub struct #name {
                        child_0: #(#child_field_types)*
                    }

                    impl Default for #name {
                        fn default() -> Self {
                            Self {
                                child_0: #default,
                            }
                        }
                    }

                    impl ::fandango::typing::Node for #name {
                        type Type<'program> = Type<'program>;
                        type TypeMut<'program> = TypeMut<'program>;
                        type ChildrenRef<'program> = &'program #child_type;
                        type ChildrenRefMut<'program> = &'program mut #child_type;

                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> { &self.child_0 }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> { &mut self.child_0 }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildren<Type<'program>> for &'program #name
                    {
                        fn visit_each<V>(self, mut visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                        {
                            for (i, child) in ::fandango::typing::Node::children(self).iter().enumerate() {
                                visitor = match visitor.visit(#ref_prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse<V>(self, mut visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                        {
                            for (i, child) in ::fandango::typing::Node::children(self).iter().enumerate().rev() {
                                visitor = match visitor.visit(#ref_prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_from<V>(self, mut visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue=V>
                        {
                            for (i, child) in ::fandango::typing::Node::children(self).iter().skip(idx).enumerate().rev() {
                                visitor = match visitor.visit(#ref_prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_from<V>(self, mut visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue=V>
                        {
                            for (i, child) in ::fandango::typing::Node::children(self).iter().skip(idx).enumerate() {
                                visitor = match visitor.visit(#ref_prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_nth<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>>
                        {
                            if let Some(node) = ::fandango::typing::Node::children(self).iter().nth(idx) {
                                Ok(visitor.visit(#ref_prefix(node), idx))
                            } else {
                                Err(visitor)
                            }
                        }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildrenMut<TypeMut<'program>> for &'program mut #name
                    {
                        fn visit_each_mut<V>(self, mut visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                        {
                            for (i, child) in ::fandango::typing::Node::children_mut(self).iter_mut().enumerate() {
                                visitor = match visitor.visit_mut(#prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_mut<V>(self, mut visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                        {
                            for (i, child) in ::fandango::typing::Node::children_mut(self).iter_mut().enumerate().rev() {
                                visitor = match visitor.visit_mut(#prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_mut_from<V>(self, mut visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue=V>
                        {
                            for (i, child) in ::fandango::typing::Node::children_mut(self).iter_mut().skip(idx).enumerate().rev() {
                                visitor = match visitor.visit_mut(#prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_mut_from<V>(self, mut visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue=V>
                        {
                            for (i, child) in ::fandango::typing::Node::children_mut(self).iter_mut().skip(idx).enumerate() {
                                visitor = match visitor.visit_mut(#prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_nth_mut<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>>
                        {
                            if let Some(node) = ::fandango::typing::Node::children_mut(self).iter_mut().nth(idx) {
                                Ok(visitor.visit_mut(#prefix(node), idx))
                            } else {
                                Err(visitor)
                            }
                        }
                    }

                    impl<S, G> ::fandango::generation::DefaultGenerated<S, G> for #name
                    where
                        S: TypeSampler,
                        G: TypeGenerator<S>,
                    {
                        fn generate_default(sampler: &mut S, with: &mut G, depth: usize) -> Self {
                            Self {
                                child_0: #sampler,
                            }
                        }
                    }

                    #child_accessor

                    #from
                });
                if emit_parse_glue {
                    local_output.extend(quote! {
                        impl ::core::convert::TryFrom<::pest::iterators::Pair<'_, Rule>> for #name {
                            type Error = ParseError;

                            fn try_from(value: ::pest::iterators::Pair<'_, Rule>) -> Result<Self, Self::Error> {
                                debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                                let span = value.as_span();
                                let child_0 = value.into_inner().map(|value| {
                                    debug_assert_eq!(value.as_rule(), #(Rule::#pest_child_names),*);

                                    Ok(#(#child_types::try_from(value)?.into()),*)
                                }).collect::<Result<_, Self::Error>>()?;

                                #range_check_fail

                                Ok(Self {
                                    child_0,
                                })
                            }
                        }
                    });
                }
            }
            _ => {
                assert_ne!(children.len(), 0);
                let child_names = (0..children.len())
                    .map(|i| format_ident!("child_{i}"))
                    .collect::<Vec<_>>();
                let indices = (0..child_names.len()).collect::<Vec<_>>();

                let mut child_names_rev = child_names.clone();
                child_names_rev.reverse();
                let mut indices_rev = indices.clone();
                indices_rev.reverse();
                let mut ref_prefixes_rev = ref_visit_prefixes.clone();
                ref_prefixes_rev.reverse();
                let mut prefixes_rev = visit_prefixes.clone();
                prefixes_rev.reverse();

                let child_range = 0usize..;

                if matches!(node_weight, FandangoNode::Nonterminal(_)) {
                    for incoming in graph.edges_directed(*self, Direction::Incoming) {
                        if let FandangoNode::Production(prod) = graph[incoming.source()] {
                            span = prod.nonterminal().span();
                            last_nonterminal = Span::new(
                                span.get_input(),
                                prod.nonterminal().span().start(),
                                prod.alternative().span().end(),
                            )
                            .unwrap();
                        }
                    }
                }

                local_output.extend(quote! {
                    #derives
                    #[derive(Default)]
                    #[allow(missing_docs)]
                    pub struct #name {
                        #( #child_names: #child_field_types ),*
                    }

                    impl ::fandango::typing::Node for #name {
                        type Type<'program> = Type<'program>;
                        type TypeMut<'program> = TypeMut<'program>;
                        type ChildrenRef<'program> = ( #( &'program #child_types ),*, );
                        type ChildrenRefMut<'program> = ( #( &'program mut #child_types ),*, );

                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> { (#(#ref_visit_prefixes(&self.#child_names)),*,) }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> { (#(#visit_prefixes(&mut self.#child_names)),*,) }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildren<Type<'program>> for &'program #name
                    {
                        fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>,
                        {
                            #(
                            let visitor = match visitor.visit(#ref_visit_prefixes(&self.#child_names), #indices)? {
                                ::core::ops::ControlFlow::Continue(v) => v,
                                c => return Ok(c),
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                        {
                            #(
                            let visitor = match visitor.visit(#ref_prefixes_rev(&self.#child_names_rev), #indices_rev)? {
                                ::core::ops::ControlFlow::Continue(v) => v,
                                c => return Ok(c),
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue=V>
                        {
                            #(
                            let visitor = if #indices_rev <= idx {
                                match visitor.visit(#ref_prefixes_rev(&self.#child_names_rev), #indices_rev)? {
                                    ::core::ops::ControlFlow::Continue(v) => v,
                                    c => return Ok(c),
                                }
                            } else {
                                visitor
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>, Continue=V>
                        {
                            #(
                            let visitor = if idx <= #indices {
                                match visitor.visit(#ref_visit_prefixes(&self.#child_names), #indices)? {
                                    ::core::ops::ControlFlow::Continue(v) => v,
                                    c => return Ok(c),
                                }
                            } else {
                                visitor
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_nth<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::MaybeVisitResult<V, Type<'program>>
                        where
                            V: ::fandango::visitor::Visitor<Type<'program>>,
                        {
                            match idx {
                                #(#indices => Ok(visitor.visit(#ref_visit_prefixes(&self.#child_names), #indices))),*,
                                _ => Err(visitor)
                            }
                        }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildrenMut<TypeMut<'program>> for &'program mut #name
                    {
                        fn visit_each_mut<V>(self, visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>,
                        {
                            #(
                            let visitor = match visitor.visit_mut(#visit_prefixes(&mut self.#child_names), #indices)? {
                                ::core::ops::ControlFlow::Continue(v) => v,
                                c => return Ok(c),
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_mut<V>(self, visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                        {
                            #(
                            let visitor = match visitor.visit_mut(#prefixes_rev(&mut self.#child_names_rev), #indices_rev)? {
                                ::core::ops::ControlFlow::Continue(v) => v,
                                c => return Ok(c),
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_mut_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue=V>
                        {
                            #(
                            let visitor = if #indices_rev <= idx {
                                match visitor.visit_mut(#prefixes_rev(&mut self.#child_names_rev), #indices_rev)? {
                                    ::core::ops::ControlFlow::Continue(v) => v,
                                    c => return Ok(c),
                                }
                            } else {
                                visitor
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_mut_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue=V>
                        {
                            #(
                            let visitor = if idx <= #indices {
                                match visitor.visit_mut(#visit_prefixes(&mut self.#child_names), #indices)? {
                                    ::core::ops::ControlFlow::Continue(v) => v,
                                    c => return Ok(c),
                                }
                            } else {
                                visitor
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_nth_mut<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::MaybeVisitMutResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::VisitorMut<TypeMut<'program>>,
                        {
                            match idx {
                                #(#indices => Ok(visitor.visit_mut(#visit_prefixes(&mut self.#child_names), #indices))),*,
                                _ => Err(visitor)
                            }
                        }
                    }

                    impl<S, G> ::fandango::generation::DefaultGenerated<S, G> for #name
                    where
                        S: TypeSampler,
                        G: TypeGenerator<S>,
                    {
                        fn generate_default(sampler: &mut S, with: &mut G, depth: usize) -> Self {
                            Self {
                                #( #child_names: #generate_prefixes(::fandango::generation::Generated::generate(sampler, with, depth + 1)) ),*,
                            }
                        }
                    }

                    #(
                        impl ::fandango::typing::ChildAccessor<#child_range> for #name {
                            type Child<'a> = #child_ref_types;
                            type ChildMut<'a> = #child_mut_types;

                            fn child(&self) -> Self::Child<'_> {
                                #ref_visit_prefixes(&self.#child_names)
                            }

                            fn child_mut(&mut self) -> Self::ChildMut<'_> {
                                #visit_prefixes(&mut self.#child_names)
                            }
                        }
                    )*

                    #from
                });
                if emit_parse_glue {
                    let (underscore, eoi) = if starting_symbol {
                        (quote! { _ }, quote! { Rule::EOI })
                    } else {
                        (quote! {}, quote! {})
                    };
                    local_output.extend(quote! {
                        impl ::core::convert::TryFrom<::pest::iterators::Pair<'_, Rule>> for #name {
                            type Error = ParseError;

                            fn try_from(value: ::pest::iterators::Pair<'_, Rule>) -> Result<Self, Self::Error> {
                                debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                                let span = value.as_span();
                                let (#(#child_names),*,#underscore) = ::fandango::parse_pairs_as!(value.into_inner(), (#(#pest_child_names),*,#eoi));

                                Ok(Self {
                                    #(#child_names: #child_types::try_from(#child_names)?.into()),*,
                                })
                            }
                        }
                    });
                }
            }
        }

        // underlines a segment of the grammar to show where a definition comes from
        let start_offset =
            span.as_str().as_ptr() as usize - last_nonterminal.as_str().as_ptr() as usize;
        let end_offset = start_offset + span.as_str().as_bytes().len();
        let mut lines = Vec::new();
        for line in last_nonterminal.as_str().lines() {
            let line_start_offset =
                line.as_ptr() as usize - last_nonterminal.as_str().as_ptr() as usize;
            let line_end_offset = line_start_offset + line.as_bytes().len();
            if end_offset <= line_start_offset || start_offset >= line_end_offset {
                lines.push(format!("`{}`", line.replace("`", "\\`")));
                continue;
            }
            let start = line_start_offset.max(start_offset) - line_start_offset;
            let end = line_end_offset.min(end_offset) - line_start_offset;

            let mut composed = String::new();
            if start != 0 {
                composed.extend(format!("`{}`", line[..start].replace("`", "\\`")).chars());
            }
            composed.extend(format!("<u>`{}`</u>", line[start..end].replace("`", "\\`")).chars());
            if end != line.as_bytes().len() {
                composed.extend(format!("`{}`", line[end..].replace("`", "\\`")).chars());
            }
            lines.push(composed);
        }
        let documentation = lines.join("\n");
        output.extend(quote! {
            #[doc = concat!("This type is derived from the following grammar segment:\n\n", #documentation)]
        });
        output.extend(local_output);

        for (((_, child, child_weight, span), name), pest_name) in
            children.into_iter().zip(child_types).zip(pest_child_names)
        {
            child.emit_rust(
                (
                    name,
                    pest_name,
                    child_weight,
                    span,
                    last_nonterminal,
                    mapped_names,
                    graph,
                    needs_indirection,
                    emit_parse_glue,
                    derives,
                    shortest_paths,
                    false,
                ),
                output,
            )?;
        }

        Ok(())
    }
}
