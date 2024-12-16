use crate::lang::{Nonterminal, Tagged};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Constraint<'a> {
    Fitness(Tagged<'a, Expr<'a>>),
    Implies(Tagged<'a, Implies<'a>>),
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Implies<'a> {
    quantifier: Tagged<'a, Quantifier<'a>>,
    implies: Option<Box<Tagged<'a, Implies<'a>>>>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Quantifier<'a> {
    Forall(Tagged<'a, QuantifierSpecification<'a>>),
    Exists(Tagged<'a, QuantifierSpecification<'a>>),
    Disjunction(Tagged<'a, Disjunction<'a>>),
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct QuantifierSpecification<'a> {
    nonterminal: Tagged<'a, Nonterminal<'a>>,
    selector: Tagged<'a, Selector<'a>>,
    quantifier: Box<Tagged<'a, Quantifier<'a>>>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Disjunction<'a> {
    conjunctions: Vec<Tagged<'a, Conjunction<'a>>>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Conjunction<'a> {
    atoms: Vec<Tagged<'a, Atom<'a>>>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Atom<'a> {
    Comparison(Tagged<'a, Comparison<'a>>),
    Implies(Tagged<'a, Implies<'a>>), // no indirection needed, we are in a Vec
    Expr(Tagged<'a, Expr<'a>>),
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Comparison<'a> {
    left: Tagged<'a, Expr<'a>>,
    right: Tagged<'a, Expr<'a>>,
    operator: Tagged<'a, ConstraintOperator>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ConstraintOperator {
    Neq,
    Lt,
    LtEq,
    Eq,
    GtEq,
    Gt,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Expr<'a> {
    SelectorLength(Tagged<'a, SelectorLength<'a>>),
    ConstraintIte(Tagged<'a, ConstraintIte<'a>>),
    Inversion(Tagged<'a, Inversion<'a>>),
}
