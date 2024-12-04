use fandango_core::graph::FandangoNode;
use fandango_core::lang::Operator;
use pest::Span;
use petgraph::graphmap::DiGraphMap;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::convert::Infallible;

/// Produces a Rust source tree using the provided context.
pub trait IntoRustSource<C> {
    /// The error type which is encountered as a result of trying to emit the source code.
    type OutputError;

    /// Emits the corresponding Rust code for this structure.
    fn emit_rust(&self, ctx: &mut C, output: &mut TokenStream) -> Result<(), Self::OutputError>;
}

impl<'source> IntoRustSource<DiGraphMap<Self, Span<'source>>> for FandangoNode<'_, 'source> {
    type OutputError = Infallible;

    fn emit_rust(
        &self,
        graph: &mut DiGraphMap<Self, Span<'source>>,
        output: &mut TokenStream,
    ) -> Result<(), Self::OutputError> {
        let FandangoNode::Nonterminal(nt) = self else {
            unimplemented!("Can only transforms non-terminals into source code.")
        };

        let mut edges = graph.edges(*self);
        let (_, child, &weight) = edges
            .next()
            .expect("Nonterminals should have exactly one definition.");
        assert!(
            edges.next().is_none(),
            "Nonterminals should have exactly one definition."
        );

        let input = weight.get_input();
        output.extend(quote! {
            const SOURCE: &'static str = #input;
        });

        for production in graph
            .nodes()
            .filter(|n| matches!(n, FandangoNode::Production(_)))
        {
            let mut edges = graph.edges(production);
            let (_, child, &weight) = edges
                .next()
                .expect("Productions should have exactly one definition.");
            assert!(
                edges.next().is_none(),
                "Productions should have exactly one definition."
            );
            let FandangoNode::Nonterminal(nt) = child else {
                unreachable!("Production children should be strictly nonterminals.");
            };
            let name = format_ident!("nonterminal_{}", nt.name());
            let start = weight.start();
            let end = weight.end();
            output.extend(quote! {
                impl<'source> ::fandango::typing::NonterminalProduction<'source> for #name<'source> {
                    const DEF_SPAN: (usize, usize) = (#start, #end);
                }
            });
        }

        let name = format_ident!("nonterminal_{}", nt.name());
        let mut visited = HashSet::new();

        let pest_child_name = if let FandangoNode::Nonterminal(nt) = child {
            format_ident!("{}", nt.name())
        } else {
            format_ident!("{name}_0")
        };
        let child_name = if let FandangoNode::Nonterminal(nt) = child {
            format_ident!("nonterminal_{}", nt.name())
        } else {
            format_ident!("{name}_0")
        };
        let child_type = match child {
            FandangoNode::Nonterminal(_) => {
                quote! { ::std::boxed::Box<#child_name<'source>> }
            }
            _ => quote! { #child_name<'source> },
        };

        // TODO implement pest parsing

        let start = weight.start();
        let end = weight.end();
        output.extend(quote! {
            pub struct #name<'source> {
                span: ::core::option::Option<::fandango::Span<'source>>,
                child_0: #child_type,
            }

            impl<'source> ::fandango::typing::Node<'source> for #name<'source> {
                const SOURCE: &'static str = SOURCE;

                fn span(&self) -> ::core::option::Option<::fandango::Span<'source>> { self.span }
            }

            impl<'source> ::fandango::typing::Children<'source> for #name<'source> {
                type ChildrenRef<'program> = (&'program #child_name<'source>,) where 'source: 'program;
                type ChildrenRefMut<'program> = (&'program mut #child_name<'source>,) where 'source: 'program;

                const DEF_SPANS: &'static [(usize, usize)] = &[(#start, #end)];

                fn children(&self) -> Self::ChildrenRef<'_> {{ (&self.child_0,) }}
                fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {{ (&mut self.child_0,) }}
            }
        });

        visited.insert(*self);
        child.emit_rust(
            &mut (child_name, pest_child_name, weight, visited, graph),
            output,
        )
    }
}

type FandangoGenContext<'graph, 'program, 'source> = (
    Ident,
    Ident,
    Span<'source>,
    HashSet<FandangoNode<'program, 'source>>,
    &'graph mut DiGraphMap<FandangoNode<'program, 'source>, Span<'source>>,
);

impl<'graph, 'program, 'source> IntoRustSource<FandangoGenContext<'graph, 'program, 'source>>
    for FandangoNode<'program, 'source>
{
    type OutputError = Infallible;

    fn emit_rust(
        &self,
        ctx: &mut FandangoGenContext<'graph, 'program, 'source>,
        output: &mut TokenStream,
    ) -> Result<(), Self::OutputError> {
        let (name, _pest_child_name, weight, visited, graph) = ctx;
        if visited.contains(self) {
            return Ok(());
        }
        visited.insert(*self);

        let mut children = graph
            .edges(*self)
            .map(|(n1, n2, &w)| (n1, n2, w))
            .collect::<Vec<_>>();
        children.sort_by_key(|(_, _, w)| w.start());
        let pest_names = children
            .iter()
            .enumerate()
            .map(|(i, (_, child, _))| {
                if let FandangoNode::Nonterminal(nt) = child {
                    format_ident!("{}", nt.name())
                } else {
                    format_ident!("{name}_{i}")
                }
            })
            .collect::<Vec<_>>();
        let child_types = children
            .iter()
            .enumerate()
            .map(|(i, (_, child, _))| {
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
            .map(|((_, child, _), name)| {
                let base = quote! { #name<'source> };
                match self {
                    FandangoNode::Operator(op) => match op {
                        Operator::Kleene(_) | Operator::Plus(_) | Operator::Repeat(_, _) => {
                            quote! { ::std::vec::Vec<#base> }
                        }
                        Operator::Option(_) => match child {
                            FandangoNode::Nonterminal(_) => {
                                quote! { ::core::option::Option<::std::boxed::Box<#base>> }
                            }
                            _ => quote! { ::core::option::Option<#base> },
                        },
                        Operator::Symbol(_) => {
                            unimplemented!("Unexpected symbol; should be elided.")
                        }
                    },
                    _ => match child {
                        FandangoNode::Nonterminal(_) => {
                            quote! { ::std::boxed::Box<#base> }
                        }
                        _ => base,
                    },
                }
            })
            .collect::<Vec<_>>();

        let start = weight.start();
        let end = weight.end();

        match self {
            FandangoNode::String(s) => {
                output.extend(quote! {
                    pub struct #name<'source> {
                        span: ::core::option::Option<::fandango::Span<'source>>
                    }

                    impl<'source> ::fandango::typing::Node<'source> for #name<'source> {
                        const SOURCE: &'static str = SOURCE;

                        fn span(&self) -> ::core::option::Option<::fandango::Span<'source>> { self.span }
                    }

                    impl<'source> ::fandango::typing::Children<'source> for #name<'source> {
                        type ChildrenRef<'program> = (&'static str,) where 'source: 'program;
                        type ChildrenRefMut<'program> = (&'static str,) where 'source: 'program;

                        const DEF_SPANS: &'static [(usize, usize)] = &[(#start, #end)];

                        fn children(&self) -> Self::ChildrenRef<'_> {{ (&#s,) }}
                        fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {{ (&#s,) }}
                    }
                });
            }
            FandangoNode::Alternative(_) => {
                let child_variants = (0..children.len())
                    .map(|i| format_ident!("alt_child_{i}"))
                    .collect::<Vec<_>>();
                output.extend(quote! {
                    pub enum #name<'source> {
                        #( #child_variants ( #child_field_types ) ),*
                    }
                });
            }
            _ => {
                let child_names = (0..children.len())
                    .map(|i| format_ident!("child_{i}"))
                    .collect::<Vec<_>>();
                let child_spans = children
                    .iter()
                    .map(|(_, _, span)| {
                        let start = span.start();
                        let end = span.end();
                        quote! { (#start, #end) }
                    })
                    .collect::<Vec<_>>();
                output.extend(quote! {
                    pub struct #name<'source> {
                        span: ::core::option::Option<::fandango::Span<'source>>,
                        #( #child_names: #child_field_types ),*
                    }

                    impl<'source> ::fandango::typing::Node<'source> for #name<'source> {
                        const SOURCE: &'static str = SOURCE;

                        fn span(&self) -> ::core::option::Option<::fandango::Span<'source>> { self.span }
                    }

                    impl<'source> ::fandango::typing::Children<'source> for #name<'source> {
                        type ChildrenRef<'program> = ( #( &'program #child_field_types ),* ) where 'source: 'program;
                        type ChildrenRefMut<'program> = ( #( &'program mut #child_field_types ),* ) where 'source: 'program;

                        const DEF_SPANS: &'static [(usize, usize)] = &[#(#child_spans),*];

                        fn children(&self) -> Self::ChildrenRef<'_> { (#(&self.#child_names),*) }
                        fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> { (#(&mut self.#child_names),*) }
                    }
                });
            }
        }
        for (((_, child, i), name), pest_name) in
            children.into_iter().zip(child_types).zip(pest_names)
        {
            ctx.0 = name;
            ctx.1 = pest_name;
            ctx.2 = i;
            child.emit_rust(ctx, output)?;
        }

        Ok(())
    }
}
