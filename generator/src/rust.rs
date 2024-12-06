use fandango_core::graph::FandangoNode;
use fandango_core::lang::{Nonterminal, Operator};
use pest::Span;
use petgraph::graphmap::DiGraphMap;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::convert::Infallible;

/// Produces a Rust source tree using the provided context.
pub trait IntoRustSource<C> {
    /// The error type which is encountered as a result of trying to emit the source code.
    type OutputError;

    /// Emits the corresponding Rust code for this structure.
    fn emit_rust(&self, ctx: C, output: &mut TokenStream) -> Result<(), Self::OutputError>;
}

impl<'graph, 'program, 'source>
    IntoRustSource<&'graph mut HashMap<FandangoNode<'program, 'source>, Ident>>
    for DiGraphMap<FandangoNode<'program, 'source>, Span<'source>>
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
            .nodes()
            .find(|n| match n {
                FandangoNode::Nonterminal(nt) if nt.name() == "start" => true,
                _ => false,
            })
            .expect("No start node?");

        let mut edges = self.edges(start_node);
        let (_, child, &weight) = edges
            .next()
            .expect("Nonterminals should have exactly one definition.");
        assert!(
            edges.next().is_none(),
            "Nonterminals should have exactly one definition."
        );

        let FandangoNode::Nonterminal(nt) = start_node else {
            unimplemented!("Can only transforms non-terminals into source code.")
        };

        let input = weight.get_input();
        output.extend(quote! {
            const SOURCE: &'static str = #input;
            pub type ParseError = Box<::fandango::error::Error<Rule>>;
        });

        let pest_name = format_ident!("{}", nt.name());
        let name = format_ident!("nonterminal_{}", nt.name());

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

        output.extend(quote! {
            pub struct #name<'source> {
                span: ::std::option::Option<(::std::rc::Rc<::std::borrow::Cow<'source, str>>, usize, usize)>,
                child_0: #child_type,
            }

            impl<'source> ::fandango::typing::Node for #name<'source> {
                fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> { ::fandango::typing::maybe_owned_span(&self.span) }
            }

            impl<'source> ::fandango::typing::Children for #name<'source> {
                type ChildrenRef<'program> = (&'program #child_name<'source>,) where 'source: 'program;
                type ChildrenRefMut<'program> = (&'program mut #child_name<'source>,) where 'source: 'program;

                fn children(&self) -> Self::ChildrenRef<'_> {{ (&self.child_0,) }}
                fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {{ (&mut self.child_0,) }}
            }

            impl<'source> ::std::convert::TryFrom<(::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                type Error = ParseError;

                fn try_from((source, value): (::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
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

        mapped_names.insert(start_node, name);
        child.emit_rust(
            (child_name, pest_child_name, weight, mapped_names, self),
            output,
        )
    }
}

type FandangoGenContext<'names, 'graph, 'program, 'source> = (
    Ident,
    Ident,
    Span<'source>,
    &'names mut HashMap<FandangoNode<'program, 'source>, Ident>,
    &'graph DiGraphMap<FandangoNode<'program, 'source>, Span<'source>>,
);

impl<'program, 'source> IntoRustSource<FandangoGenContext<'_, '_, 'program, 'source>>
    for FandangoNode<'program, 'source>
{
    type OutputError = Infallible;

    fn emit_rust(
        &self,
        ctx: FandangoGenContext<'_, '_, 'program, 'source>,
        output: &mut TokenStream,
    ) -> Result<(), Self::OutputError> {
        let (name, pest_name, _, mapped_names, graph) = ctx;
        match mapped_names.entry(*self) {
            Entry::Occupied(_) => return Ok(()),
            Entry::Vacant(e) => {
                e.insert(name.clone());
            }
        }

        let mut children = graph
            .edges(*self)
            .map(|(n1, n2, &w)| (n1, n2, w))
            .collect::<Vec<_>>();
        children.sort_by_key(|(_, _, w)| w.start());
        let pest_child_names = children
            .iter()
            .enumerate()
            .map(|(i, (_, child, _))| {
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
                                quote! { ::std::option::Option<::std::boxed::Box<#base>> }
                            }
                            _ => quote! { ::std::option::Option<#base> },
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

        match self {
            FandangoNode::String(s) => {
                output.extend(quote! {
                    pub struct #name<'source> {
                        span: ::std::option::Option<(::std::rc::Rc<::std::borrow::Cow<'source, str>>, usize, usize)>,
                    }

                    impl<'source> ::fandango::typing::Node for #name<'source> {
                        fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> { ::fandango::typing::maybe_owned_span(&self.span) }
                    }

                    impl<'source> ::fandango::typing::Children for #name<'source> {
                        type ChildrenRef<'program> = (&'static str,) where 'source: 'program;
                        type ChildrenRefMut<'program> = (&'static str,) where 'source: 'program;

                        fn children(&self) -> Self::ChildrenRef<'_> {{ (&#s,) }}
                        fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {{ (&#s,) }}
                    }

                    impl<'source> ::std::convert::TryFrom<(::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                        type Error = ParseError;

                        fn try_from((source, value): (::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
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
                output.extend(quote! {
                    pub enum #name<'source> {
                        #( #child_variants ( #child_field_types ) ),*
                    }

                    impl<'source> ::std::convert::TryFrom<(::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                        type Error = ParseError;

                        fn try_from((source, value): (::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
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
                    Operator::Repeat(_, r) => {
                        let start = r.start();
                        let end = r.end();
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

                output.extend(quote! {
                    pub struct #name<'source> {
                        span: ::std::option::Option<(::std::rc::Rc<::std::borrow::Cow<'source, str>>, usize, usize)>,
                        child_0: #(#child_field_types)*
                    }

                    impl<'source> ::fandango::typing::Node for #name<'source> {
                        fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> { ::fandango::typing::maybe_owned_span(&self.span) }
                    }

                    impl<'source> ::fandango::typing::Children for #name<'source> {
                        type ChildrenRef<'program> = &'program #(#child_field_types),* where 'source: 'program;
                        type ChildrenRefMut<'program> = &'program mut #(#child_field_types),* where 'source: 'program;

                        fn children(&self) -> Self::ChildrenRef<'_> { &self.child_0 }
                        fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> { &mut self.child_0 }
                    }

                    impl<'source> ::std::convert::TryFrom<(::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                        type Error = ParseError;

                        fn try_from((source, value): (::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
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
                output.extend(quote! {
                    pub struct #name<'source> {
                        span: ::std::option::Option<(::std::rc::Rc<::std::borrow::Cow<'source, str>>, usize, usize)>,
                        #( #child_names: #child_field_types ),*
                    }

                    impl<'source> ::fandango::typing::Node for #name<'source> {
                        fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> { ::fandango::typing::maybe_owned_span(&self.span) }
                    }

                    impl<'source> ::fandango::typing::Children for #name<'source> {
                        type ChildrenRef<'program> = ( #( &'program #child_field_types ),*, ) where 'source: 'program;
                        type ChildrenRefMut<'program> = ( #( &'program mut #child_field_types ),*, ) where 'source: 'program;

                        fn children(&self) -> Self::ChildrenRef<'_> { (#(&self.#child_names),*,) }
                        fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> { (#(&mut self.#child_names),*,) }
                    }

                    impl<'source> ::std::convert::TryFrom<(::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)> for #name<'source> {
                        type Error = ParseError;

                        fn try_from((source, value): (::std::rc::Rc<::std::borrow::Cow<'source, str>>, ::fandango::iterators::Pair<'source, Rule>)) -> Result<Self, Self::Error> {
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
        for (((_, child, i), name), pest_name) in
            children.into_iter().zip(child_types).zip(pest_child_names)
        {
            child.emit_rust((name, pest_name, i, mapped_names, graph), output)?;
        }

        Ok(())
    }
}
