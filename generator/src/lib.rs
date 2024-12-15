//! Code generation routines for FANDANGO. Only usable if the crate `fandango` is a dependency,
//! and may not be renamed.

mod pest;
mod rust;
mod structure;

use ::pest::Span;
use fandango_core::graph::{FandangoNode, IntoGraph};
use fandango_core::lang::{ParseError, Program};
use pest::error::{InputLocation, LineColLocation};
use quote::quote;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, ExprLit, Lit, Meta};

use crate::pest::IntoPestSource;
use crate::rust::IntoRustSource;
use crate::structure::tokenize_metadata;
use proc_macro2::{Ident, TokenStream, TokenTree};

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

/// Perform the derivation, or emit a compiler error with a (potentially useful) warning.
pub fn derive_fandango_or_emit_error(
    source: FandangoDerivation,
) -> Result<TokenStream, TokenStream> {
    let parsed = Program::try_from(&source.merged).map_err(|e| source.to_compile_error(e))?;

    let graph = (&parsed).into_graph();
    let mut tokenized = TokenStream::new();

    let mut mapped_names = HashMap::new();
    graph.emit_rust(&mut mapped_names, &mut tokenized).unwrap();

    let mut grammar_source = String::new();
    graph.emit_pest(&mut (), &mut grammar_source).unwrap();

    let node_names = mapped_names.values().cloned().collect::<Vec<_>>();

    let mut arrays = Vec::new();
    let mut referenced = Vec::new();
    let metadata = tokenize_metadata(
        &FandangoNode::from(&parsed),
        Span::new(&source.merged, 0, source.merged.len()).unwrap(),
        quote! { STRUCTURE },
        &mut mapped_names,
        &mut arrays,
        &mut referenced,
    );

    let ident = &source.ident;

    let mocked = quote! {
        #[derive(Parser)]
        #[grammar_inline = #grammar_source]
        pub struct #ident;
    };

    let grammar = pest_generator::derive_parser(mocked, false);
    // rewrite: we don't want to force people to import other dependencies
    let grammar = grammar
        .into_iter()
        .map(|e| match e {
            TokenTree::Ident(ident) => {
                if ident == "pest" || ident == "pest_derive" {
                    TokenTree::Ident(Ident::new("fandango", proc_macro2::Span::mixed_site()))
                } else {
                    TokenTree::Ident(ident)
                }
            }
            other => other,
        })
        .collect::<TokenStream>();

    let discriminants = (0usize..node_names.len()).collect::<Vec<_>>();

    Ok(quote! {
        #tokenized

        #(#arrays)*

        #(#referenced)*

        trait TypeSampler<'source>
        where
            #(Self: ::fandango::generation::Sampler<#node_names<'source>>),*
        {}

        impl<'source, S> TypeSampler<'source> for S
        where
            #(S: ::fandango::generation::Sampler<#node_names<'source>>),*
        {}

        trait TypeGenerator<'source, S>
        where
            #(Self: ::fandango::generation::GeneratorTuple<#node_names<'source>, S>),*,
            #(Self: ::fandango::generation::GeneratorTuple<::std::boxed::Box<#node_names<'source>>, S>),*
        {}

        impl<'source, S, G> TypeGenerator<'source, S> for G
        where
            #(G: ::fandango::generation::GeneratorTuple<#node_names<'source>, S>),*,
            #(G: ::fandango::generation::GeneratorTuple<::std::boxed::Box<#node_names<'source>>, S>),*
        {}

        #[derive(Clone, Debug)]
        pub enum Type<'program, 'source> {
            #(#node_names(&'program #node_names<'source>)),*
        }

        #[derive(Debug)]
        pub enum TypeMut<'program, 'source> {
            #(#node_names(&'program mut #node_names<'source>)),*
        }

        impl<'program, 'source> TypeMut<'program, 'source> {
            fn reborrow<'a>(&'a mut self) -> TypeMut<'a, 'source> where 'source: 'a {
                match self {
                    #(TypeMut::#node_names(n) => TypeMut::#node_names(&mut *n)),*
                }
            }
        }

        impl<'program, 'source> From<TypeMut<'program, 'source>> for Type<'program, 'source> {
            fn from(mutable: TypeMut<'program, 'source>) -> Type<'program, 'source> {
                match mutable {
                    #(TypeMut::#node_names(n) => Type::#node_names(n)),*
                }
            }
        }

        impl<'a, 'program, 'source, V> ::fandango::visitor::VisitWith<'a, V> for TypeMut<'program, 'source>
        where
            'program: 'a,
            'source: 'program,
        {
            type Visited = TypeMut<'a, 'source>;

            fn visit_with(&'a mut self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Self::Visited>
            where
                V: ::fandango::visitor::Visitor<TypeMut<'a, 'source>>, {
                match self.reborrow() {
                    #(TypeMut::#node_names(n) => visitor.visit(n, idx)),*
                }
            }
        }

        impl<'a, 'program, 'source, S, G> ::fandango::generation::InPlaceGenerated<'a, S, G> for TypeMut<'program, 'source>
        where
            #(#node_names<'source>: ::fandango::generation::Generated<S, G>),*,
            #(Box<#node_names<'source>>: ::fandango::generation::Generated<S, G>),*,
            'program: 'a,
            'source: 'program,
        {
            fn generate_in_place(&'a mut self, sampler: &mut S, with: &mut G) {
                match self.reborrow() {
                    #(TypeMut::#node_names(n) => {
                        *n = ::fandango::generation::Generated::generate(sampler, with);
                    }),*
                }
            }
        }

        impl<'program, 'source> ::fandango::visitor::VisitableChildren<TypeMut<'program, 'source>> for TypeMut<'program, 'source> {
            fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
            where
                V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
            {
                match self {
                    #(TypeMut::#node_names(n) => n.visit_each(visitor)),*
                }
            }

            fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
            where
                V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue = V>
            {
                match self {
                    #(TypeMut::#node_names(n) => n.visit_each_reverse(visitor)),*
                }
            }

            fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
            where
                V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue=V>
            {
                match self {
                    #(TypeMut::#node_names(n) => n.visit_each_reverse_from(visitor, idx)),*
                }
            }

            fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, TypeMut<'program, 'source>>
            where
                V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>, Continue=V>
            {
                match self {
                    #(TypeMut::#node_names(n) => n.visit_each_from(visitor, idx)),*
                }
            }

            fn visit_nth<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::MaybeVisitResult<V, TypeMut<'program, 'source>>
            where
                V: ::fandango::visitor::Visitor<TypeMut<'program, 'source>>
            {
                match self {
                    #(TypeMut::#node_names(n) => n.visit_nth(visitor, idx)),*
                }
            }
        }

        #(
            impl<'source> ::fandango::typing::Discriminable for #node_names<'source>
            {
                const DISCRIMINANT: usize = #discriminants;
            }
        )*

        pub const STRUCTURE: &'static ::fandango::lang::Tagged<
            'static,
            ::fandango::lang::Program
        > = &#metadata;

        // debug: pest grammar source
        pub const _PEST_SOURCE: &'static str = #grammar_source;

        impl #ident {
            pub fn extract<'source>(
                source: &'source str
            ) -> ::std::result::Result<nonterminal_start<'_>, ParseError> {
                use ::fandango::Parser;

                let (grammar,) = ::fandango::parse_pairs_as!(#ident::parse(Rule::start, source)?, (Rule::start,));
                let source = ::std::rc::Rc::new(::std::borrow::Cow::Borrowed(source));

                nonterminal_start::try_from((source, grammar))
            }
        }

        #grammar
    })
}
