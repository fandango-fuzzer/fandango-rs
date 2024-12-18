use crate::graph::{traverse_children, FandangoNode, GraphTraverse};
use crate::lang::{Nonterminal, ParseError, Rule, Statement, Tagged};
use pest::iterators::Pair;
use pest::Span;
use std::fmt::{Debug, Display, Formatter, Write};
use std::iter;
use std::ops::Deref;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Constraint<'a> {
    Fitness(Tagged<'a, Expr<'a>>),
    Implies(Tagged<'a, Implies<'a>>),
}

impl_fandango_traverse!(Constraint, match { Fitness(fitness), Implies(implies) });

impl<'a> TryFrom<Pair<'a, Rule>> for Constraint<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::constraint);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::expr => Constraint::Fitness(Pair::try_into(inner)?),
            Rule::implies => Constraint::Implies(Pair::try_into(inner)?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Implies<'a> {
    quantifier: Tagged<'a, Quantifier<'a>>,
    implies: Option<Box<Tagged<'a, Implies<'a>>>>,
}

impl<'program, 'source: 'program> GraphTraverse<'program> for &'program Implies<'source> {
    type Node = FandangoNode<'program, 'source>;

    fn traverse<F>(self, consumer: F)
    where
        F: FnMut(Self::Node, Self::Node, Span<'program>),
    {
        traverse_children(
            self,
            {
                let next = iter::once(&self.quantifier).map(From::from);
                next.chain(self.implies.iter().map(Deref::deref).map(From::from))
            },
            consumer,
        );
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Implies<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::implies);

        let mut inner = value.into_inner();
        let quantifier = Pair::try_into(inner.next().unwrap())?;
        let implies = if let Some(implies) = inner {
            Some(Box::new(Pair::try_into(implies)?))
        } else {
            None
        };
        Ok(Self {
            quantifier,
            implies,
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Quantifier<'a> {
    Forall(Tagged<'a, QuantifierSpecification<'a>>),
    Exists(Tagged<'a, QuantifierSpecification<'a>>),
    Disjunction(Tagged<'a, Disjunction<'a>>),
}

impl_fandango_traverse!(Quantifier, match { Forall(forall), Exists(exists), Disjunction(disjunction) });

impl<'a> TryFrom<Pair<'a, Rule>> for Quantifier<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::quantifier);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::forall_specification => {
                Quantifier::Forall(Pair::try_into(inner.into_inner().next().unwrap())?)
            }
            Rule::exists_specification => {
                Quantifier::Exists(Pair::try_into(inner.into_inner().next().unwrap())?)
            }
            Rule::formula_disjunction => Quantifier::Disjunction(Pair::try_into(inner)?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct QuantifierSpecification<'a> {
    nonterminal: Tagged<'a, Nonterminal<'a>>,
    selector: Tagged<'a, Selector<'a>>,
    quantifier: Box<Tagged<'a, Quantifier<'a>>>,
}

impl<'program, 'source: 'program> GraphTraverse<'program>
    for &'program QuantifierSpecification<'source>
{
    type Node = FandangoNode<'program, 'source>;

    fn traverse<F>(self, consumer: F)
    where
        F: FnMut(Self::Node, Self::Node, Span<'program>),
    {
        traverse_children(
            self,
            {
                let next = iter::once(&self.nonterminal).map(From::from);
                {
                    let next = next.chain(iter::once(&self.selector).map(From::from));
                    next.chain(
                        iter::once(&self.quantifier)
                            .map(Deref::deref)
                            .map(From::from),
                    )
                }
            },
            consumer,
        );
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for QuantifierSpecification<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::quantifier_specification);

        let mut inner = value.into_inner();

        Ok(Self {
            nonterminal: Pair::try_into(inner.next().unwrap())?,
            selector: Pair::try_into(inner.next().unwrap())?,
            quantifier: Pair::try_into(inner.next().unwrap())?,
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Disjunction<'a> {
    conjunctions: Vec<Tagged<'a, Conjunction<'a>>>,
}

impl_fandango_traverse!(Disjunction, [conjunctions]);

impl<'a> TryFrom<Pair<'a, Rule>> for Disjunction<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::formula_disjunction);

        let mut inner = value.into_inner();

        Ok(Self {
            conjunctions: inner
                .map(|p| Pair::try_from(p))
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Conjunction<'a> {
    atoms: Vec<Tagged<'a, Atom<'a>>>,
}

impl_fandango_traverse!(Conjunction, [atoms]);

impl<'a> TryFrom<Pair<'a, Rule>> for Conjunction<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::formula_conjunction);

        let mut inner = value.into_inner();

        Ok(Self {
            atoms: inner
                .map(|p| Pair::try_from(p))
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Atom<'a> {
    Comparison(Tagged<'a, Comparison<'a>>),
    Implies(Tagged<'a, Implies<'a>>), // no indirection needed, we are in a Vec
    Expr(Tagged<'a, Expr<'a>>),
}

impl_fandango_traverse!(Atom, match { Comparison(comp), Implies(implies), Expr(expr) });

impl<'a> TryFrom<Pair<'a, Rule>> for Atom<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::formula_atom);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::formula_comparison => Atom::Comparison(Pair::try_into(inner)?),
            Rule::implies => Atom::Implies(Pair::try_into(inner)?),
            Rule::expr => Atom::Expr(Pair::try_into(inner)?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Comparison<'a> {
    left: Tagged<'a, Expr<'a>>,
    right: Tagged<'a, Expr<'a>>,
    operator: Tagged<'a, ConstraintOperator>,
}

impl_fandango_traverse!(Comparison, left, right, operator);

impl<'a> TryFrom<Pair<'a, Rule>> for Comparison<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::formula_comparison);

        let mut inner = value.into_inner();

        Ok(Self {
            left: Pair::try_into(inner.next().unwrap())?,
            operator: Pair::try_into(inner.next().unwrap())?,
            right: Pair::try_into(inner.next().unwrap())?,
        })
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ConstraintOperator {
    Neq,
    Lt,
    LtEq,
    Eq,
    GtEq,
    Gt,
}

impl<'a> TryFrom<Pair<'a, Rule>> for ConstraintOperator<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::constraint_operator);

        Ok(match value.as_str() {
            "<>" | "!=" => ConstraintOperator::Neq,
            "<" => ConstraintOperator::Lt,
            "<=" => ConstraintOperator::LtEq,
            "==" => ConstraintOperator::Eq,
            ">=" => ConstraintOperator::GtEq,
            ">" => ConstraintOperator::Gt,
            _ => unreachable!("This case is not handled in the grammar."),
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Expr<'a> {
    Selector(Tagged<'a, SelectorLength<'a>>),
    // ConstraintIte(Tagged<'a, ConstraintIte<'a>>),
    Inversion(Tagged<'a, Inversion<'a>>),
}

impl_fandango_traverse!(Expr, match { Selector(selector) });

impl<'a> TryFrom<Pair<'a, Rule>> for Expr<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::expr);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::selector_maybe_length => Expr::Selector(Pair::try_into(inner)?),
            Rule::inversion => Expr::Inversion(Pair::try_into(inner)?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum SelectorLength<'a> {
    WithLength(Tagged<'a, Selector<'a>>),
    NoLength(Tagged<'a, Selector<'a>>),
}

impl_fandango_traverse!(SelectorLength, match { WithLength(selector), NoLength(selector) });

impl<'a> TryFrom<Pair<'a, Rule>> for SelectorLength<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::selector_maybe_length);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::selector_length => {
                SelectorLength::WithLength(Pair::try_into(inner.into_inner().next().unwrap())?)
            }
            Rule::selector => SelectorLength::NoLength(Pair::try_into(inner)?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Selector<'a> {
    ChildSelector(Tagged<'a, Selection<'a>>, Box<Tagged<'a, Selector<'a>>>),
    PathSelector(Tagged<'a, Selection<'a>>, Box<Tagged<'a, Selector<'a>>>),
    Basic(Tagged<'a, Selection<'a>>),
}

impl<'program, 'source: 'program> GraphTraverse<'program> for &'program Selector<'source> {
    type Node = FandangoNode<'program, 'source>;

    fn traverse<F>(self, consumer: F)
    where
        F: FnMut(Self::Node, Self::Node, Span<'program>),
    {
        #![allow(unused_imports)]
        use Selector::*;
        match self {
            ChildSelector(basic, child) => traverse_children(
                self,
                iter::once(basic)
                    .map(From::from)
                    .chain(iter::once(child.deref()).map(From::from)),
                consumer,
            ),
            PathSelector(basic, descendent) => traverse_children(
                self,
                iter::once(basic)
                    .map(From::from)
                    .chain(iter::once(descendent.deref()).map(From::from)),
                consumer,
            ),
            Basic(basic) => traverse_children(self, iter::once(basic).map(From::from), consumer),
        }
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Selector<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::selector);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::child_selection => {
                SelectorLength::WithLength(Pair::try_into(inner.into_inner().next().unwrap())?)
            }
            Rule::path_selection => {}
            Rule::selector => SelectorLength::NoLength(Pair::try_into(inner)?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Selection<'a> {
    OverSlices(Tagged<'a, BaseSelection<'a>>, Tagged<'a, RsSlices<'a>>),
    OverPairs(Tagged<'a, BaseSelection<'a>>, Tagged<'a, RsPairs<'a>>),
    Basic(Tagged<'a, BaseSelection<'a>>),
}

impl_fandango_traverse!(Selection, match { OverSlices(basic, slices), OverPairs(basic, pairs), Basic(basic) });

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum BaseSelection<'a> {
    Nonterminal(Tagged<'a, Nonterminal<'a>>),
    Selector(Box<Tagged<'a, Selector<'a>>>),
}

impl<'program, 'source: 'program> GraphTraverse<'program> for &'program BaseSelection<'source> {
    type Node = FandangoNode<'program, 'source>;

    fn traverse<F>(self, consumer: F)
    where
        F: FnMut(Self::Node, Self::Node, Span<'program>),
    {
        #![allow(unused_imports)]
        use BaseSelection::*;
        match self {
            Nonterminal(nonterminal) => {
                traverse_children(self, iter::once(nonterminal).map(From::from), consumer)
            }
            Selector(selector) => {
                traverse_children(self, iter::once(selector.deref()).map(From::from), consumer)
            }
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RsPairs<'a> {
    pairs: Vec<Tagged<'a, RsPair<'a>>>,
}

impl_fandango_traverse!(RsPairs, [pairs]);

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RsPair<'a> {
    nonterminal: Tagged<'a, Nonterminal<'a>>,
    slice: Option<Tagged<'a, RsSlice>>,
}

impl_fandango_traverse!(RsPair, nonterminal, [slice]);

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RsSlices<'a> {
    slices: Vec<Tagged<'a, RsSlice>>,
}

impl_fandango_traverse!(RsSlices, [slices]);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RsSlice {
    RangeWithStep(Option<usize>, Option<usize>, Option<usize>),
    Range(Option<usize>, Option<usize>),
    Exact(usize),
}

impl Display for RsSlice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RsSlice::RangeWithStep(start, end, step) => {
                if let Some(start) = start {
                    Display::fmt(start, f)?;
                }
                f.write_char(':')?;
                if let Some(end) = end {
                    Display::fmt(end, f)?;
                }
                f.write_char(':')?;
                if let Some(step) = step {
                    Display::fmt(step, f)?;
                }
            }
            RsSlice::Range(start, end) => {
                if let Some(start) = start {
                    Display::fmt(start, f)?;
                }
                f.write_char(':')?;
                if let Some(end) = end {
                    Display::fmt(end, f)?;
                }
            }
            RsSlice::Exact(n) => Display::fmt(n, f)?,
        }
        Ok(())
    }
}
