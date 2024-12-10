use fandango_core::graph::FandangoNode;
use fandango_core::lang::{Operator, Statement, Symbol, Tagged, compute_tag_hash};
use pest::Span;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;

pub(crate) fn tokenize_metadata<'p, 's>(
    node: &FandangoNode<'p, 's>,
    span: Span<'s>,
    accessor: TokenStream,
    named: &mut HashMap<FandangoNode<'p, 's>, Ident>,
    arrays: &mut Vec<TokenStream>,
    referenced: &mut Vec<TokenStream>,
) -> TokenStream {
    let name = named.remove(node);

    let (ftype, base) = match node {
        FandangoNode::Program(p) => {
            let count = arrays.len();
            let name = format_ident!("FANDANGO_ARRAY_{}", count);
            arrays.push(quote! {}); // recursion guard
            let children = p
                .statements()
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    tokenize_metadata(
                        &FandangoNode::Statement(c.inner()),
                        c.span(),
                        quote! {
                            ({
                                match #accessor.inner().statements() {
                                    ::std::borrow::Cow::Borrowed(inner) => &inner[#i],
                                    _ => unreachable!()
                                }
                            })
                        },
                        named,
                        arrays,
                        referenced,
                    )
                })
                .collect::<Vec<_>>();
            arrays[count] = quote! {
                const #name: &'static [::fandango::lang::Tagged<'static, ::fandango::lang::Statement<'static>>] = &[
                    #(#children),*
                ];
            };
            (
                quote! {
                    ::fandango::lang::Program<'static>
                },
                quote! {
                    ::fandango::lang::Program::known(#name)
                },
            )
        }
        FandangoNode::Statement(s) => (
            quote! {
                ::fandango::lang::Statement<'static>
            },
            match s {
                Statement::Production(c) => {
                    let inner = tokenize_metadata(
                        &FandangoNode::Production(c.inner()),
                        c.span(),
                        quote! {
                            ({
                                match #accessor.inner() {
                                    ::fandango::lang::Statement::Production(c) => c,
                                    _ => unreachable!(),
                                }
                            })
                        },
                        named,
                        arrays,
                        referenced,
                    );
                    quote! {
                        ::fandango::lang::Statement::Production(
                            #inner
                        )
                    }
                }
                Statement::Constraint | Statement::Python => unreachable!(),
            },
        ),
        FandangoNode::Production(p) => {
            let nonterminal = {
                let c = p.nonterminal();
                tokenize_metadata(
                    &FandangoNode::Nonterminal(c.inner()),
                    c.span(),
                    quote! {
                        #accessor.inner().nonterminal()
                    },
                    named,
                    arrays,
                    referenced,
                )
            };
            let alternative = {
                let c = p.alternative();
                tokenize_metadata(
                    &FandangoNode::Alternative(c.inner()),
                    c.span(),
                    quote! {
                        #accessor.inner().alternative()
                    },
                    named,
                    arrays,
                    referenced,
                )
            };
            (
                quote! {
                    ::fandango::lang::Production<'static>
                },
                quote! {
                    ::fandango::lang::Production::known(
                        #nonterminal, #alternative
                    )
                },
            )
        }
        FandangoNode::Nonterminal(nt) => {
            let name = nt.name();
            (
                quote! {
                    ::fandango::lang::Nonterminal<'static>
                },
                quote! {
                    ::fandango::lang::Nonterminal::new(#name)
                },
            )
        }
        FandangoNode::Alternative(alt) => {
            let count = arrays.len();
            let name = format_ident!("FANDANGO_ARRAY_{}", count);
            arrays.push(quote! {}); // recursion guard
            let children = alt
                .concatenations()
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    tokenize_metadata(
                        &FandangoNode::Concatenation(c.inner()),
                        c.span(),
                        quote! {
                            ({
                                match #accessor.inner().concatenations() {
                                    ::std::borrow::Cow::Borrowed(inner) => &inner[#i],
                                    _ => unreachable!()
                                }
                            })
                        },
                        named,
                        arrays,
                        referenced,
                    )
                })
                .collect::<Vec<_>>();
            arrays[count] = quote! {
                const #name: &'static [::fandango::lang::Tagged<'static, ::fandango::lang::Concatenation<'static>>] = &[
                    #(#children),*
                ];
            };
            (
                quote! {
                    ::fandango::lang::Alternative<'static>
                },
                quote! {
                    ::fandango::lang::Alternative::known(#name)
                },
            )
        }
        FandangoNode::Concatenation(concat) => {
            let count = arrays.len();
            let name = format_ident!("FANDANGO_ARRAY_{}", count);
            arrays.push(quote! {}); // recursion guard
            let children = concat
                .operators()
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    tokenize_metadata(
                        &FandangoNode::Operator(c.inner()),
                        c.span(),
                        quote! {
                            ({
                                match #accessor.inner().operators() {
                                    ::std::borrow::Cow::Borrowed(inner) => &inner[#i],
                                    _ => unreachable!()
                                }
                            })
                        },
                        named,
                        arrays,
                        referenced,
                    )
                })
                .collect::<Vec<_>>();
            arrays[count] = quote! {
                const #name: &'static [::fandango::lang::Tagged<'static, ::fandango::lang::Operator<'static>>] = &[
                    #(#children),*
                ];
            };
            (
                quote! {
                    ::fandango::lang::Concatenation<'static>
                },
                quote! {
                    ::fandango::lang::Concatenation::known(#name)
                },
            )
        }
        FandangoNode::Operator(o) => {
            let (node, span) = match o {
                Operator::Kleene(c)
                | Operator::Plus(c)
                | Operator::Option(c)
                | Operator::Repeat(c, _, _)
                | Operator::Symbol(c) => (FandangoNode::Symbol(c.inner()), c.span()),
            };

            (
                quote! {
                    ::fandango::lang::Operator<'static>
                },
                match o {
                    Operator::Repeat(_, start, end) => {
                        let child = tokenize_metadata(
                            &node,
                            span,
                            quote! {
                                ({
                                    match #accessor.inner() {
                                         ::fandango::lang::Operator::Repeat(c, _) => c,
                                        _ => unreachable!(),
                                    }
                                })
                            },
                            named,
                            arrays,
                            referenced,
                        );
                        quote! {
                            ::fandango::lang::Operator::Repeat(
                                #child,
                                #start,
                                #end
                            )
                        }
                    }
                    _ => {
                        let matcher = match o {
                            Operator::Kleene(_) => quote! { ::fandango::lang::Operator::Kleene },
                            Operator::Plus(_) => quote! { ::fandango::lang::Operator::Plus },
                            Operator::Option(_) => quote! { ::fandango::lang::Operator::Option },
                            Operator::Symbol(_) => quote! { ::fandango::lang::Operator::Symbol },
                            _ => unreachable!(),
                        };

                        let child = tokenize_metadata(
                            &node,
                            span,
                            quote! {
                                ({
                                    match #accessor.inner() {
                                         #matcher(c) => c,
                                        _ => unreachable!(),
                                    }
                                })
                            },
                            named,
                            arrays,
                            referenced,
                        );
                        quote! {
                            #matcher(
                                #child
                            )
                        }
                    }
                },
            )
        }
        FandangoNode::Symbol(s) => {
            let (node, span, matcher) = match s {
                Symbol::Alternative(c) => (
                    FandangoNode::Alternative(c.inner()),
                    c.span(),
                    quote! { ::fandango::lang::Symbol::Alternative },
                ),
                Symbol::Nonterminal(c) => (
                    FandangoNode::Nonterminal(c.inner()),
                    c.span(),
                    quote! { ::fandango::lang::Symbol::Nonterminal },
                ),
                Symbol::String(c) => (
                    FandangoNode::String(c),
                    c.span(),
                    quote! { ::fandango::lang::Symbol::String },
                ),
            };
            (
                quote! {
                    ::fandango::lang::Symbol<'static>
                },
                {
                    let child = tokenize_metadata(
                        &node,
                        span,
                        quote! {
                            ({
                                match #accessor.inner() {
                                     #matcher(c) => c,
                                    _ => unreachable!(),
                                }
                            })
                        },
                        named,
                        arrays,
                        referenced,
                    );
                    quote! {
                        #matcher(
                            #child
                        )
                    }
                },
            )
        }
        FandangoNode::String(s) => (
            quote! {
                ::std::borrow::Cow<'static, str>
            },
            {
                let s = s.inner();
                quote! {
                    ::std::borrow::Cow::Borrowed(#s)
                }
            },
        ),
    };

    if let Some(name) = name {
        referenced.push(quote! {
            impl ::fandango::typing::Structured for #name<'_> {
                type FandangoType = #ftype;
                const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = #accessor;
            }
        });
    };

    let start = span.start();
    let end = span.end();
    let hash = compute_tag_hash(&span);

    quote! {
        ::fandango::lang::Tagged::known(
            #base,
            SOURCE,
            #start,
            #end,
            #hash,
        )
    }
}
