use std::borrow::Cow;
use std::convert::Infallible;
use std::marker::PhantomData;
use std::rc::Rc;

pub enum SolveState<T> {
    True,
    False,
    Subgoal(T),
}

pub trait ConstraintType {
    type Evaluated;
}

pub struct BooleanGoal(Infallible);
impl ConstraintType for BooleanGoal {
    type Evaluated = bool;
}

pub trait StringGoal {
    type Inner: NodeGoal;
}
impl<S> ConstraintType for S
where
    S: StringGoal,
{
    type Evaluated = Rc<Cow<'static, str>>;
}

pub trait NodeGoal {
    type Type;
}
impl<N> ConstraintType for N
where
    N: NodeGoal,
{
    type Evaluated = N::Type;
}

pub trait Goal {
    type Type: ConstraintType;

    fn evaluate(&self) -> <Self::Type as ConstraintType>::Evaluated;
}

pub struct Eq<G> {
    first: G,
    second: G,
}

impl<G> Goal for Eq<G>
where
    G: Goal,
    G::Type: StringGoal,
{
    type Type = BooleanGoal;

    fn evaluate(&self) -> <Self::Type as ConstraintType>::Evaluated {
        todo!()
    }
}

impl<G> Goal for Eq<G>
where
    G: Goal,
    G::Type: NodeGoal,
{
    type Type = BooleanGoal;
}

pub struct And<L> {
    operands: L,
}

impl<Tail> Goal for And<(BooleanGoal, Tail)>
where
    And<Tail>: Goal,
{
    type Type = BooleanGoal;
}

impl Goal for And<()> {
    type Type = BooleanGoal;
}

pub struct Or<L> {
    operands: L,
}

impl<Tail> Goal for Or<BooleanGoal, Tail> where Or<Tail>: Goal {}

pub trait Rewrite {
    type Rewritten;

    fn rewrite(self) -> Result<Self::Rewritten, Self>;
}

pub trait TransitiveRewrite<T>: Rewrite
where
    Self::Rewritten: TransitiveRewrite<T>,
{
    fn rewrite_then(self, task: T);
}
