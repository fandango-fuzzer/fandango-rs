use crate::generation::{DefaultGenerated, Generated, GeneratorTuple, Sampler};
use crate::lang::{Operator, Symbol};
use crate::typing::{AsNode, Discriminable, Node};
use crate::visitor::{MaybeVisitResult, VisitResult, VisitableChildren, Visitor};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::hash::{BuildHasher, Hash};
use core::ops::ControlFlow;
use hashbrown::{DefaultHashBuilder, HashMap};
use pest::Span;

pub type FandangoNode = crate::lang::FandangoNode<'static, 'static>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DynamicNodeVariant {
    Terminal,
    Sequence(Vec<DynamicNode>),
    Alternation {
        variant: usize,
        content: Box<[DynamicNode; 1]>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DynamicNode {
    root: FandangoNode,
    definition: FandangoNode,
    content: DynamicNodeVariant,
}

#[derive(Debug)]
pub struct DynamicSampler<'sampler, S> {
    root: FandangoNode,
    definition: FandangoNode,
    nonterminals: &'sampler HashMap<FandangoNode, FandangoNode>,
    inner: &'sampler mut S,
}

impl<'sampler, S> DynamicSampler<'sampler, S> {
    pub fn root(&self) -> FandangoNode {
        self.root
    }

    pub fn definition(&self) -> FandangoNode {
        self.definition
    }

    pub fn nonterminal(&self, node: &FandangoNode) -> Option<FandangoNode> {
        self.nonterminals.get(node).copied()
    }

    pub fn with_definition(&mut self, definition: FandangoNode) -> DynamicSampler<'_, S> {
        DynamicSampler::<'_, S> {
            root: self.root,
            definition,
            nonterminals: &*self.nonterminals,
            inner: &mut *self.inner,
        }
    }

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
}

impl<S, G> DefaultGenerated<DynamicSampler<'_, S>, G> for DynamicNode
where
    S: Sampler<DynamicNode>,
    for<'a> G: GeneratorTuple<DynamicNode, DynamicSampler<'a, S>>,
{
    fn generate_default(sampler: &mut DynamicSampler<'_, S>, with: &mut G) -> Self {
        let definition = sampler.definition();
        match definition {
            FandangoNode::Nonterminal(_) => {
                let inner = sampler
                    .nonterminal(&definition)
                    .expect("Expected a corresponding inner node for this nonterminal.");
                let child = DynamicNode::generate(&mut sampler.with_definition(inner), with);
                Self {
                    root: sampler.root(),
                    definition,
                    content: DynamicNodeVariant::Sequence(vec![child]),
                }
            }
            FandangoNode::Alternative(alt) => {
                if alt.concatenations().len() == 1 {
                    DynamicNode::generate(
                        &mut sampler.with_definition(FandangoNode::from(&alt.concatenations()[0])),
                        with,
                    )
                } else {
                    let variant = sampler.sample_alternative(alt.concatenations().len());
                    Self {
                        root: sampler.root(),
                        definition,
                        content: DynamicNodeVariant::Alternation {
                            variant,
                            content: Box::new([DynamicNode::generate(
                                &mut sampler.with_definition(FandangoNode::from(
                                    &alt.concatenations()[variant],
                                )),
                                with,
                            )]),
                        },
                    }
                }
            }
            FandangoNode::Concatenation(concat) => {
                if concat.operators().len() == 1 {
                    DynamicNode::generate(
                        &mut sampler.with_definition(FandangoNode::from(&concat.operators()[0])),
                        with,
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
                                        &mut sampler.with_definition(FandangoNode::from(item)),
                                        with,
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
                    Operator::Symbol(sym) => (1, sym),
                };
                let sym = FandangoNode::from(sym);
                let mut sampler = sampler.with_definition(sym);
                Self {
                    root: sampler.root(),
                    definition,
                    content: DynamicNodeVariant::Sequence(
                        (0..count)
                            .map(|_| DynamicNode::generate(&mut sampler, with))
                            .collect(),
                    ),
                }
            }
            FandangoNode::Symbol(sym) => match sym {
                Symbol::Nonterminal(nt) => DynamicNode::generate(
                    &mut sampler.with_definition(FandangoNode::from(nt)),
                    with,
                ),
                Symbol::Alternative(alt) => DynamicNode::generate(
                    &mut sampler.with_definition(FandangoNode::from(alt)),
                    with,
                ),
                Symbol::String(s) => Self {
                    root: sampler.root(),
                    definition: FandangoNode::from(s),
                    content: DynamicNodeVariant::Terminal,
                },
            },
            _ => unreachable!("Cannot generate this case."),
        }
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

impl Discriminable for DynamicNode {
    fn discriminant(&self) -> usize {
        DefaultHashBuilder::default().hash_one(self.definition) as usize
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
        = &'program [DynamicNode]
    where
        Self: 'program;
    type ChildrenRefMut<'program>
        = &'program mut [DynamicNode]
    where
        Self: 'program;

    fn span(&self) -> Option<Span<'_>> {
        None
    }

    fn clear_span(&mut self) {}

    fn children(&self) -> Self::ChildrenRef<'_> {
        match &self.content {
            DynamicNodeVariant::Terminal => &[],
            DynamicNodeVariant::Sequence(seq) => seq.as_slice(),
            DynamicNodeVariant::Alternation { content, .. } => content.as_slice(),
        }
    }

    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_> {
        match &mut self.content {
            DynamicNodeVariant::Terminal => &mut [],
            DynamicNodeVariant::Sequence(seq) => seq.as_mut_slice(),
            DynamicNodeVariant::Alternation { content, .. } => content.as_mut_slice(),
        }
    }
}

impl<'a> VisitableChildren<&'a mut DynamicNode> for &'a mut DynamicNode {
    fn visit_each<V>(self, visitor: V) -> VisitResult<V, &'a mut DynamicNode>
    where
        V: Visitor<&'a mut DynamicNode, Continue = V>,
    {
        let mut result = Ok(ControlFlow::Continue(visitor));
        for (idx, child) in self.children_mut().iter_mut().enumerate() {
            match result {
                Ok(ControlFlow::Continue(visitor)) => {
                    result = visitor.visit(child, idx);
                }
                result => return result,
            }
        }
        result
    }

    fn visit_each_reverse<V>(self, visitor: V) -> VisitResult<V, &'a mut DynamicNode>
    where
        V: Visitor<&'a mut DynamicNode, Continue = V>,
    {
        let mut result = Ok(ControlFlow::Continue(visitor));
        for (idx, child) in self.children_mut().iter_mut().enumerate().rev() {
            match result {
                Ok(ControlFlow::Continue(visitor)) => {
                    result = visitor.visit(child, idx);
                }
                result => return result,
            }
        }
        result
    }

    fn visit_each_from<V>(self, visitor: V, idx: usize) -> VisitResult<V, &'a mut DynamicNode>
    where
        V: Visitor<&'a mut DynamicNode, Continue = V>,
    {
        let mut result = Ok(ControlFlow::Continue(visitor));
        for (idx, child) in self.children_mut().iter_mut().enumerate().skip(idx) {
            match result {
                Ok(ControlFlow::Continue(visitor)) => {
                    result = visitor.visit(child, idx);
                }
                result => return result,
            }
        }
        result
    }

    fn visit_each_reverse_from<V>(
        self,
        visitor: V,
        idx: usize,
    ) -> VisitResult<V, &'a mut DynamicNode>
    where
        V: Visitor<&'a mut DynamicNode, Continue = V>,
    {
        let mut result = Ok(ControlFlow::Continue(visitor));
        for (idx, child) in self
            .children_mut()
            .iter_mut()
            .enumerate()
            .rev()
            .skip_while(|(i, _)| *i > idx)
        {
            match result {
                Ok(ControlFlow::Continue(visitor)) => {
                    result = visitor.visit(child, idx);
                }
                result => return result,
            }
        }
        result
    }

    fn visit_nth<V>(self, visitor: V, idx: usize) -> MaybeVisitResult<V, &'a mut DynamicNode>
    where
        V: Visitor<&'a mut DynamicNode>,
    {
        match self.children_mut().get_mut(idx) {
            None => Err(visitor),
            Some(child) => Ok(visitor.visit(child, idx)),
        }
    }
}
