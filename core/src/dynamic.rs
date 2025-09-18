//! Dynamic shims for producing inputs without static typing.
//!
//! This is currently not feature complete.

#![expect(deprecated)]

use crate::generation::{DefaultGenerated, Generated, GeneratorTuple, InPlaceGenerated, Sampler};
use crate::lang::{Operator, Symbol};
use crate::typing::{
    AsNode, AsNodeMut, AsNodeRef, AssignFrom, Discriminable, DiscriminantLookup, Node,
};
use crate::visitor::{
    MaybeVisitMutResult, MaybeVisitResult, VisitMutResult, VisitResult, VisitWith, VisitWithMut,
    VisitableChildren, VisitableChildrenMut, Visitor, VisitorMut,
};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::hash::BuildHasher;
use core::ops::ControlFlow;
use hashbrown::{DefaultHashBuilder, HashMap};

type FandangoNode = crate::lang::FandangoNode<'static, 'static>;

/// Content of a [`DynamicNode`], without the typing information.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DynamicNodeVariant {
    /// This node is a terminal for which the content is entirely determined by its definition.
    Terminal,
    /// This node is a sequence, which represents any non-alternation.
    Sequence(Vec<DynamicNode>),
    /// This node is an alternation.
    Alternation {
        /// The variation of the alternation, by index according to the definition.
        variant: usize,
        /// The node content. For consistency in [`DynamicNodeVariant::iter`] and
        /// [`DynamicNodeVariant::iter_mut`], the item is wrapped in a slice.
        content: Box<[DynamicNode; 1]>,
    },
}

impl DynamicNodeVariant {
    /// An immutable iterator over the nodes contained by the [`DynamicNode`].
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&DynamicNode, usize)> {
        match self {
            DynamicNodeVariant::Terminal => [].iter().zip(0..usize::MAX),
            DynamicNodeVariant::Sequence(seq) => seq.iter().zip(0..usize::MAX),
            DynamicNodeVariant::Alternation { content, variant } => {
                content.iter().zip(*variant..usize::MAX)
            }
        }
    }

    /// A mutable iterator over the nodes contained by the [`DynamicNode`].
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (&mut DynamicNode, usize)> {
        match self {
            DynamicNodeVariant::Terminal => [].iter_mut().zip(0..usize::MAX),
            DynamicNodeVariant::Sequence(seq) => seq.iter_mut().zip(0..usize::MAX),
            DynamicNodeVariant::Alternation { content, variant } => {
                content.iter_mut().zip(*variant..usize::MAX)
            }
        }
    }
}

/// A dynamically typed implementation of grammar nodes.
///
/// While this allows you to adjust the grammar definition dynamically, it is both less performant
/// and offers significantly fewer guarantees regarding the correctness of your operations on
/// derivation trees. As there are exceedingly few use cases which justify the use of dynamic nodes,
/// this type is marked with `#[deprecated]`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[deprecated(
    note = "DynamicNode was only created for performing an ablation study. It is universally less performant. Unless you are absolutely certain that you need dynamic typing (e.g., with dynamic grammars), do not use this."
)]
pub struct DynamicNode {
    root: FandangoNode,
    definition: FandangoNode,
    content: DynamicNodeVariant,
}

/// A sampler which maintains the correctness of the derivation tree structure during generation by
/// tracking which node is currently being generated.
///
/// The corresponding [`DefaultGenerated`] implementation of [`DynamicNode`] follows the same
/// structure optimization rules as the static typing (i.e., trivial alternations/concatenations are
/// elided). Ensure that any custom mutation operations you define over [`DynamicNode`]s are
/// consistent with the structure optimization rules as seen in [`DynamicNode::generate_default`].
#[derive(Debug)]
pub struct DynamicSampler<'sampler, S> {
    root: FandangoNode,
    definition: FandangoNode,
    nonterminals: &'sampler HashMap<FandangoNode, FandangoNode>,
    inner: &'sampler mut S,
}

/// Helper trait for wrappers of [`DynamicSampler`]s. See [`Flattener`] for an example.
///
/// Sadly, it is not possible to auto-impl this via e.g. [`core::ops::DerefMut`] due to conflicting
/// trait implementations with [`rand::Rng`].
///
/// Use [`impl_has_dynamic_sampler`] for easy implementation on samplers which wrap others.
pub trait HasDynamicSampler {
    /// The root node of the grammar (i.e., the [`crate::lang::Program`] node).
    fn root(&self) -> FandangoNode;
    /// The current node being generated.
    ///
    /// Since we cannot infer the node type to be generated from the Rust type, this information
    /// must be tracked through the generation process.
    fn definition(&self) -> FandangoNode;
    /// The production rule associated with the provided nonterminal (for resolving a nonterminal).
    fn nonterminal(&self, node: &FandangoNode) -> Option<FandangoNode>;
    /// Update the definition present in the [`DynamicSampler`]. You can provide your own
    /// [`DynamicSampler`] instead.
    fn with_definition(&mut self, definition: FandangoNode) -> &mut Self;
}

/// Helper macro for defining [`HasDynamicSampler`]. Used to generate the content of a
/// [`HasDynamicSampler`] like so:
///
/// ```
/// use fandango_core::dynamic::HasDynamicSampler;
/// use fandango_core::impl_has_dynamic_sampler;
///
/// struct DynamicSamplerWrapper<S> {
///     sampler: S,
/// }
///
/// impl<S> HasDynamicSampler for DynamicSamplerWrapper<S> where S: HasDynamicSampler {
///     impl_has_dynamic_sampler!(sampler);
/// }
/// ```
#[macro_export]
macro_rules! impl_has_dynamic_sampler {
    ($inner: ident) => {
        fn root(&self) -> $crate::lang::FandangoNode<'static, 'static> {
            self.$inner.root()
        }

        fn definition(&self) -> $crate::lang::FandangoNode<'static, 'static> {
            self.$inner.definition()
        }

        fn nonterminal(
            &self,
            node: &$crate::lang::FandangoNode<'static, 'static>,
        ) -> Option<$crate::lang::FandangoNode<'static, 'static>> {
            self.$inner.nonterminal(node)
        }

        fn with_definition(
            &mut self,
            definition: $crate::lang::FandangoNode<'static, 'static>,
        ) -> &mut Self {
            self.$inner.with_definition(definition);
            self
        }
    };
}

/// Get the definition of a node `N`. Allows for static or dynamic checking, according to what's
/// appropriate.
///
/// Use [`impl_definition_of`] for easy implementation on samplers which wrap others.
pub trait DefinitionOf<N> {
    /// The root of the given node.
    fn root_of(&self) -> crate::lang::FandangoNode<'static, 'static>;

    /// The definition of the given node.
    fn definition_of(&self) -> crate::lang::FandangoNode<'static, 'static>;
}

/// Helper macro for defining [`DefinitionOf`], just like [`impl_has_dynamic_handler`].
#[macro_export]
macro_rules! impl_definition_of {
    ($inner: ident) => {
        fn root_of(&self) -> $crate::lang::FandangoNode<'static, 'static> {
            self.$inner.root_of()
        }

        fn definition_of(&self) -> $crate::lang::FandangoNode<'static, 'static> {
            self.$inner.definition_of()
        }
    };
}

impl<S> DefinitionOf<DynamicNode> for DynamicSampler<'_, S> {
    fn root_of(&self) -> crate::lang::FandangoNode<'static, 'static> {
        self.root
    }

    fn definition_of(&self) -> crate::lang::FandangoNode<'static, 'static> {
        self.definition
    }
}

impl<S> HasDynamicSampler for DynamicSampler<'_, S> {
    fn root(&self) -> FandangoNode {
        self.root
    }

    fn definition(&self) -> FandangoNode {
        self.definition
    }

    fn nonterminal(&self, node: &FandangoNode) -> Option<FandangoNode> {
        self.nonterminals.get(node).copied()
    }

    fn with_definition(&mut self, definition: FandangoNode) -> &mut Self {
        self.definition = definition;
        self
    }
}

impl<'sampler, S> DynamicSampler<'sampler, S> {
    /// The inner sampler upon which this [`DynamicSampler`] relies for actual sampling operations.
    pub fn inner(&mut self) -> &mut S {
        &mut *self.inner
    }

    /// Build a [`DynamicSampler`]. Note that you will first need to produce a mapping for
    /// nonterminal productions, e.g. with [`crate::lang::Program::nonterminals`], and an existing
    /// sampler, e.g. [`rand::rngs::StdRng`].
    pub fn new(
        root: FandangoNode,
        definition: FandangoNode,
        nonterminals: &'sampler HashMap<FandangoNode, FandangoNode>,
        inner: &'sampler mut S,
    ) -> Self {
        Self {
            root,
            definition,
            nonterminals,
            inner,
        }
    }
}

impl<S> Sampler<DynamicNode> for DynamicSampler<'_, S>
where
    S: Sampler<DynamicNode>,
{
    fn sample_kleene(&mut self) -> usize {
        self.inner.sample_kleene()
    }

    fn sample_plus(&mut self) -> usize {
        self.inner.sample_plus()
    }

    fn sample_optional(&mut self) -> bool {
        self.inner.sample_optional()
    }

    fn sample_repetition(&mut self, lower: usize, upper: usize) -> usize {
        self.inner.sample_repetition(lower, upper)
    }

    fn sample_alternative(&mut self, count: usize) -> usize {
        self.inner.sample_alternative(count)
    }

    fn sample(&mut self) -> usize {
        self.inner.sample()
    }

    fn reseed(&mut self, seed: u64) {
        self.inner.reseed(seed)
    }
}

impl<S, G> DefaultGenerated<S, G> for DynamicNode
where
    S: Sampler<DynamicNode> + HasDynamicSampler,
    G: GeneratorTuple<DynamicNode, S>,
{
    fn generate_default(sampler: &mut S, with: &mut G, depth: usize) -> Self {
        let definition = sampler.definition();
        let result = (|| match definition {
            FandangoNode::Nonterminal(_) => {
                let inner = sampler
                    .nonterminal(&definition)
                    .expect("Expected a corresponding inner node for this nonterminal.");
                let child = DynamicNode::generate(sampler.with_definition(inner), with, depth + 1);
                Self {
                    root: sampler.root(),
                    definition,
                    content: DynamicNodeVariant::Sequence(vec![child]),
                }
            }
            FandangoNode::Alternative(alt) => {
                if alt.concatenations().len() == 1 {
                    DynamicNode::generate(
                        sampler.with_definition(FandangoNode::from(&alt.concatenations()[0])),
                        with,
                        depth,
                    )
                } else {
                    let variant = sampler.sample_alternative(alt.concatenations().len());
                    Self {
                        root: sampler.root(),
                        definition,
                        content: DynamicNodeVariant::Alternation {
                            variant,
                            content: Box::new([DynamicNode::generate(
                                sampler.with_definition(FandangoNode::from(
                                    &alt.concatenations()[variant],
                                )),
                                with,
                                depth + 1,
                            )]),
                        },
                    }
                }
            }
            FandangoNode::Concatenation(concat) => {
                if concat.operators().len() == 1 {
                    DynamicNode::generate(
                        sampler.with_definition(FandangoNode::from(&concat.operators()[0])),
                        with,
                        depth,
                    )
                } else {
                    Self {
                        root: sampler.root(),
                        definition,
                        content: DynamicNodeVariant::Sequence(
                            concat
                                .operators()
                                .iter()
                                .map(|item| {
                                    DynamicNode::generate(
                                        sampler.with_definition(FandangoNode::from(item)),
                                        with,
                                        depth + 1,
                                    )
                                })
                                .collect(),
                        ),
                    }
                }
            }
            FandangoNode::Operator(op) => {
                let (count, sym) = match op {
                    Operator::Kleene(kl) => (sampler.sample_kleene(), kl),
                    Operator::Plus(pl) => (sampler.sample_plus(), pl),
                    Operator::Option(opt) => (if sampler.sample_optional() { 1 } else { 0 }, opt),
                    Operator::Repeat(rpt, lower, upper) => {
                        (sampler.sample_repetition(*lower, *upper), rpt)
                    }
                    Operator::Symbol(sym) => {
                        return DynamicNode::generate(
                            sampler.with_definition(FandangoNode::from(sym)),
                            with,
                            depth,
                        );
                    }
                };
                let sym = FandangoNode::from(sym);
                Self {
                    root: sampler.root(),
                    definition,
                    content: DynamicNodeVariant::Sequence(
                        (0..count)
                            .map(|_| {
                                DynamicNode::generate(sampler.with_definition(sym), with, depth + 1)
                            })
                            .collect(),
                    ),
                }
            }
            FandangoNode::Symbol(sym) => {
                let inner = match sym {
                    Symbol::Nonterminal(nt) => FandangoNode::from(nt),
                    Symbol::Alternative(alt) => FandangoNode::from(alt),
                    Symbol::String(s) => FandangoNode::from(s),
                };
                DynamicNode::generate(sampler.with_definition(inner), with, depth)
            }
            FandangoNode::String(s) => Self {
                root: sampler.root(),
                definition: FandangoNode::from(s),
                content: DynamicNodeVariant::Terminal,
            },
            _ => unreachable!("Cannot generate this case."),
        })();
        sampler.with_definition(definition);
        result
    }
}

impl<S, G> InPlaceGenerated<S, G> for DynamicNode
where
    S: Sampler<DynamicNode> + HasDynamicSampler,
    G: GeneratorTuple<DynamicNode, S>,
{
    fn generate_in_place(&mut self, sampler: &mut S, with: &mut G, depth: usize) {
        debug_assert_eq!(self.root, sampler.root());
        debug_assert_eq!(self.definition, sampler.definition());

        self.content = DynamicNode::generate(sampler, with, depth).content;
    }
}

impl AsNode for DynamicNode {
    fn root(&self) -> crate::lang::FandangoNode<'static, 'static> {
        self.root
    }

    fn definition(&self) -> crate::lang::FandangoNode<'static, 'static> {
        self.definition
    }
}

impl AssignFrom<&'_ DynamicNode> for &'_ mut DynamicNode {
    fn assign_from(&mut self, other: &'_ DynamicNode) -> bool {
        if self.discriminant() == other.discriminant() {
            **self = other.clone();
            true
        } else {
            false
        }
    }
}

impl DiscriminantLookup for &'_ DynamicNode {
    fn lookup_discriminant(node: &crate::lang::FandangoNode) -> usize {
        DefaultHashBuilder::default().hash_one(node) as usize
    }
}

impl DiscriminantLookup for &'_ mut DynamicNode {
    fn lookup_discriminant(node: &crate::lang::FandangoNode) -> usize {
        DefaultHashBuilder::default().hash_one(node) as usize
    }
}

impl Discriminable for DynamicNode {
    fn discriminant(&self) -> usize {
        <&DynamicNode>::lookup_discriminant(&self.definition)
    }
}

impl Node for DynamicNode {
    type Type<'program>
        = &'program Self
    where
        Self: 'program;
    type TypeMut<'program>
        = &'program mut Self
    where
        Self: 'program;
    type ChildrenRef<'program>
        = &'program DynamicNodeVariant
    where
        Self: 'program;
    type ChildrenRefMut<'program>
        = &'program mut DynamicNodeVariant
    where
        Self: 'program;

    fn children(&self) -> Self::ChildrenRef<'_> {
        &self.content
    }

    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {
        &mut self.content
    }
}

impl Discriminable for &'_ DynamicNode {
    fn discriminant(&self) -> usize {
        (**self).discriminant()
    }
}

impl Discriminable for &'_ mut DynamicNode {
    fn discriminant(&self) -> usize {
        (**self).discriminant()
    }
}

impl AsNodeRef<DynamicNode> for DynamicNode {
    fn as_node(&self) -> Option<&DynamicNode> {
        Some(self)
    }
}

impl AsNodeRef<DynamicNode> for &DynamicNode {
    fn as_node(&self) -> Option<&DynamicNode> {
        Some(self)
    }
}

impl AsNodeRef<DynamicNode> for &mut DynamicNode {
    fn as_node(&self) -> Option<&DynamicNode> {
        Some(self)
    }
}

impl AsNodeMut<DynamicNode> for DynamicNode {
    fn as_node_mut(&mut self) -> Option<&mut DynamicNode> {
        Some(self)
    }
}

impl AsNodeMut<DynamicNode> for &mut DynamicNode {
    fn as_node_mut(&mut self) -> Option<&mut DynamicNode> {
        Some(self)
    }
}

impl<'a> VisitableChildren<&'a DynamicNode> for &'a DynamicNode {
    fn visit_each<V>(self, mut visitor: V) -> VisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode, Continue = V>,
    {
        for (child, idx) in self.children().iter() {
            match visitor.visit(child, idx) {
                Ok(ControlFlow::Continue(v)) => {
                    visitor = v;
                }
                result => return result,
            }
        }
        Ok(ControlFlow::Continue(visitor))
    }

    fn visit_each_reverse<V>(self, mut visitor: V) -> VisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode, Continue = V>,
    {
        for (child, idx) in self.children().iter().rev() {
            match visitor.visit(child, idx) {
                Ok(ControlFlow::Continue(v)) => {
                    visitor = v;
                }
                result => return result,
            }
        }
        Ok(ControlFlow::Continue(visitor))
    }

    fn visit_each_from<V>(self, mut visitor: V, from: usize) -> VisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode, Continue = V>,
    {
        for (child, idx) in self.children().iter() {
            if idx >= from {
                match visitor.visit(child, idx) {
                    Ok(ControlFlow::Continue(v)) => {
                        visitor = v;
                    }
                    result => return result,
                }
            }
        }
        Ok(ControlFlow::Continue(visitor))
    }

    fn visit_each_reverse_from<V>(
        self,
        mut visitor: V,
        from: usize,
    ) -> VisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode, Continue = V>,
    {
        for (child, idx) in self.children().iter() {
            if idx >= from {
                match visitor.visit(child, idx) {
                    Ok(ControlFlow::Continue(v)) => {
                        visitor = v;
                    }
                    result => return result,
                }
            }
        }
        Ok(ControlFlow::Continue(visitor))
    }

    fn visit_nth<V>(self, visitor: V, idx: usize) -> MaybeVisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode>,
    {
        let child = match self.children() {
            DynamicNodeVariant::Sequence(seq) => seq.get(idx),
            #[expect(clippy::get_first)]
            DynamicNodeVariant::Alternation { variant, content } if *variant == idx => {
                content.get(0)
            }
            _ => None,
        };
        match child {
            None => Err(visitor),
            Some(child) => Ok(visitor.visit(child, idx)),
        }
    }
}

impl<'a> VisitableChildren<&'a DynamicNode> for &'a mut DynamicNode {
    fn visit_each<V>(self, visitor: V) -> VisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode, Continue = V>,
    {
        (&*self).visit_each(visitor)
    }

    fn visit_each_reverse<V>(self, visitor: V) -> VisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode, Continue = V>,
    {
        (&*self).visit_each_reverse(visitor)
    }

    fn visit_each_from<V>(self, visitor: V, from: usize) -> VisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode, Continue = V>,
    {
        (&*self).visit_each_from(visitor, from)
    }

    fn visit_each_reverse_from<V>(self, visitor: V, from: usize) -> VisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode, Continue = V>,
    {
        (&*self).visit_each_reverse_from(visitor, from)
    }

    fn visit_nth<V>(self, visitor: V, idx: usize) -> MaybeVisitResult<V, &'a DynamicNode>
    where
        V: Visitor<&'a DynamicNode>,
    {
        (&*self).visit_nth(visitor, idx)
    }
}

impl<'a> VisitableChildrenMut<&'a mut DynamicNode> for &'a mut DynamicNode {
    fn visit_each_mut<V>(self, mut visitor: V) -> VisitMutResult<V, &'a mut DynamicNode>
    where
        V: VisitorMut<&'a mut DynamicNode, Continue = V>,
    {
        for (child, idx) in self.children_mut().iter_mut() {
            match visitor.visit_mut(child, idx) {
                Ok(ControlFlow::Continue(v)) => {
                    visitor = v;
                }
                result => return result,
            }
        }
        Ok(ControlFlow::Continue(visitor))
    }

    fn visit_each_reverse_mut<V>(self, mut visitor: V) -> VisitMutResult<V, &'a mut DynamicNode>
    where
        V: VisitorMut<&'a mut DynamicNode, Continue = V>,
    {
        for (child, idx) in self.children_mut().iter_mut().rev() {
            match visitor.visit_mut(child, idx) {
                Ok(ControlFlow::Continue(v)) => {
                    visitor = v;
                }
                result => return result,
            }
        }
        Ok(ControlFlow::Continue(visitor))
    }

    fn visit_each_mut_from<V>(
        self,
        mut visitor: V,
        from: usize,
    ) -> VisitMutResult<V, &'a mut DynamicNode>
    where
        V: VisitorMut<&'a mut DynamicNode, Continue = V>,
    {
        for (child, idx) in self.children_mut().iter_mut() {
            if idx >= from {
                match visitor.visit_mut(child, idx) {
                    Ok(ControlFlow::Continue(v)) => {
                        visitor = v;
                    }
                    result => return result,
                }
            }
        }
        Ok(ControlFlow::Continue(visitor))
    }

    fn visit_each_reverse_mut_from<V>(
        self,
        mut visitor: V,
        from: usize,
    ) -> VisitMutResult<V, &'a mut DynamicNode>
    where
        V: VisitorMut<&'a mut DynamicNode, Continue = V>,
    {
        for (child, idx) in self.children_mut().iter_mut() {
            if idx >= from {
                match visitor.visit_mut(child, idx) {
                    Ok(ControlFlow::Continue(v)) => {
                        visitor = v;
                    }
                    result => return result,
                }
            }
        }
        Ok(ControlFlow::Continue(visitor))
    }

    fn visit_nth_mut<V>(self, visitor: V, idx: usize) -> MaybeVisitMutResult<V, &'a mut DynamicNode>
    where
        V: VisitorMut<&'a mut DynamicNode>,
    {
        let child = match self.children_mut() {
            DynamicNodeVariant::Sequence(seq) => seq.get_mut(idx),
            DynamicNodeVariant::Alternation { variant, content } if *variant == idx => {
                content.get_mut(0)
            }
            _ => None,
        };
        match child {
            None => Err(visitor),
            Some(child) => Ok(visitor.visit_mut(child, idx)),
        }
    }
}

impl<'a, V> VisitWith<'a, V> for &'a DynamicNode {
    type Visited = &'a DynamicNode;

    fn visit_with(&'a self, visitor: V, idx: usize) -> VisitResult<V, Self::Visited>
    where
        V: Visitor<Self::Visited>,
    {
        visitor.visit(self, idx)
    }
}

impl<'a, V> VisitWithMut<'a, V> for &'a mut DynamicNode {
    type Visited = &'a mut DynamicNode;

    fn visit_with_mut(&'a mut self, visitor: V, idx: usize) -> VisitMutResult<V, Self::Visited>
    where
        V: VisitorMut<Self::Visited>,
    {
        visitor.visit_mut(self, idx)
    }
}
