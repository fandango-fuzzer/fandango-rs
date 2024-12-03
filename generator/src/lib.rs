//! Code generation routines for FANDANGO. Only usable if the crate `fandango` is a dependency,
//! and may not be renamed.

use fandango_core::graph::{FandangoNode, IntoGraph};
use fandango_core::lang::{Nonterminal, ParseError, Program, Span, Tagged};
use pest::error::{InputLocation, LineColLocation};
use petgraph::graphmap::DiGraphMap;
use quote::{format_ident, quote};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::convert::Infallible;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, ExprLit, Lit, Meta};

use proc_macro2::{Ident, TokenStream};

/// A `#[derive]`-style derivation of a FANDANGO grammar.
pub struct FandangoDerivation {
    ident: Ident,
    merged: String,
    offsets: BTreeMap<usize, PathBuf>,
}

impl FandangoDerivation {
    fn lookup(&self, offset: usize) -> Option<(&Path, usize)> {
        let (&lower, path) = self.offsets.range(0..offset).last()?;
        Some((path.as_path(), offset - lower))
    }

    fn line_lookup(&self, line: usize) -> Option<(&Path, usize)> {
        self.merged
            .lines()
            .take(line)
            .next()
            .map(|s| s.as_ptr() as usize - self.merged.as_ptr() as usize)
            .and_then(|offset| {
                let (&lower, path) = self.offsets.range(0..offset).last()?;
                Some((path.as_path(), self.merged[lower..offset].lines().count()))
            })
    }

    fn to_compile_error(&self, mut error: ParseError) -> TokenStream {
        let first_file;
        let mut second_file = None;
        match error.location {
            InputLocation::Pos(offset) => {
                let (file, offset) = self.lookup(offset).unwrap();
                first_file = file;
                InputLocation::Pos(offset)
            }
            InputLocation::Span((start, end)) => {
                let (f1, o1) = self.lookup(start).unwrap();
                let (f2, o2) = self.lookup(end).unwrap();
                first_file = f1;
                second_file = Some(f2);
                InputLocation::Span((o1, o2))
            }
        };
        error.line_col = match error.line_col {
            LineColLocation::Pos((line, col)) => {
                assert!(second_file.is_none());
                let (file, line) = self.line_lookup(line).unwrap();
                assert_eq!(file, first_file);
                LineColLocation::Pos((line, col))
            }
            LineColLocation::Span((l1, c1), (l2, c2)) => {
                let (f1, l1) = self.line_lookup(l1).unwrap();
                assert_eq!(f1, first_file);
                let (f2, l2) = self.line_lookup(l2).unwrap();
                assert_eq!(Some(f2), second_file);
                LineColLocation::Span((l1, c1), (l2, c2))
            }
        };

        let composed_filename = match (first_file, second_file) {
            (f1, None) => f1.to_string_lossy(),
            (f1, Some(f2)) if f1 == f2 => f1.to_string_lossy(),
            (f1, Some(f2)) => Cow::Owned(format!(
                "(starting in {}, ending in {})",
                f1.to_string_lossy(),
                f2.to_string_lossy()
            )),
        };

        let error = error.with_path(&composed_filename);
        let stringified = error.to_string();

        quote! {
            compile_error!("Parse error: {}", #stringified)
        }
    }
}

impl Parse for FandangoDerivation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let derived: DeriveInput = input.parse()?;
        let ident = derived.ident;
        let mut merged = Vec::new();
        let mut offsets = BTreeMap::new();
        for attr in derived.attrs {
            let span = attr.span();
            if let Meta::NameValue(v) = attr.meta {
                if let Some(ident) = v.path.get_ident() {
                    if ident == "grammar" {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) = v.value
                        {
                            let path = PathBuf::from(s.value());
                            let mut file = File::open(&path).map_err(|e| {
                                syn::Error::new(
                                    span,
                                    format!("Failed to open {}: {}", path.to_string_lossy(), e),
                                )
                            })?;
                            let offset_before = merged.len();
                            file.read_to_end(&mut merged).map_err(|e| {
                                syn::Error::new(
                                    span,
                                    format!(
                                        "Error while reading {}: {}",
                                        path.to_string_lossy(),
                                        e
                                    ),
                                )
                            })?;
                            merged.push(b'\n'); // accounting for potential no endline
                            offsets.insert(offset_before, path);
                            continue;
                        }
                        return Err(syn::Error::new(span, "Invalid grammar source."));
                    }
                }
            }
        }
        let merged = String::from_utf8(merged).map_err(|e| {
            let invalid = e.utf8_error().valid_up_to() + 1;
            let (&lower, path) = offsets.range(0..invalid).last().unwrap();
            let invalid = invalid - lower;
            let path = path.to_string_lossy();
            syn::Error::new(
                proc_macro2::Span::mixed_site(),
                format!("Invalid UTF-8 byte at {}:{}.", path, invalid),
            )
        })?;
        Ok(Self {
            ident,
            merged,
            offsets,
        })
    }
}

/// Produces a Rust source tree using the provided context.
trait IntoRustSource<C> {
    /// The error type which is encountered as a result of trying to emit the source code.
    type OutputError;

    /// Emits the types for this structure.
    fn typeinfo(&self, ctx: &mut C, output: &mut TokenStream) -> Result<(), Self::OutputError>;
}

impl<'source> IntoRustSource<DiGraphMap<Self, Span<'source>>> for FandangoNode<'_, 'source> {
    type OutputError = Infallible;

    fn typeinfo(
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
            let name = format_ident!("{}", nt.name());
            let start = weight.start();
            let end = weight.end();
            output.extend(quote! {
                impl ::fandango::typing::NonterminalProduction for #name {
                    const DEF_SPAN: (usize, usize) = (#start, #end);
                }
            });
        }

        let name = format_ident!("{}", nt.name());
        let mut visited = HashSet::new();

        let child_name = if let FandangoNode::Nonterminal(nt) = child {
            format_ident!("{}", nt.name())
        } else {
            format_ident!("{name}_0")
        };

        let start = weight.start();
        let end = weight.end();
        output.extend(quote! {
            pub struct #name {
                child_0: #child_name,
            }

            impl ::fandango::typing::Children for #name {
                type ChildrenRef<'a> = (&'a #child_name,);
                type ChildrenRefMut<'a> = (&'a mut #child_name,);

                const SOURCE: &'static str = SOURCE;
                const DEF_SPANS: &'static [(usize, usize)] = &[(#start, #end)];

                fn children(&self) -> Self::ChildrenRef<'_> {{ (&self.child_0,) }}
                fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {{ (&mut self.child_0,) }}
            }
        });

        visited.insert(*self);
        child.typeinfo(&mut (child_name, *self, weight, visited, graph), output)
    }
}

type FandangoGenContext<'graph, 'program, 'source> = (
    Ident,
    FandangoNode<'program, 'source>,
    Span<'source>,
    HashSet<FandangoNode<'program, 'source>>,
    &'graph mut DiGraphMap<FandangoNode<'program, 'source>, Span<'source>>,
);

impl<'graph, 'program, 'source> IntoRustSource<FandangoGenContext<'graph, 'program, 'source>>
    for FandangoNode<'program, 'source>
{
    type OutputError = Infallible;

    fn typeinfo(
        &self,
        ctx: &mut FandangoGenContext<'graph, 'program, 'source>,
        output: &mut TokenStream,
    ) -> Result<(), Self::OutputError> {
        let (name, _parent, weight, visited, graph) = ctx;
        if visited.contains(self) {
            return Ok(());
        }
        visited.insert(*self);

        let mut children = graph
            .edges(*self)
            .map(|(n1, n2, &w)| (n1, n2, w))
            .collect::<Vec<_>>();
        children.sort_by_key(|(_, _, w)| w.start());
        let child_types = children
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
        let child_field_types = children
            .iter()
            .zip(&child_types)
            .map(|((_, child, _), name)| match child {
                FandangoNode::Nonterminal(_) => {
                    quote! { Box<#name> }
                }
                _ => {
                    quote! { #name }
                }
            })
            .collect::<Vec<_>>();

        let start = weight.start();
        let end = weight.end();

        match self {
            FandangoNode::String(s) => {
                output.extend(quote! {
                    pub struct #name;

                    impl ::fandango::typing::Children for #name {
                        type ChildrenRef<'a> = (&'static str,);
                        type ChildrenRefMut<'a> = (&'static str,);

                        const SOURCE: &'static str = SOURCE;
                        const DEF_SPANS: &'static [(usize, usize)] = &[(#start, #end)];

                        fn children(&self) -> Self::ChildrenRef<'_> {{ (&#s,) }}
                        fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {{ (&#s,) }}
                    }
                });
            }
            FandangoNode::Bytes(bytes) => {
                output.extend(quote! {
                    pub struct #name;

                    impl ::fandango::typing::Children for #name {
                        type ChildrenRef<'a> = (&'static [u8],);
                        type ChildrenRefMut<'a> = (&'static [u8],);

                        const SOURCE: &'static str = SOURCE;
                        const DEF_SPANS: &'static [(usize, usize)] = &[(#start, #end)];

                        fn children(&self) -> Self::ChildrenRef<'_> { (&[#(#bytes),*],) }
                        fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> { (&[#(#bytes),*],) }
                    }
                });
            }
            FandangoNode::Alternative(_) => {
                let child_variants = (0..children.len())
                    .map(|i| format_ident!("alt_child_{i}"))
                    .collect::<Vec<_>>();
                output.extend(quote! {
                    pub enum #name {
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
                    pub struct #name {
                        #( #child_names: #child_field_types ),*
                    }

                    impl ::fandango::typing::Children for #name {
                        type ChildrenRef<'a> = ( #( &'a #child_types ),* );
                        type ChildrenRefMut<'a> = ( #( &'a mut #child_types ),* );

                        const SOURCE: &'static str = SOURCE;
                        const DEF_SPANS: &'static [(usize, usize)] = &[#(#child_spans),*];

                        fn children(&self) -> Self::ChildrenRef<'_> { (#(&self.#child_names),*) }
                        fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> { (#(&mut self.#child_names),*) }
                    }
                });
            }
        }
        for ((_, child, i), name) in children.into_iter().zip(child_types) {
            ctx.0 = name;
            ctx.2 = i;
            child.typeinfo(ctx, output)?;
        }

        Ok(())
    }
}

/// Perform the derivation, or emit a compiler error with a (potentially useful) warning.
pub fn derive_fandango_or_emit_error(
    source: FandangoDerivation,
) -> Result<TokenStream, TokenStream> {
    let mod_name = format_ident!("parser_{}", source.ident);
    let inner_name = format_ident!("parser_{}_inner", source.ident);
    // let ident = &source.ident;

    let parsed = Tagged::<Program>::try_from(source.merged.as_str())
        .map_err(|e| source.to_compile_error(e))?;

    let mut graph = (&parsed).into_graph();
    let start = Nonterminal::new(Cow::Borrowed("start"));
    let start = FandangoNode::Nonterminal(&start);
    let mut tokenized = TokenStream::new();

    start.typeinfo(&mut graph, &mut tokenized).unwrap();

    Ok(quote! {
        mod #mod_name {
            #![allow(non_snake_case)]
            #![allow(non_camel_case_types)]

            mod #inner_name {
                #tokenized
            }
        }
    })
}
