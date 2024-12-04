//! Type information used in generated FANDANGO grammars.

use pest::Span;

/// A node representing an entry in a grammar or a derivation tree.
pub trait Node<'source>: Sized {
    /// The source code of the grammar in which this node was defined.
    const SOURCE: &'static str;

    /// The span, or [`None`] if the node was generated.
    fn span(&self) -> Option<Span<'source>>;
}

/// Denotes that the provided node has direct children. Alternates must first be unwrapped to their
/// concrete variants.
pub trait Children<'source>: Node<'source> {
    /// The type which references each child individually.
    type ChildrenRef<'program>
    where
        Self: 'program,
        'source: 'program;
    /// The type which mutably references each child individually.
    type ChildrenRefMut<'program>
    where
        Self: 'program,
        'source: 'program;

    /// The spans (in terms of raw offset) under which these nodes were defined.
    const DEF_SPANS: &'static [(usize, usize)];

    /// Immutable accessors to children nodes.
    fn children(&self) -> Self::ChildrenRef<'_>;
    /// Mutable accessors to children nodes.
    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_>;

    /// The spans in the original grammar at which the children are specified.
    fn def_spans() -> impl Iterator<Item = Span<'static>> {
        Self::DEF_SPANS
            .iter()
            .map(|&(start, end)| Span::new(Self::SOURCE, start, end).unwrap())
    }
}

/// The original definition of a non-terminal's production.
pub trait NonterminalProduction<'source>: Children<'source> {
    /// The span (in terms of raw offset) under which this production is defined.
    const DEF_SPAN: (usize, usize);

    /// The span in the original grammar at which the production is specified.
    fn def_span() -> Span<'static> {
        Span::new(Self::SOURCE, Self::DEF_SPAN.0, Self::DEF_SPAN.1).unwrap()
    }
}
