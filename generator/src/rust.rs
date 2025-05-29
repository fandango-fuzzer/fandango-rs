use fandango_core::lang::FandangoNode;
use fandango_core::lang::Operator;
use pest::Span;
use petgraph::graph::DiGraph;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::{Direction, algo, graph};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
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
    )> for DiGraph<FandangoNode<'program, 'source>, Span<'source>>
where
    'program: 'graph,
{
    type OutputError = Infallible;

    fn emit_rust(
        &self,
        (mapped_names, emit_parse_glue): (
            &'graph mut HashMap<FandangoNode<'program, 'source>, Ident>,
            bool,
        ),
        output: &mut TokenStream,
    ) -> Result<(), Self::OutputError> {
        let start_node = self
            .node_indices()
            .find(|&n| matches!(self.node_weight(n).unwrap(), FandangoNode::Nonterminal(nt) if nt.name() == "start"))
            .expect("No start node?");

        let needs_indirection = if cfg!(no_opt_indirect) {
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
        } else {
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
        };

        let mut edges = self.edges(start_node);
        let e = edges
            .next()
            .expect("Nonterminals should have exactly one definition.");
        let child = e.target();
        assert!(
            edges.next().is_none(),
            "Nonterminals should have exactly one definition."
        );

        let node_weight = *self.node_weight(start_node).unwrap();
        let child_weight = *self.node_weight(child).unwrap();

        let FandangoNode::Nonterminal(nt) = node_weight else {
            unimplemented!("Can only transforms non-terminals into source code.")
        };

        let pest_name = format_ident!("{}", nt.name());
        let name = format_ident!("nonterminal_{}", nt.name());

        let pest_child_name = if let FandangoNode::Nonterminal(nt) = child_weight {
            format_ident!("{}", nt.name())
        } else {
            format_ident!("{name}_0")
        };
        let child_name = if let FandangoNode::Nonterminal(nt) = child_weight {
            format_ident!("nonterminal_{}", nt.name())
        } else {
            format_ident!("{name}_0")
        };
        let (child_type, prefix) = if needs_indirection.contains(&(node_weight, child_weight)) {
            (
                quote! { ::alloc::boxed::Box<#child_name> },
                quote! { ::core::ops::DerefMut::deref_mut },
            )
        } else {
            (quote! { #child_name }, quote! {})
        };

        let from = from_boilerplate(&name);

        output.extend(quote! {
            #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
            #[allow(missing_docs)]
            pub struct #name {
                pub child_0: #child_type,
            }

            impl ::fandango::typing::Node for #name {
                type Type<'program> = Type<'program>;
                type TypeMut<'program> = TypeMut<'program>;
                type ChildrenRef<'program> = (&'program #child_name, ());
                type ChildrenRefMut<'program> = (&'program mut #child_name, ());

                fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {{ (&self.child_0, ()) }}
                fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {{ (&mut self.child_0, ()) }}
            }

            impl<'program> ::fandango::visitor::VisitableChildren<TypeMut<'program>> for &'program mut #name
            {
                fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                {
                    visitor.visit(#prefix(&mut self.child_0), 0)
                }

                fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                {
                    self.visit_each(visitor)
                }

                fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue=V>
                {
                    self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                }

                fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue=V>
                {
                    self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                }

                fn visit_nth<V>(
                    self,
                    visitor: V,
                    idx: usize,
                ) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program>>
                {
                    if idx == 0 {
                        Ok(visitor.visit(#prefix(&mut self.child_0), 0))
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
                        child_0: ::fandango::generation::Generated::generate(sampler, with, depth + 1),
                    }
                }
            }

            #from
        });
        if emit_parse_glue {
            output.extend(quote! {
                pub type ParseError = ::alloc::boxed::Box<::fandango::error::Error<Rule>>;

                impl ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)> for #name {
                    type Error = ParseError;

                    fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)) -> Result<Self, Self::Error> {
                        debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                        let span = value.as_span();
                        let (inner,_) = ::fandango::parse_pairs_as!(value.into_inner(), (Rule::#pest_child_name,Rule::EOI));

                        Ok(Self {
                            child_0: #child_name::try_from((source.clone(), inner))?.into(),
                        })
                    }
                }
            });
        }

        mapped_names.insert(node_weight, name);
        child.emit_rust(
            (
                child_name,
                pest_child_name,
                child_weight,
                mapped_names,
                self,
                &needs_indirection,
                emit_parse_glue,
            ),
            output,
        )
    }
}

type FandangoGenContext<'names, 'graph, 'program, 'source> = (
    Ident,
    Ident,
    FandangoNode<'program, 'source>,
    &'names mut HashMap<FandangoNode<'program, 'source>, Ident>,
    &'graph DiGraph<FandangoNode<'program, 'source>, Span<'source>>,
    &'graph HashSet<(
        FandangoNode<'program, 'source>,
        FandangoNode<'program, 'source>,
    )>,
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
        let (name, pest_name, node_weight, mapped_names, graph, needs_indirection, emit_parse_glue) =
            ctx;
        match mapped_names.entry(node_weight) {
            Entry::Occupied(_) => return Ok(()),
            Entry::Vacant(e) => {
                e.insert(name.clone());
            }
        }

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

        let (child_field_types, visit_prefixes) = children
            .iter()
            .zip(&child_types)
            .map(|((_, _, child, _), name)| {
                let base = quote! { #name };
                match node_weight {
                    FandangoNode::Operator(op) => match op {
                        Operator::Kleene(_) | Operator::Plus(_) | Operator::Repeat(_, _, _) => {
                            (quote! { ::alloc::vec::Vec<#base> }, quote! {})
                        }
                        Operator::Option(_) => {
                            if needs_indirection.contains(&(node_weight, *child)) {
                                (
                                    quote! { ::core::option::Option<::alloc::boxed::Box<#base>> },
                                    quote! { ::core::ops::DerefMut::deref_mut },
                                )
                            } else {
                                (quote! { ::core::option::Option<#base> }, quote! {})
                            }
                        }
                        Operator::Symbol(_) => {
                            unimplemented!("Unexpected symbol; should be elided.")
                        }
                    },
                    _ => {
                        if needs_indirection.contains(&(node_weight, *child)) {
                            (
                                quote! { ::alloc::boxed::Box<#base> },
                                quote! { ::core::ops::DerefMut::deref_mut },
                            )
                        } else {
                            (base, quote! {})
                        }
                    }
                }
            })
            .collect::<(Vec<_>, Vec<_>)>();

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
                output.extend(quote! {
                    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

                    impl<'program> ::fandango::visitor::VisitableChildren<TypeMut<'program>> for &'program mut #name
                    {
                        fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V> {
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                        {
                            self.visit_each(visitor)
                        }

                        fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                        {
                            self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                        }

                        fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                        {
                            self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                        }

                        fn visit_nth<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>> {
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
                    output.extend(quote! {
                        impl ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)> for #name {
                            type Error = ParseError;

                            fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)) -> Result<Self, Self::Error> {
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
                output.extend(quote! {
                    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
                    #[allow(missing_docs)]
                    pub enum #name {
                        #( #child_variants ( #child_field_types ) ),*
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

                    impl<'program> ::fandango::visitor::VisitableChildren<TypeMut<'program>> for &'program mut #name
                    {
                        fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V> {
                            match self {
                                #(#name::#child_variants(n) => visitor.visit(#visit_prefixes(n), #indices)),*
                            }
                        }

                        fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                        {
                            self.visit_each(visitor)
                        }

                        fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                        {
                            match self {
                                #(#name::#child_variants(n) if idx >= #indices => visitor.visit(#visit_prefixes(n), idx)),*,
                                _ => Ok(::core::ops::ControlFlow::Continue(visitor))
                            }
                        }

                        fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                        {
                            match self {
                                #(#name::#child_variants(n) if idx <= #indices => visitor.visit(#visit_prefixes(n), idx)),*,
                                _ => Ok(::core::ops::ControlFlow::Continue(visitor))
                            }
                        }

                        fn visit_nth<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>> {
                            match self {
                                #(#name::#child_variants(n) if idx == #indices => Ok(visitor.visit(#visit_prefixes(n), idx))),*,
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
                                #(#indices => Self::#child_variants(::fandango::generation::Generated::generate(sampler, with, depth + 1))),*,
                                _ => unreachable!()
                            }
                        }
                    }

                    #from
                });
                if emit_parse_glue {
                    output.extend(quote! {
                        impl ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)> for #name {
                            type Error = ParseError;

                            fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)) -> Result<Self, Self::Error> {
                                debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                                let mut children = value.into_inner();
                                let child_0 = children.next().expect("Expected exactly one descendent.");
                                debug_assert!(children.next().is_none(), "Expected exactly one descendent.");

                                Ok(match child_0.as_rule() {
                                    #(Rule::#pest_child_names => #name::#child_variants(
                                        #child_types::try_from((source, child_0))?.into()
                                    )),*,
                                    _ => unimplemented!("Not a child of this alternative.")
                                })
                            }
                        }
                    });
                }
            }
            FandangoNode::Operator(op) => {
                assert_eq!(children.len(), 1);
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
                        quote! { Vec<#(#child_types),*> }
                    }
                    Operator::Option(_) => {
                        quote! { Option<#(#child_types),*> }
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
                            <S as ::fandango::generation::Sampler<Self>>::sample_optional(sampler).then(|| ::fandango::generation::Generated::generate(sampler, with, depth + 1))
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

                output.extend(quote! {
                    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
                    #[allow(missing_docs)]
                    pub struct #name {
                        pub child_0: #(#child_field_types)*
                    }

                    impl ::fandango::typing::Node for #name {
                        type Type<'program> = Type<'program>;
                        type TypeMut<'program> = TypeMut<'program>;
                        type ChildrenRef<'program> = &'program #child_type;
                        type ChildrenRefMut<'program> = &'program mut #child_type;

                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> { &self.child_0 }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> { &mut self.child_0 }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildren<TypeMut<'program>> for &'program mut #name
                    {
                        fn visit_each<V>(self, mut visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                        {
                            for (i, child) in self.children_mut().iter_mut().enumerate() {
                                visitor = match visitor.visit(#prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse<V>(self, mut visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                        {
                            for (i, child) in self.children_mut().iter_mut().enumerate().rev() {
                                visitor = match visitor.visit(#prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_from<V>(self, mut visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue=V>
                        {
                            for (i, child) in self.children_mut().iter_mut().skip(idx).enumerate().rev() {
                                visitor = match visitor.visit(#prefix(child), i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_from<V>(self, mut visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue=V>
                        {
                            for (i, child) in self.children_mut().iter_mut().skip(idx).enumerate() {
                                visitor = match visitor.visit(#prefix(child), i)? {
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
                        ) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>>
                        {
                            if let Some(node) = self.children_mut().iter_mut().nth(idx) {
                                Ok(visitor.visit(#prefix(node), idx))
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

                    #from
                });
                if emit_parse_glue {
                    output.extend(quote! {
                        impl ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)> for #name {
                            type Error = ParseError;

                            fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)) -> Result<Self, Self::Error> {
                                debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                                let span = value.as_span();
                                let child_0 = value.into_inner().map(|value| {
                                    debug_assert_eq!(value.as_rule(), #(Rule::#pest_child_names),*);

                                    Ok(#(#child_types::try_from((source.clone(), value))?.into()),*)
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
                let mut prefixes_rev = visit_prefixes.clone();
                prefixes_rev.reverse();

                output.extend(quote! {
                    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
                    #[allow(missing_docs)]
                    pub struct #name {
                        #( pub #child_names: #child_field_types ),*
                    }

                    impl ::fandango::typing::Node for #name {
                        type Type<'program> = Type<'program>;
                        type TypeMut<'program> = TypeMut<'program>;
                        type ChildrenRef<'program> = ( #( &'program #child_field_types ),*, );
                        type ChildrenRefMut<'program> = ( #( &'program mut #child_field_types ),*, );

                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> { (#(&self.#child_names),*,) }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> { (#(&mut self.#child_names),*,) }
                    }

                    impl<'program> ::fandango::visitor::VisitableChildren<TypeMut<'program>> for &'program mut #name
                    {
                        fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>,
                        {
                            #(
                            let visitor = match visitor.visit(#visit_prefixes(&mut self.#child_names), #indices)? {
                                ::core::ops::ControlFlow::Continue(v) => v,
                                c => return Ok(c),
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue = V>
                        {
                            #(
                            let visitor = match visitor.visit(#prefixes_rev(&mut self.#child_names_rev), #indices_rev)? {
                                ::core::ops::ControlFlow::Continue(v) => v,
                                c => return Ok(c),
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue=V>
                        {
                            #(
                            let visitor = if #indices_rev <= idx {
                                match visitor.visit(#prefixes_rev(&mut self.#child_names_rev), #indices_rev)? {
                                    ::core::ops::ControlFlow::Continue(v) => v,
                                    c => return Ok(c),
                                }
                            } else {
                                visitor
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>, Continue=V>
                        {
                            #(
                            let visitor = if idx <= #indices {
                                match visitor.visit(#visit_prefixes(&mut self.#child_names), #indices)? {
                                    ::core::ops::ControlFlow::Continue(v) => v,
                                    c => return Ok(c),
                                }
                            } else {
                                visitor
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_nth<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program>>,
                        {
                            match idx {
                                #(#indices => Ok(visitor.visit(#visit_prefixes(&mut self.#child_names), #indices))),*,
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
                                #( #child_names: ::fandango::generation::Generated::generate(sampler, with, depth + 1) ),*,
                            }
                        }
                    }

                    #from
                });
                if emit_parse_glue {
                    output.extend(quote! {
                        impl ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)> for #name {
                            type Error = ParseError;

                            fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'_, str>>, ::fandango::iterators::Pair<'_, Rule>)) -> Result<Self, Self::Error> {
                                debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                                let span = value.as_span();
                                let (#(#child_names),*,) = ::fandango::parse_pairs_as!(value.into_inner(), (#(#pest_child_names),*,));

                                Ok(Self {
                                    #(#child_names: #child_types::try_from((source.clone(), #child_names))?.into()),*,
                                })
                            }
                        }
                    });
                }
            }
        }
        for (((_, child, child_weight, _), name), pest_name) in
            children.into_iter().zip(child_types).zip(pest_child_names)
        {
            child.emit_rust(
                (
                    name,
                    pest_name,
                    child_weight,
                    mapped_names,
                    graph,
                    needs_indirection,
                    emit_parse_glue,
                ),
                output,
            )?;
        }

        Ok(())
    }
}
