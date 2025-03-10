use fandango_core::lang::FandangoNode;
use fandango_core::lang::Operator;
use pest::Span;
use petgraph::graph;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
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
        impl<'program, 'source> ::core::convert::From<&'program #name<'source>> for Type<'program, 'source> where 'source: 'program {
            fn from(node: &'program #name<'source>) -> Type<'program, 'source> {
                Type::#name(node)
            }
        }

        impl<'program, 'source> ::core::convert::From<&'program mut #name<'source>> for Type<'program, 'source> where 'source: 'program {
            fn from(node: &'program mut #name<'source>) -> Type<'program, 'source> {
                Type::#name(node)
            }
        }

        impl<'program, 'source> ::core::convert::From<&'program mut #name<'source>> for TypeMut<'program, 'source> where 'source: 'program {
            fn from(node: &'program mut #name<'source>) -> TypeMut<'program, 'source> {
                TypeMut::#name(node)
            }
        }

        impl<'program, 'source> ::core::convert::From<&'program ::alloc::boxed::Box<#name<'source>>> for Type<'program, 'source> where 'source: 'program {
            fn from(node: &'program ::alloc::boxed::Box<#name<'source>>) -> Type<'program, 'source> {
                Type::#name(&**node)
            }
        }

        impl<'program, 'source> ::core::convert::From<&'program mut ::alloc::boxed::Box<#name<'source>>> for Type<'program, 'source> where 'source: 'program {
            fn from(node: &'program mut ::alloc::boxed::Box<#name<'source>>) -> Type<'program, 'source> {
                Type::#name(&**node)
            }
        }

        impl<'program, 'source> ::core::convert::From<&'program mut ::alloc::boxed::Box<#name<'source>>> for TypeMut<'program, 'source> where 'source: 'program {
            fn from(node: &'program mut ::alloc::boxed::Box<#name<'source>>) -> TypeMut<'program, 'source> {
                TypeMut::#name(&mut **node)
            }
        }
    }
}

impl<'graph, 'program, 'source>
    IntoRustSource<&'graph mut HashMap<FandangoNode<'program, 'source>, Ident>>
    for DiGraph<FandangoNode<'program, 'source>, Span<'source>>
where
    'program: 'graph,
{
    type OutputError = Infallible;

    fn emit_rust(
        &self,
        mapped_names: &'graph mut HashMap<FandangoNode<'program, 'source>, Ident>,
        output: &mut TokenStream,
    ) -> Result<(), Self::OutputError> {
        let start_node = self
            .node_indices()
            .find(|&n| matches!(self.node_weight(n).unwrap(), FandangoNode::Nonterminal(nt) if nt.name() == "start"))
            .expect("No start node?");

        let mut edges = self.edges(start_node);
        let e = edges
            .next()
            .expect("Nonterminals should have exactly one definition.");
        let child = e.target();
        let weight = *e.weight();
        assert!(
            edges.next().is_none(),
            "Nonterminals should have exactly one definition."
        );

        let node_weight = *self.node_weight(start_node).unwrap();
        let child_weight = *self.node_weight(child).unwrap();

        let FandangoNode::Nonterminal(nt) = node_weight else {
            unimplemented!("Can only transforms non-terminals into source code.")
        };

        let input = weight.get_input();
        output.extend(quote! {
            const SOURCE: &'static str = #input;
            pub type ParseError = ::alloc::boxed::Box<::fandango::error::Error<Rule>>;
        });

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
        let child_type = match child_weight {
            FandangoNode::Nonterminal(_) => {
                quote! { ::alloc::boxed::Box<#child_name<'source>> }
            }
            _ => quote! { #child_name<'source> },
        };

        let from = from_boilerplate(&name);

        output.extend(quote! {
            #[derive(Clone, Debug, Eq, PartialEq)]
            pub struct #name<'source> {
                span: ::core::option::Option<(::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, usize, usize)>,
                child_0: #child_type,
            }

            impl<'source> ::fandango::typing::Node for #name<'source> {
                type Type<'program> = Type<'program, 'source> where 'source: 'program;
                type TypeMut<'program> = TypeMut<'program, 'source> where 'source: 'program;
                type ChildrenRef<'program> = (&'program #child_name<'source>, ()) where 'source: 'program;
                type ChildrenRefMut<'program> = (&'program mut #child_name<'source>, ()) where 'source: 'program;

                fn span(&self) -> ::core::option::Option<::fandango::Span<'_>> { ::fandango::typing::maybe_owned_span(&self.span) }
                fn clear_span(&mut self) { self.span = None; }
                fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {{ (&self.child_0, ()) }}
                fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {{ (&mut self.child_0, ()) }}
            }

            impl<'program, 'source> ::fandango::visitor::VisitableChildren<TypeMut<'program, 'source>> for &'program mut #name<'source> where 'source: 'program
            {
                fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                {
                    visitor.visit(&mut self.child_0, 0)
                }

                fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                {
                    self.visit_each(visitor)
                }

                fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue=V>
                {
                    self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                }

                fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue=V>
                {
                    self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                }

                fn visit_nth<V>(
                    self,
                    visitor: V,
                    idx: usize,
                ) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program, 'source>>
                where
                    V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>>
                {
                    if idx == 0 {
                        Ok(visitor.visit(&mut self.child_0, 0))
                    } else {
                        Err(visitor)
                    }
                }
            }

            impl<'source, S, G> ::fandango::generation::DefaultGenerated<S, G> for #name<'source>
            where
                S: TypeSampler<'source>,
                G: TypeGenerator<'source, S>,
            {
                fn generate_default(sampler: &mut S, with: &mut G) -> Self {
                    Self {
                        child_0: ::fandango::generation::Generated::generate(sampler, with),
                        span: None,
                    }
                }
            }

            #from

            impl<'source> ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                type Error = ParseError;

                fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
                    debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                    let span = value.as_span();
                    let (inner,_) = ::fandango::parse_pairs_as!(value.into_inner(), (Rule::#pest_child_name,Rule::EOI));

                    Ok(Self {
                        child_0: #child_name::try_from((source.clone(), inner))?.into(),
                        span: Some((source, span.start(), span.end())),
                    })
                }
            }
        });

        mapped_names.insert(node_weight, name);
        child.emit_rust(
            (
                child_name,
                pest_child_name,
                child_weight,
                mapped_names,
                self,
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
        let (name, pest_name, node_weight, mapped_names, graph) = ctx;
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

        let child_field_types = children
            .iter()
            .zip(&child_types)
            .map(|((_, _, child, _), name)| {
                let base = quote! { #name<'source> };
                match node_weight {
                    FandangoNode::Operator(op) => match op {
                        Operator::Kleene(_) | Operator::Plus(_) | Operator::Repeat(_, _, _) => {
                            quote! { ::alloc::vec::Vec<#base> }
                        }
                        Operator::Option(_) => match child {
                            FandangoNode::Nonterminal(_) => {
                                quote! { ::core::option::Option<::alloc::boxed::Box<#base>> }
                            }
                            _ => quote! { ::core::option::Option<#base> },
                        },
                        Operator::Symbol(_) => {
                            unimplemented!("Unexpected symbol; should be elided.")
                        }
                    },
                    _ => match child {
                        FandangoNode::Nonterminal(_) => {
                            quote! { ::alloc::boxed::Box<#base> }
                        }
                        _ => base,
                    },
                }
            })
            .collect::<Vec<_>>();

        match node_weight {
            FandangoNode::String(s) => {
                let s = s.inner();
                output.extend(quote! {
                    #[derive(Clone, Debug, Eq, PartialEq)]
                    pub struct #name<'source> {
                        span: ::core::option::Option<(::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, usize, usize)>,
                    }

                    impl<'source> ::fandango::typing::Node for #name<'source> {
                        type Type<'program> = Type<'program, 'source> where 'source: 'program;
                        type TypeMut<'program> = TypeMut<'program, 'source> where 'source: 'program;
                        type ChildrenRef<'program> = (&'static str,) where 'source: 'program;
                        type ChildrenRefMut<'program> = (&'static str,) where 'source: 'program;

                        fn span(&self) -> ::core::option::Option<::fandango::Span<'_>> { ::fandango::typing::maybe_owned_span(&self.span) }
                        fn clear_span(&mut self) { self.span = None; }
                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> { (&#s,) }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> { (&#s,) }
                    }

                    impl<'program, 'source> ::fandango::visitor::VisitableChildren<TypeMut<'program, 'source>> for &'program mut #name<'source> where 'source: 'program
                    {
                        fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V> {
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                        {
                            self.visit_each(visitor)
                        }

                        fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                        {
                            self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                        }

                        fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                        {
                            self.visit_nth(visitor, idx).unwrap_or_else(|c| Ok(::core::ops::ControlFlow::Continue(c)))
                        }

                        fn visit_nth<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>> {
                            Err(visitor)
                        }
                    }

                    impl<'source, S, G> ::fandango::generation::DefaultGenerated<S, G> for #name<'source> {
                        fn generate_default(sampler: &mut S, with: &mut G) -> Self {
                            Self {
                                span: None,
                            }
                        }
                    }

                    #from

                    impl<'source> ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                        type Error = ParseError;

                        fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
                            let span = value.as_span();
                            debug_assert_eq!(span.as_str(), #s);

                            Ok(Self { span: Some((source, span.start(), span.end())), })
                        }
                    }
                });
            }
            FandangoNode::Alternative(_) => {
                let child_variants = (0..children.len())
                    .map(|i| format_ident!("variant_{i}"))
                    .collect::<Vec<_>>();
                let indices = (0..children.len()).collect::<Vec<_>>();
                let count = children.len();
                output.extend(quote! {
                    #[derive(Clone, Debug, Eq, PartialEq)]
                    pub enum #name<'source> {
                        #( #child_variants ( #child_field_types ) ),*
                    }

                    impl<'source> ::fandango::typing::Node for #name<'source> {
                        type Type<'program> = Type<'program, 'source> where 'source: 'program;
                        type TypeMut<'program> = TypeMut<'program, 'source> where 'source: 'program;
                        type ChildrenRef<'program> = &'program Self where 'source: 'program;
                        type ChildrenRefMut<'program> = &'program mut Self where 'source: 'program;

                        fn span(&self) -> ::core::option::Option<::fandango::Span<'_>> {
                            match self {
                                #( Self::#child_variants ( inner ) => inner.span() ),*
                            }
                        }

                        fn clear_span(&mut self) {
                            match self {
                                #( Self::#child_variants ( inner ) => inner.clear_span() ),*
                            }
                        }

                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
                            self
                        }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
                            self
                        }
                    }

                    impl<'program, 'source> ::fandango::visitor::VisitableChildren<TypeMut<'program, 'source>> for &'program mut #name<'source> where 'source: 'program
                    {
                        fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V> {
                            match self {
                                #(#name::#child_variants(n) => visitor.visit(n, #indices)),*
                            }
                        }

                        fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                        {
                            self.visit_each(visitor)
                        }

                        fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                        {
                            match self {
                                #(#name::#child_variants(n) if idx >= #indices => visitor.visit(n, idx)),*,
                                _ => Ok(::core::ops::ControlFlow::Continue(visitor))
                            }
                        }

                        fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                        {
                            match self {
                                #(#name::#child_variants(n) if idx <= #indices => visitor.visit(n, idx)),*,
                                _ => Ok(::core::ops::ControlFlow::Continue(visitor))
                            }
                        }

                        fn visit_nth<V>(
                            self,
                            visitor: V,
                            idx: usize,
                        ) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>> {
                            match self {
                                #(#name::#child_variants(n) if idx == #indices => Ok(visitor.visit(n, idx))),*,
                                _ => Err(visitor)
                            }
                        }
                    }

                    impl<'source, S, G> ::fandango::generation::DefaultGenerated<S, G> for #name<'source>
                    where
                        S: TypeSampler<'source>,
                        G: TypeGenerator<'source, S>,
                    {
                        fn generate_default(sampler: &mut S, with: &mut G) -> Self {
                            match <S as ::fandango::generation::Sampler<Self>>::sample_alternative(sampler, #count) {
                                #(#indices => Self::#child_variants(::fandango::generation::Generated::generate(sampler, with))),*,
                                _ => unreachable!()
                            }
                        }
                    }

                    #from

                    impl<'source> ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                        type Error = ParseError;

                        fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
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
            FandangoNode::Operator(op) => {
                assert_eq!(children.len(), 1);

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
                        quote! { Vec<#(#child_types<'source>),*> }
                    }
                    Operator::Option(_) => {
                        quote! { Option<#(#child_types<'source>),*> }
                    }
                    Operator::Symbol(_) => {
                        unimplemented!("Unexpected symbol; should be elided.")
                    }
                };
                let sampler = match op {
                    Operator::Kleene(_) => {
                        quote! {
                            (0..=<S as ::fandango::generation::Sampler<Self>>::sample_kleene(sampler)).map(|_| ::fandango::generation::Generated::generate(sampler, with)).collect()
                        }
                    }
                    Operator::Plus(_) => {
                        quote! {
                            (0..=<S as ::fandango::generation::Sampler<Self>>::sample_plus(sampler)).map(|_| ::fandango::generation::Generated::generate(sampler, with)).collect()
                        }
                    }
                    Operator::Option(_) => {
                        quote! {
                            <S as ::fandango::generation::Sampler<Self>>::sample_optional(sampler).then(|_| ::fandango::generation::Generated::generate(sampler, with))
                        }
                    }
                    Operator::Repeat(_, start, end) => {
                        quote! {
                            (0..=<S as ::fandango::generation::Sampler<Self>>::sample_repetition(sampler, #start, #end)).map(|_| ::fandango::generation::Generated::generate(sampler, with)).collect()
                        }
                    }
                    Operator::Symbol(_) => {
                        unimplemented!("Unexpected symbol; should be elided.")
                    }
                };

                output.extend(quote! {
                    #[derive(Clone, Debug, Eq, PartialEq)]
                    pub struct #name<'source> {
                        span: ::core::option::Option<(::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, usize, usize)>,
                        child_0: #(#child_field_types)*
                    }

                    impl<'source> ::fandango::typing::Node for #name<'source> {
                        type Type<'program> = Type<'program, 'source> where 'source: 'program;
                        type TypeMut<'program> = TypeMut<'program, 'source> where 'source: 'program;
                        type ChildrenRef<'program> = &'program #child_type where 'source: 'program;
                        type ChildrenRefMut<'program> = &'program mut #child_type where 'source: 'program;

                        fn span(&self) -> ::core::option::Option<::fandango::Span<'_>> { ::fandango::typing::maybe_owned_span(&self.span) }
                        fn clear_span(&mut self) { self.span = None; }
                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> { &self.child_0 }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> { &mut self.child_0 }
                    }

                    impl<'program, 'source> ::fandango::visitor::VisitableChildren<TypeMut<'program, 'source>> for &'program mut #name<'source> where 'source: 'program
                    {
                        fn visit_each<V>(self, mut visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                        {
                            for (i, child) in self.children_mut().iter_mut().enumerate() {
                                visitor = match visitor.visit(child, i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse<V>(self, mut visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                        {
                            for (i, child) in self.children_mut().iter_mut().enumerate().rev() {
                                visitor = match visitor.visit(child, i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_from<V>(self, mut visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue=V>
                        {
                            for (i, child) in self.children_mut().iter_mut().skip(idx).enumerate().rev() {
                                visitor = match visitor.visit(child, i)? {
                                    ::core::ops::ControlFlow::Continue(visitor) => visitor,
                                    c => return Ok(c),
                                }
                            }
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_from<V>(self, mut visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue=V>
                        {
                            for (i, child) in self.children_mut().iter_mut().skip(idx).enumerate() {
                                visitor = match visitor.visit(child, i)? {
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
                        ) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>>
                        {
                            if let Some(node) = self.children_mut().iter_mut().nth(idx) {
                                Ok(visitor.visit(node, idx))
                            } else {
                                Err(visitor)
                            }
                        }
                    }

                    impl<'source, S, G> ::fandango::generation::DefaultGenerated<S, G> for #name<'source>
                    where
                        S: TypeSampler<'source>,
                        G: TypeGenerator<'source, S>,
                    {
                        fn generate_default(sampler: &mut S, with: &mut G) -> Self {
                            Self {
                                child_0: #sampler,
                                span: None,
                            }
                        }
                    }

                    #from

                    impl<'source> ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                        type Error = ParseError;

                        fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
                            debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                            let span = value.as_span();
                            let child_0 = value.into_inner().map(|value| {
                                debug_assert_eq!(value.as_rule(), #(Rule::#pest_child_names),*);

                                Ok(#(#child_types::try_from((source.clone(), value))?.into()),*)
                            }).collect::<Result<_, Self::Error>>()?;

                            #range_check_fail

                            Ok(Self {
                                child_0,
                                span: Some((source, span.start(), span.end())),
                            })
                        }
                    }
                });
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

                output.extend(quote! {
                    #[derive(Clone, Debug, Eq, PartialEq)]
                    pub struct #name<'source> {
                        span: ::core::option::Option<(::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, usize, usize)>,
                        #( #child_names: #child_field_types ),*
                    }

                    impl<'source> ::fandango::typing::Node for #name<'source> {
                        type Type<'program> = Type<'program, 'source> where 'source: 'program;
                        type TypeMut<'program> = TypeMut<'program, 'source> where 'source: 'program;
                        type ChildrenRef<'program> = ( #( &'program #child_field_types ),*, ) where 'source: 'program;
                        type ChildrenRefMut<'program> = ( #( &'program mut #child_field_types ),*, ) where 'source: 'program;

                        fn span(&self) -> ::core::option::Option<::fandango::Span<'_>> { ::fandango::typing::maybe_owned_span(&self.span) }
                        fn clear_span(&mut self) { self.span = None; }
                        fn children<'program>(&'program self) -> Self::ChildrenRef<'program> { (#(&self.#child_names),*,) }
                        fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> { (#(&mut self.#child_names),*,) }
                    }

                    impl<'program, 'source> ::fandango::visitor::VisitableChildren<TypeMut<'program, 'source>> for &'program mut #name<'source> where 'source: 'program
                    {
                        fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>,
                        {
                            #(
                            let visitor = match visitor.visit(&mut self.#child_names, #indices)? {
                                ::core::ops::ControlFlow::Continue(v) => v,
                                c => return Ok(c),
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
                        {
                            #(
                            let visitor = match visitor.visit(&mut self.#child_names_rev, #indices_rev)? {
                                ::core::ops::ControlFlow::Continue(v) => v,
                                c => return Ok(c),
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue=V>
                        {
                            #(
                            let visitor = if #indices_rev <= idx {
                                match visitor.visit(&mut self.#child_names_rev, #indices_rev)? {
                                    ::core::ops::ControlFlow::Continue(v) => v,
                                    c => return Ok(c),
                                }
                            } else {
                                visitor
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue=V>
                        {
                            #(
                            let visitor = if idx <= #indices {
                                match visitor.visit(&mut self.#child_names, #indices)? {
                                    ::core::ops::ControlFlow::Continue(v) => v,
                                    c => return Ok(c),
                                }
                            } else {
                                visitor
                            };
                            )*
                            Ok(::core::ops::ControlFlow::Continue(visitor))
                        }

                        fn visit_nth<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program, 'source>>
                        where
                            V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>>,
                        {
                            match idx {
                                #(#indices => Ok(visitor.visit(&mut self.#child_names, #indices))),*,
                                _ => Err(visitor)
                            }
                        }
                    }

                    impl<'source, S, G> ::fandango::generation::DefaultGenerated<S, G> for #name<'source>
                    where
                        S: TypeSampler<'source>,
                        G: TypeGenerator<'source, S>,
                    {
                        fn generate_default(sampler: &mut S, with: &mut G) -> Self {
                            Self {
                                #( #child_names: ::fandango::generation::Generated::generate(sampler, with) ),*,
                                span: None,
                            }
                        }
                    }

                    #from

                    impl<'source> ::core::convert::TryFrom<(::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                        type Error = ParseError;

                        fn try_from((source, value): (::alloc::rc::Rc<::alloc::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
                            debug_assert_eq!(value.as_rule(), Rule::#pest_name);

                            let span = value.as_span();
                            let (#(#child_names),*,) = ::fandango::parse_pairs_as!(value.into_inner(), (#(#pest_child_names),*,));

                            Ok(Self {
                                #(#child_names: #child_types::try_from((source.clone(), #child_names))?.into()),*,
                                span: Some((source, span.start(), span.end())),
                            })
                        }
                    }
                });
            }
        }
        for (((_, child, child_weight, _), name), pest_name) in
            children.into_iter().zip(child_types).zip(pest_child_names)
        {
            child.emit_rust((name, pest_name, child_weight, mapped_names, graph), output)?;
        }

        Ok(())
    }
}
