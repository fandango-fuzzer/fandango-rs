//! Code generation routines for FANDANGO. Only usable if the crate `fandango` is a dependency,
//! and may not be renamed.

mod pest;
mod rust;
mod structure;

use ::pest::Span;
use fandango_core::graph::IntoGraph;
use fandango_core::lang::{FandangoNode, ParseError, Program};
use hashbrown::HashMap;
use pest::error::{InputLocation, LineColLocation};
use quote::{format_ident, quote};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env, fs};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, ExprLit, Lit, Meta, MetaNameValue, Token};

use crate::pest::IntoPestSource;
use crate::rust::IntoRustSource;
use crate::structure::tokenize_metadata;
use proc_macro2::{Ident, TokenStream};
use syn::punctuated::Punctuated;

/// A `#[derive]`-style derivation of a FANDANGO grammar.
pub struct FandangoDerivation {
    ident: Ident,
    merged: String,
    offsets: BTreeMap<usize, PathBuf>,
    parse: bool,
    dynamic: bool,
    serde: bool,
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
                let (&lower, path) = self.offsets.range(0..=offset).last()?;
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
        let stringified = format!("Parse error: {error}");

        quote! {
            compile_error!(#stringified)
        }
    }

    fn parse(&self) -> bool {
        self.parse
    }

    fn dynamic(&self) -> bool {
        self.dynamic
    }

    fn serde(&self) -> bool {
        self.serde
    }
}

impl Parse for FandangoDerivation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
        let derived: DeriveInput = input.parse()?;
        let ident = derived.ident;
        let mut merged = Vec::new();
        let mut offsets = BTreeMap::new();
        let mut parse = true;
        let mut dynamic = false;
        let mut serde = false;
        for attr in derived.attrs {
            let span = attr.span();
            if let Meta::List(v) = attr.meta
                && let Some(ident) = v.path.get_ident()
                && ident == "fandango"
            {
                let args =
                    v.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;
                for arg in args {
                    if let Some(ident) = arg.path.get_ident() {
                        if ident == "grammar" {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(s), ..
                            }) = arg.value
                            {
                                let path = root.join(s.value());
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
                        } else if ident == "parse" {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Bool(b), ..
                            }) = arg.value
                            {
                                parse = b.value();
                                continue;
                            }
                        } else if ident == "dynamic" {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Bool(b), ..
                            }) = arg.value
                            {
                                dynamic = b.value();
                                continue;
                            }
                        } else if ident == "serde"
                            && let Expr::Lit(ExprLit {
                                lit: Lit::Bool(b), ..
                            }) = arg.value
                        {
                            serde = b.value();
                            continue;
                        }
                        return Err(syn::Error::new(span, "Invalid fandango list."));
                    }
                }
            }
        }
        if offsets.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Invalid fandango derivation; need at least 1 grammar specified. Use #[fandango(grammar = ...)].",
            ));
        }
        let merged = String::from_utf8(merged).map_err(|e| {
            let invalid = e.utf8_error().valid_up_to() + 1;
            let (&lower, path) = offsets.range(0..invalid).last().unwrap();
            let invalid = invalid - lower;
            let path = path.to_string_lossy();
            syn::Error::new(
                proc_macro2::Span::mixed_site(),
                format!("Invalid UTF-8 byte at {path}:{invalid}."),
            )
        })?;
        Ok(Self {
            ident,
            merged,
            offsets,
            parse,
            dynamic,
            serde,
        })
    }
}

/// Perform the derivation, or emit a compiler error with a (potentially useful) warning.
pub fn derive_fandango_or_emit_error(
    source: FandangoDerivation,
) -> Result<TokenStream, TokenStream> {
    let parsed = Program::try_from(&source.merged).map_err(|e| source.to_compile_error(e))?;
    let files = source.offsets.values().map(|f| {
        fs::canonicalize(f.as_path())
            .expect("Couldn't canonicalize a source path?")
            .to_str()
            .expect("Filename was not UTF-8 compatible.")
            .to_string()
    });

    let input = parsed.statements()[0].span().get_input();

    let (_lookup, graph) = (&parsed).into_graph();
    let mut tokenized = TokenStream::new();

    let mut mapped_names = HashMap::new();
    graph
        .emit_rust(
            (&mut mapped_names, source.parse(), source.serde()),
            &mut tokenized,
        )
        .unwrap();

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

    let dyn_content = quote! {
        #(#arrays)*

        #(#referenced)*

        const SOURCE: &'static str = #input;
        const SOURCE_FILES: &'static [&'static str] = &[#(
            include_str!(#files)
        ),*];

        #[allow(missing_docs)]
        pub const STRUCTURE: &'static ::fandango::lang::Tagged<
            'static,
            ::fandango::lang::Program
        > = &#metadata;
    };

    if source.dynamic() {
        Ok(quote! {
            #dyn_content

            #(
                struct #node_names(::core::convert::Infallible);
            )*
        })
    } else {
        let ident = &source.ident;
        let module = format_ident!("__{ident}_defs");

        let grammar = if source.parse() {
            let mut grammar_source = String::new();
            graph.emit_pest(&mut (), &mut grammar_source).unwrap();
            let mocked = quote! {
                #[derive(Parser)]
                #[grammar_inline = #grammar_source]
                pub struct #ident;
            };

            let mut grammar = pest_generator::derive_parser(mocked, false);
            // rewrite: we don't want to force people to import other dependencies
            grammar.extend(quote! {
            impl #ident {
                #[allow(missing_docs)]
                pub fn extract(
                    source: &str
                ) -> ::core::result::Result<nonterminal_start, ParseError> {
                    use ::pest::Parser;

                    let (grammar,) = ::fandango::parse_pairs_as!(#ident::parse(Rule::start, source)?, (Rule::start,));

                    nonterminal_start::try_from(grammar)
                }
            }
        });
            grammar
        } else {
            TokenStream::new()
        };

        let discriminants = (0usize..node_names.len()).collect::<Vec<_>>();

        Ok(quote! {
            mod #module {
                #tokenized

                #dyn_content

                trait TypeSampler
                where
                    #(Self: ::fandango::generation::Sampler<#node_names>),*
                {}

                impl<S> TypeSampler for S
                where
                    #(S: ::fandango::generation::Sampler<#node_names>),*
                {}

                trait TypeGenerator<S>
                where
                    #(Self: ::fandango::generation::GeneratorTuple<#node_names, S>),*,
                {}

                impl<S, G> TypeGenerator<S> for G
                where
                    #(G: ::fandango::generation::GeneratorTuple<#node_names, S>),*,
                {}

                #[derive(Clone, Copy, Debug, Eq, PartialEq)]
                #[allow(missing_docs)]
                pub enum Type<'program> {
                    #(#node_names(&'program #node_names)),*
                }

                impl<'program> PartialEq<TypeMut<'program>> for Type<'program> {
                    fn eq(&self, other: &TypeMut<'program>) -> bool {
                        match self {
                            #(
                                Type::#node_names(n1) => match other {
                                    TypeMut::#node_names(n2) => *n1 == *n2,
                                    _ => false,
                                }
                            )*
                        }
                    }
                }

                impl ::fandango::typing::Discriminable for Type<'_> {
                    fn discriminant(&self) -> usize {
                        match self {
                            #(
                                Self::#node_names(n) => n.discriminant(),
                            )*
                        }
                    }
                }

                impl ::fandango::typing::DiscriminantLookup for Type<'_> {
                    fn lookup_discriminant(node: &::fandango::lang::FandangoNode<'static, 'static>) -> usize {
                        use ::fandango::typing::{AsStaticNode, StaticDiscriminable};
                        #(
                            if node == &#node_names::static_definition() {
                                return #node_names::DISCRIMINANT;
                            }
                        )*
                        panic!("Could not find a discriminant for the provided node type. Wrong grammar?");
                    }
                }

                impl ::fandango::typing::NodeLookup for Type<'_> {
                    fn lookup_node(discriminant: usize) -> ::fandango::lang::FandangoNode<'static, 'static> {
                        use ::fandango::typing::AsStaticNode;
                        match discriminant {
                            #(
                                #discriminants => #node_names::static_definition(),
                            )*
                            _ => panic!("Could not find a discriminant for the provided node type. Wrong grammar?")
                        }
                    }
                }

                #(
                    impl<'program> ::fandango::typing::AsNodeRef<#node_names> for Type<'program> {
                        fn as_node(&self) -> Option<&#node_names> {
                            match self {
                                Self::#node_names(n) => Some(n),
                                _ => None,
                            }
                        }
                    }
                )*

                impl<'program> Type<'program> {
                    fn reborrow<'a>(&'a self) -> Type<'a> {
                        match self {
                            #(Type::#node_names(n) => Type::#node_names(&*n)),*
                        }
                    }
                }

                impl<'a, 'program, V> ::fandango::visitor::VisitWith<'a, V> for Type<'program>
                where
                    'program: 'a,
                {
                    type Visited = Type<'a>;

                    fn visit_with(&'a self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Self::Visited>
                    where
                        V: ::fandango::visitor::Visitor<Type<'a>>, {
                        match self.reborrow() {
                            #(Type::#node_names(n) => visitor.visit(n, idx)),*
                        }
                    }
                }

                impl<'program> ::fandango::visitor::VisitableChildren<Type<'program>> for Type<'program> {
                    fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                    {
                        match self {
                            #(Type::#node_names(n) => n.visit_each(visitor)),*
                        }
                    }

                    fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                    {
                        match self {
                            #(Type::#node_names(n) => n.visit_each_reverse(visitor)),*
                        }
                    }

                    fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>, Continue=V>
                    {
                        match self {
                            #(Type::#node_names(n) => n.visit_each_reverse_from(visitor, idx)),*
                        }
                    }

                    fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>, Continue=V>
                    {
                        match self {
                            #(Type::#node_names(n) => n.visit_each_from(visitor, idx)),*
                        }
                    }

                    fn visit_nth<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::MaybeVisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>>
                    {
                        match self {
                            #(Type::#node_names(n) => n.visit_nth(visitor, idx)),*
                        }
                    }
                }

                #[derive(Debug, Eq, PartialEq)]
                #[allow(missing_docs)]
                pub enum TypeMut<'program> {
                    #(#node_names(&'program mut #node_names)),*
                }

                impl<'program> PartialEq<Type<'program>> for TypeMut<'program> {
                    fn eq(&self, other: &Type<'program>) -> bool {
                        other.eq(self)
                    }
                }

                impl ::fandango::typing::Discriminable for TypeMut<'_> {
                    fn discriminant(&self) -> usize {
                        match self {
                            #(
                                Self::#node_names(n) => n.discriminant(),
                            )*
                        }
                    }
                }

                impl<'a> ::fandango::typing::DiscriminantLookup for TypeMut<'a> {
                    fn lookup_discriminant(node: &::fandango::lang::FandangoNode<'static, 'static>) -> usize {
                        <Type::<'a> as ::fandango::typing::DiscriminantLookup>::lookup_discriminant(node)
                    }
                }

                impl<'a> ::fandango::typing::NodeLookup for TypeMut<'a> {
                    fn lookup_node(discriminant: usize) -> ::fandango::lang::FandangoNode<'static, 'static> {
                        <Type::<'a> as ::fandango::typing::NodeLookup>::lookup_node(discriminant)
                    }
                }

                #(
                    impl<'program> ::fandango::typing::AsNodeRef<#node_names> for TypeMut<'program> {
                        fn as_node(&self) -> Option<&#node_names> {
                            match self {
                                Self::#node_names(n) => Some(n),
                                _ => None,
                            }
                        }
                    }

                    impl<'program> ::fandango::typing::AsNodeMut<#node_names> for TypeMut<'program> {
                        fn as_node_mut(&mut self) -> Option<&mut #node_names> {
                            match self {
                                Self::#node_names(n) => Some(n),
                                _ => None,
                            }
                        }
                    }
                )*

                impl<'program> TypeMut<'program> {
                    fn reborrow<'a>(&'a self) -> Type<'a> {
                        match self {
                            #(TypeMut::#node_names(n) => Type::#node_names(&*n)),*
                        }
                    }

                    fn reborrow_mut<'a>(&'a mut self) -> TypeMut<'a> {
                        match self {
                            #(TypeMut::#node_names(n) => TypeMut::#node_names(&mut *n)),*
                        }
                    }
                }

                impl<'program> From<TypeMut<'program>> for Type<'program> {
                    fn from(mutable: TypeMut<'program>) -> Type<'program> {
                        match mutable {
                            #(TypeMut::#node_names(n) => Type::#node_names(n)),*
                        }
                    }
                }

                impl<'program> ::fandango::typing::AssignFrom<Type<'program>> for TypeMut<'program> {
                    fn assign_from(&mut self, other: Type<'program>) -> bool {
                        match self {
                            #(TypeMut::#node_names(n) => match other {
                                Type::#node_names(v) => {
                                    **n = v.clone();
                                    true
                                },
                                _ => false,
                            }),*
                        }
                    }
                }

                impl<'a, 'program, V> ::fandango::visitor::VisitWith<'a, V> for TypeMut<'program>
                where
                    'program: 'a,
                {
                    type Visited = Type<'a>;

                    fn visit_with(&'a self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Self::Visited>
                    where
                        V: ::fandango::visitor::Visitor<Type<'a>>, {
                        match self.reborrow() {
                            #(Type::#node_names(n) => visitor.visit(n, idx)),*
                        }
                    }
                }

                impl<'a, 'program, V> ::fandango::visitor::VisitWithMut<'a, V> for TypeMut<'program>
                where
                    'program: 'a,
                {
                    type Visited = TypeMut<'a>;

                    fn visit_with_mut(&'a mut self, visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, Self::Visited>
                    where
                        V: ::fandango::visitor::VisitorMut<TypeMut<'a>>, {
                        match self.reborrow_mut() {
                            #(TypeMut::#node_names(n) => visitor.visit_mut(n, idx)),*
                        }
                    }
                }

                impl<'program> ::fandango::visitor::VisitableChildren<Type<'program>> for TypeMut<'program> {
                    fn visit_each<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_each(visitor)),*
                        }
                    }

                    fn visit_each_reverse<V>(self, visitor: V) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>, Continue = V>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_each_reverse(visitor)),*
                        }
                    }

                    fn visit_each_reverse_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>, Continue=V>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_each_reverse_from(visitor, idx)),*
                        }
                    }

                    fn visit_each_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>, Continue=V>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_each_from(visitor, idx)),*
                        }
                    }

                    fn visit_nth<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::MaybeVisitResult<V, Type<'program>>
                    where
                        V: ::fandango::visitor::Visitor<Type<'program>>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_nth(visitor, idx)),*
                        }
                    }
                }

                impl<'program> ::fandango::visitor::VisitableChildrenMut<TypeMut<'program>> for TypeMut<'program> {
                    fn visit_each_mut<V>(self, visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                    where
                        V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_each_mut(visitor)),*
                        }
                    }

                    fn visit_each_reverse_mut<V>(self, visitor: V) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                    where
                        V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue = V>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_each_reverse_mut(visitor)),*
                        }
                    }

                    fn visit_each_reverse_mut_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                    where
                        V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue=V>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_each_reverse_mut_from(visitor, idx)),*
                        }
                    }

                    fn visit_each_mut_from<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::VisitMutResult<V, TypeMut<'program>>
                    where
                        V: ::fandango::visitor::VisitorMut<TypeMut<'program>, Continue=V>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_each_mut_from(visitor, idx)),*
                        }
                    }

                    fn visit_nth_mut<V>(self, visitor: V, idx: usize) -> ::fandango::visitor::MaybeVisitMutResult<V, TypeMut<'program>>
                    where
                        V: ::fandango::visitor::VisitorMut<TypeMut<'program>>
                    {
                        match self {
                            #(TypeMut::#node_names(n) => n.visit_nth_mut(visitor, idx)),*
                        }
                    }
                }

                impl<'program, S, G> ::fandango::generation::InPlaceGenerated<S, G> for TypeMut<'program>
                where
                    #(#node_names: ::fandango::generation::Generated<S, G>),*,
                {
                    fn generate_in_place(&mut self, sampler: &mut S, with: &mut G, depth: usize) {
                        match self.reborrow_mut() {
                            #(TypeMut::#node_names(n) => {
                                *n = ::fandango::generation::Generated::generate(sampler, with, depth);
                            }),*
                        }
                    }
                }

                #(
                    impl ::fandango::typing::StaticDiscriminable for #node_names
                    {
                        const DISCRIMINANT: usize = #discriminants;
                    }

                    impl ::fandango::typing::Discriminable for #node_names
                    {
                        fn discriminant(&self) -> usize {
                            <Self as ::fandango::typing::StaticDiscriminable>::DISCRIMINANT
                        }
                    }
                )*

                use super::#ident;
                #grammar
            }

            pub use #module::*;
        })
    }
}
