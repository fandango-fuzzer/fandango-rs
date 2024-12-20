//! Parsing and representation routines for constraints defined within a FANDANGO grammar.

use crate::graph::{traverse_children, FandangoNode, GraphTraverse};
use crate::lang::{Nonterminal, ParseError, Rule, Tagged};
use pest::iterators::Pair;
use pest::Span;
use std::fmt::{Debug, Display, Formatter, Write};
use std::iter;
use std::ops::Deref;

/// Represents a constraint statement in a FANDANGO grammar.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Constraint<'a> {
    /// A constraint which expresses a fitness
    Fitness(Tagged<'a, Expr<'a>>),
    /// A constraint which expresses an implication
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

/// An implication which defines the constraint
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
        let implies = if let Some(implies) = inner.next() {
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

/// A quantifier for the constraint
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Quantifier<'a> {
    /// A statement over which all must hold
    Forall(Tagged<'a, QuantifierSpecification<'a>>),
    /// A statement over which at least one must hold
    Exists(Tagged<'a, QuantifierSpecification<'a>>),
    /// A specific, first-order statement that must hold
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

/// The specification for the quantifier (i.e., what must hold)
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
            quantifier: Box::new(Pair::try_into(inner.next().unwrap())?),
        })
    }
}

/// A sequence of formula atoms of which at least one must hold for this to be considered true
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Disjunction<'a> {
    conjunctions: Vec<Tagged<'a, Conjunction<'a>>>,
}

impl_fandango_traverse!(Disjunction, [conjunctions]);

impl<'a> TryFrom<Pair<'a, Rule>> for Disjunction<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::formula_disjunction);

        let inner = value.into_inner();

        Ok(Self {
            conjunctions: inner
                .map(Pair::try_into)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

/// A sequence of formula atoms which must all hold for this to be considered true
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Conjunction<'a> {
    atoms: Vec<Tagged<'a, Atom<'a>>>,
}

impl_fandango_traverse!(Conjunction, [atoms]);

impl<'a> TryFrom<Pair<'a, Rule>> for Conjunction<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::formula_conjunction);

        let inner = value.into_inner();

        Ok(Self {
            atoms: inner
                .map(Pair::try_into)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

/// The smallest unit of a formula
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Atom<'a> {
    /// A comparison within the formula
    Comparison(Tagged<'a, Comparison<'a>>),
    /// An implies statement, which must hold for the formula
    Implies(Tagged<'a, Implies<'a>>), // no indirection needed, we are in a Vec
    /// The expression to be evaluated in the formula
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

/// A comparison between two expressions
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

/// An operator specified in a constraint
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum ConstraintOperator {
    Neq,
    Lt,
    LtEq,
    Eq,
    GtEq,
    Gt,
}

impl<'a> TryFrom<Pair<'a, Rule>> for ConstraintOperator {
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

/// An expression, either represented by a concrete selector or by an expression which needs to be
/// evaluated
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Expr<'a> {
    /// A direct selector
    Selector(Tagged<'a, SelectorLength<'a>>),
    // ConstraintIte(Tagged<'a, ConstraintIte<'a>>),
    /// An expression to be evaluated
    Inversion(Tagged<'a, Inversion<'a>>),
}

impl_fandango_traverse!(Expr, match { Selector(selector), Inversion(inversion) });

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

/// A selector, maybe specifying the length over a value or the value itself
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum SelectorLength<'a> {
    /// Specifies a selection that is considering the length of the selection
    WithLength(Tagged<'a, Selector<'a>>),
    /// Specifies a selection that is considering the value itself
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

/// An expression which evaluates to a selected node
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Selector<'a> {
    /// A selection which specifies any direct child of the second type from the first
    ChildSelector(Tagged<'a, Selection<'a>>, Box<Tagged<'a, Selector<'a>>>),
    /// A selection which specifies any descendent of the second type from the first
    PathSelector(Tagged<'a, Selection<'a>>, Box<Tagged<'a, Selector<'a>>>),
    /// A basic selection
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
                let mut inner = inner.into_inner();
                Selector::ChildSelector(
                    Pair::try_into(inner.next().unwrap())?,
                    Box::new(Pair::try_into(inner.next().unwrap())?),
                )
            }
            Rule::path_selection => {
                let mut inner = inner.into_inner();
                Selector::PathSelector(
                    Pair::try_into(inner.next().unwrap())?,
                    Box::new(Pair::try_into(inner.next().unwrap())?),
                )
            }
            Rule::selection => Selector::Basic(Pair::try_into(inner)?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

/// A particular selection over which a constraint is evaluated
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Selection<'a> {
    /// A selection over a sequence of slices
    OverSlices(Tagged<'a, BaseSelection<'a>>, Tagged<'a, RsSlices<'a>>),
    /// A selection over a sequence of pairs
    OverPairs(Tagged<'a, BaseSelection<'a>>, Tagged<'a, RsPairs<'a>>),
    /// A basic selection
    Basic(Tagged<'a, BaseSelection<'a>>),
}

impl_fandango_traverse!(Selection, match { OverSlices(basic, slices), OverPairs(basic, pairs), Basic(basic) });

impl<'a> TryFrom<Pair<'a, Rule>> for Selection<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::selection);

        let mut inner = value.into_inner();
        let base = inner.next().unwrap();

        Ok(match inner.next() {
            Some(inner) => match inner.as_rule() {
                Rule::rs_slices => {
                    Selection::OverSlices(Pair::try_into(base)?, Pair::try_into(inner)?)
                }
                Rule::rs_pairs => {
                    Selection::OverPairs(Pair::try_into(base)?, Pair::try_into(inner)?)
                }
                _ => unreachable!("This case is not represented within the grammar."),
            },
            None => Selection::Basic(Pair::try_into(base)?),
        })
    }
}

/// A base selection: either a nonterminal (specific), or another selection by recursion
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum BaseSelection<'a> {
    /// Concrete non-terminal to be selected
    Nonterminal(Tagged<'a, Nonterminal<'a>>),
    /// Specify another selector which defines this selection
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

impl<'a> TryFrom<Pair<'a, Rule>> for BaseSelection<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::base_selection);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::nonterminal => BaseSelection::Nonterminal(Pair::try_into(inner)?),
            Rule::selector => BaseSelection::Selector(Box::new(Pair::try_into(inner)?)),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

/// A sequence of pairs over a node
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RsPairs<'a> {
    pairs: Vec<Tagged<'a, RsPair<'a>>>,
}

impl_fandango_traverse!(RsPairs, [pairs]);

impl<'a> TryFrom<Pair<'a, Rule>> for RsPairs<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::rs_pairs);

        let inner = value.into_inner();

        Ok(Self {
            pairs: inner
                .map(Pair::try_into)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

/// TODO ask what this means :)
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RsPair<'a> {
    nonterminal: Tagged<'a, Nonterminal<'a>>,
    slice: Option<Tagged<'a, RsSlice>>,
}

impl_fandango_traverse!(RsPair, nonterminal, [slice]);

impl<'a> TryFrom<Pair<'a, Rule>> for RsPair<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::rs_pair);

        let mut inner = value.into_inner();

        Ok(Self {
            nonterminal: Pair::try_into(inner.next().unwrap())?,
            slice: if let Some(slice) = inner.next() {
                Some(Pair::try_into(slice)?)
            } else {
                None
            },
        })
    }
}

/// A sequence of slices
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RsSlices<'a> {
    slices: Vec<Tagged<'a, RsSlice>>,
}

impl_fandango_traverse!(RsSlices, [slices]);

impl<'a> TryFrom<Pair<'a, Rule>> for RsSlices<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::rs_slices);

        let inner = value.into_inner();

        Ok(Self {
            slices: inner
                .map(Pair::try_into)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

/// A slice access over a node
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RsSlice {
    /// Extract a range from this sequence with a particular step size, possibly unbounded/unspecified for each
    RangeWithStep(Option<usize>, Option<usize>, Option<usize>),
    /// Extract a range from this sequence, potentially unbounded at either or both ends
    Range(Option<usize>, Option<usize>),
    /// Access an exact member of this sequence
    Exact(usize),
}

impl<'a> TryFrom<Pair<'a, Rule>> for RsSlice {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::rs_slice);

        let mut inner = value.into_inner();

        let first = inner.next().unwrap();
        Ok(match first.as_rule() {
            Rule::rs_slice_range_step => {
                let mut range = first.into_inner();
                let start = range.next().unwrap();
                debug_assert_eq!(start.as_rule(), Rule::rs_slice_start);
                let end = range.next().unwrap();
                debug_assert_eq!(end.as_rule(), Rule::rs_slice_start);
                Self::RangeWithStep(
                    start
                        .into_inner()
                        .next()
                        .map(|p| p.as_str().parse().unwrap()),
                    end.into_inner().next().map(|p| p.as_str().parse().unwrap()),
                    inner.next().map(|p| p.as_str().parse().unwrap()),
                )
            }
            Rule::rs_slice_range => {
                let mut start = first.into_inner();
                let start = start.next().unwrap();
                debug_assert_eq!(start.as_rule(), Rule::rs_slice_start);
                Self::Range(
                    start
                        .into_inner()
                        .next()
                        .map(|p| p.as_str().parse().unwrap()),
                    inner.next().map(|p| p.as_str().parse().unwrap()),
                )
            }
            Rule::integer => Self::Exact(first.as_str().parse().unwrap()),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

/// A single expression to be evaluated for the purposes of evaluating a constraint
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Inversion<'a> {
    /// Represents that a particular selector is evaluated directly
    Selector(Tagged<'a, Selector<'a>>),
    /// Represents that a particular selection should be stringified before evaluation
    Stringified(Tagged<'a, Selector<'a>>),
}

impl_fandango_traverse!(Inversion, match { Selector(selector), Stringified(selector) });

impl<'a> TryFrom<Pair<'a, Rule>> for Inversion<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::inversion);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::stringified => {
                Inversion::Stringified(Pair::try_into(inner.into_inner().next().unwrap())?)
            }
            Rule::selector => Inversion::Selector(Pair::try_into(inner)?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
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
