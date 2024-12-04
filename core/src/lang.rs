//! Language definition for FANDANGO grammars. [`Program::try_from`] is what you want. :)

use crate::graph::{FandangoNode, Traverse};
use crate::impl_traverse;
use crate::lang::py_literal::parse_string;
use getset::Getters;
use pest::Parser;
use pest::error::{Error as PestError, ErrorVariant};
use pest::iterators::Pair;
use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut, RangeInclusive};
use std::str::FromStr;

pub use pest::Span;

mod parser {
    #![allow(missing_docs)]

    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "py_literal/grammar.pest"]
    #[grammar = "fandango.pest"]
    pub struct Fandango;
}

use parser::Fandango;
pub use parser::Rule;

/// The [`PestError`] specific to FANDANGO.
pub type ParseError = Box<PestError<Rule>>;

/// A source position tag for a given grammar element.
#[derive(Debug, Clone, Eq, PartialEq, Getters)]
pub struct Tagged<'source, T> {
    #[getset(get_copy = "pub")]
    span: Span<'source>,
    inner: T,
}

impl<'source, T> Tagged<'source, T> {
    pub(crate) fn new(inner: T, span: Span<'source>) -> Self {
        Self { inner, span }
    }
}

impl<T> Deref for Tagged<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Tagged<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'source, T> TryFrom<Pair<'source, Rule>> for Tagged<'source, T>
where
    T: TryFrom<Pair<'source, Rule>>,
{
    type Error = T::Error;

    fn try_from(value: Pair<'source, Rule>) -> Result<Self, Self::Error> {
        let span = value.as_span();
        Ok(Self {
            span,
            inner: value.try_into()?,
        })
    }
}

impl<'program, 'source, T, U> From<&'program Tagged<'source, T>> for (U, Span<'source>)
where
    U: From<&'program T>,
{
    fn from(value: &'program Tagged<'source, T>) -> Self {
        ((&value.inner).into(), value.span)
    }
}

impl<T> PartialEq<T> for Tagged<'_, T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.inner == *other
    }
}

/// The root of the FANDANGO grammar.
#[derive(Debug, Clone, Eq, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Program<'a> {
    /// The statements contained within this grammar.
    statements: Vec<Tagged<'a, Statement<'a>>>,
}

impl_fandango_traverse!(Program, [statements]);

impl<'a> TryFrom<&'a str> for Tagged<'a, Program<'a>> {
    type Error = ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (grammar,) =
            parse_pairs_as!(Fandango::parse(Rule::fandango, value)?, (Rule::fandango,));
        let (program, _) = parse_pairs_as!(grammar.into_inner(), (Rule::program, Rule::EOI));

        Ok(Tagged {
            span: program.as_span(),
            inner: Program::try_from(program)?,
        })
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Program<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::program);

        Ok(Self {
            statements: value
                .into_inner()
                .map(Pair::try_into)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

/// A statement within a FANDANGO grammar.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement<'a> {
    /// A production representing a rule within the grammar.
    Production(Tagged<'a, Production<'a>>),
    /// A constraint applied within the grammar.
    Constraint,
    /// Python code present in the grammar for the definition of e.g. generators and constraints.
    Python,
}

impl_fandango_traverse!(Statement, match { Production(prod), Constraint, Python });

impl<'a> TryFrom<Pair<'a, Rule>> for Statement<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::statement);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::production => Statement::Production(Pair::try_into(inner)?),
            Rule::constraint => todo!("Constraints are not yet implemented"),
            Rule::python => todo!("Python parsing is not yet implemented"),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

/// A production rule within the grammar.
#[derive(Debug, Clone, Eq, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Production<'a> {
    /// The nonterminal which is defined by this production.
    nonterminal: Tagged<'a, Nonterminal<'a>>,
    /// An alternative which defines the rule.
    alternative: Tagged<'a, Alternative<'a>>,
}

impl_fandango_traverse!(Production, nonterminal, alternative);

impl<'a> TryFrom<Pair<'a, Rule>> for Production<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::production);

        let (nonterminal, alternative) =
            parse_pairs_as!(value.into_inner(), (Rule::nonterminal, Rule::alternative));

        Ok(Self {
            nonterminal: nonterminal.try_into()?,
            alternative: alternative.try_into()?,
        })
    }
}

/// A non-terminal, either at definition or use site.
#[derive(Debug, Clone, Eq, Ord, PartialOrd, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Nonterminal<'a> {
    /// The name of the non-terminal.
    name: Cow<'a, str>,
}

impl<'a> Nonterminal<'a> {
    /// Create a non-terminal (useful for testing and referring to non-terminals directly).
    pub fn new(name: Cow<'a, str>) -> Self {
        Self { name }
    }
}

impl<'program, 'source> Traverse<'source> for &'program Nonterminal<'source> {
    type Node = FandangoNode<'program, 'source>;
}

impl<'a> TryFrom<Pair<'a, Rule>> for Nonterminal<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::nonterminal);

        let (name,) = parse_pairs_as!(value.into_inner(), (Rule::name,));

        Ok(Self {
            name: Cow::Borrowed(name.as_str()),
        })
    }
}

/// A list of potential instantiations.
#[derive(Debug, Clone, Eq, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Alternative<'a> {
    /// The concatenations which represent the possible alternatives.
    concatenations: Vec<Tagged<'a, Concatenation<'a>>>,
}

impl_fandango_traverse!(Alternative, [concatenations]);

impl<'a> TryFrom<Pair<'a, Rule>> for Alternative<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::alternative);

        Ok(Self {
            concatenations: value
                .into_inner()
                .map(Pair::try_into)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

/// A concatenation of individual operators.
#[derive(Debug, Clone, Eq, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Concatenation<'a> {
    /// The concatenated operators.
    operators: Vec<Tagged<'a, Operator<'a>>>,
}

impl_fandango_traverse!(Concatenation, [operators]);

impl<'a> TryFrom<Pair<'a, Rule>> for Concatenation<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::concatenation);

        Ok(Self {
            operators: value
                .into_inner()
                .map(Pair::try_into)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

/// An individual operator within a grammar.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operator<'a> {
    /// Kleene star (0 to many) postfix operation.
    Kleene(Tagged<'a, Symbol<'a>>),
    /// Plus (1 to many) postfix operation.
    Plus(Tagged<'a, Symbol<'a>>),
    /// Optional (0 or 1) postfix operation.
    Option(Tagged<'a, Symbol<'a>>),
    /// Repetition postfix operation, with range specified as `{n}` for exactly `n` repetitions or
    /// `{m,n}` for any number of repetitions between `m` and `n`, inclusive.
    Repeat(Tagged<'a, Symbol<'a>>, RangeInclusive<usize>),
    /// Simple case: exactly 1 [`Symbol`].
    Symbol(Tagged<'a, Symbol<'a>>),
}

impl_fandango_traverse!(Operator, match { Kleene(sym), Plus(sym), Option(sym), Repeat(sym, _), Symbol(sym) });

fn parse_range(pair: Pair<Rule>) -> Result<usize, ParseError> {
    usize::from_str(pair.as_str()).map_err(|_| {
        Box::new(PestError::new_from_span(
            ErrorVariant::CustomError {
                message: "invalid range specifier".to_string(),
            },
            pair.as_span(),
        ))
    })
}

impl<'a> TryFrom<Pair<'a, Rule>> for Operator<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::operator);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::kleene => Operator::Kleene(inner.into_inner().next().unwrap().try_into()?),
            Rule::plus => Operator::Plus(inner.into_inner().next().unwrap().try_into()?),
            Rule::option => Operator::Option(inner.into_inner().next().unwrap().try_into()?),
            Rule::repeat => {
                let mut pairs = inner.into_inner();
                let symbol = pairs.next().unwrap().try_into()?;
                let range_start = parse_range(pairs.next().unwrap())?;
                let range_end = pairs.next().map_or(Ok(range_start), parse_range)?;
                Operator::Repeat(symbol, range_start..=range_end)
            }
            Rule::symbol => Operator::Symbol(inner.try_into()?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

/// A single symbol within the grammar, or a list of [`Alternative`]s.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Symbol<'a> {
    /// A single non-terminal.
    Nonterminal(Tagged<'a, Nonterminal<'a>>),
    /// A string-like terminal.
    String(Tagged<'a, Cow<'a, str>>),
    /// A list of [`Alternative`]s.
    Alternative(Tagged<'a, Alternative<'a>>),
}

impl_fandango_traverse!(Symbol, match { Nonterminal(nt), String(s), Alternative(alt) });

impl<'a> TryFrom<Pair<'a, Rule>> for Symbol<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::symbol);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::nonterminal => Symbol::Nonterminal(inner.try_into()?),
            Rule::string => Symbol::String(parse_string(inner)?),
            Rule::alternative => Symbol::Alternative(inner.try_into()?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

/// This section is mostly copied from py_literal: <https://github.com/jturner314/py_literal/releases/tag/0.4.0>
/// This is necessary because pest does not easily allow for grammar + extract dependencies.
mod py_literal {
    use crate::lang::{ParseError, Rule, Tagged};
    use alloc::borrow::Cow;
    use pest::error::{Error as PestError, ErrorVariant};
    use pest::iterators::Pair;

    fn parse_string_escape_seq(escape_seq: Pair<'_, Rule>) -> Result<char, ParseError> {
        debug_assert_eq!(escape_seq.as_rule(), Rule::string_escape_seq);
        let (seq,) = parse_pairs_as!(escape_seq.into_inner(), (_,));
        match seq.as_rule() {
            Rule::char_escape => Ok(match seq.as_str() {
                "\\" => '\\',
                "'" => '\'',
                "\"" => '"',
                "a" => '\x07',
                "b" => '\x08',
                "f" => '\x0C',
                "n" => '\n',
                "r" => '\r',
                "t" => '\t',
                "v" => '\x0B',
                _ => unreachable!(),
            }),
            Rule::octal_escape => ::std::char::from_u32(
                u32::from_str_radix(seq.as_str(), 8).unwrap(),
            )
            .ok_or_else(|| {
                Box::new(PestError::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("Octal escape is invalid: \\{}", seq.as_str()),
                    },
                    seq.as_span(),
                ))
            }),
            Rule::hex_escape | Rule::unicode_hex_escape => {
                ::std::char::from_u32(u32::from_str_radix(&seq.as_str()[1..], 16).unwrap())
                    .ok_or_else(|| {
                        Box::new(PestError::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!("Hex escape is invalid: \\x{}", seq.as_str()),
                            },
                            seq.as_span(),
                        ))
                    })
            }
            Rule::name_escape => Err(Box::new(PestError::new_from_span(
                ErrorVariant::CustomError {
                    message: "Unicode name escapes are not supported.".into(),
                },
                seq.as_span(),
            ))),
            _ => unreachable!(),
        }
    }

    pub fn parse_string(string: Pair<Rule>) -> Result<Tagged<Cow<str>>, ParseError> {
        debug_assert_eq!(string.as_rule(), Rule::string);
        let (string_body,) = parse_pairs_as!(string.into_inner(), (_,));
        match string_body.as_rule() {
            Rule::short_string_body | Rule::long_string_body => {
                let mut out = String::new();
                let orig = string_body.as_str();
                let span = string_body.as_span();
                for item in string_body.into_inner() {
                    match item.as_rule() {
                        Rule::short_string_non_escape
                        | Rule::long_string_non_escape
                        | Rule::string_unknown_escape => out.push_str(item.as_str()),
                        Rule::line_continuation_seq => (),
                        Rule::string_escape_seq => out.push(parse_string_escape_seq(item)?),
                        _ => unreachable!(),
                    }
                }
                // escapes always increase length
                if orig.len() == out.len() {
                    Ok(Tagged::new(Cow::Borrowed(orig), span))
                } else {
                    Ok(Tagged::new(Cow::Owned(out), span))
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::lang::parser::Fandango;
    use crate::lang::{
        Alternative, Concatenation, Nonterminal, Operator, Production, Program, Rule, Statement,
        Symbol, Tagged, parse_string,
    };
    use alloc::borrow::Cow;
    use alloc::boxed::Box;
    use pest::Parser;
    use pest::iterators::Pair;
    use std::error::Error;

    fn check_string_operator<const BORROW: bool>(operator: Pair<'_, Rule>, expected: &str) {
        let (symbol,) = parse_pairs_as!(operator.into_inner(), (Rule::symbol,));
        let (string,) = parse_pairs_as!(symbol.into_inner(), (Rule::string,));
        let actual = parse_string(string).expect("Expected valid string");
        assert_eq!(matches!(&*actual, Cow::Borrowed(_)), BORROW);
        assert_eq!(&*actual, expected);
    }

    fn check_nonterminal_symbol(symbol: Pair<'_, Rule>, expected: &str) {
        let (nonterminal,) = parse_pairs_as!(symbol.into_inner(), (Rule::nonterminal,));
        let (name,) = parse_pairs_as!(nonterminal.into_inner(), (Rule::name,));
        assert_eq!(name.as_str(), expected);
    }

    fn check_nonterminal_operator(operator: Pair<'_, Rule>, expected: &str) {
        let (symbol,) = parse_pairs_as!(operator.into_inner(), (Rule::symbol,));
        check_nonterminal_symbol(symbol, expected);
    }

    fn check_production<F: Fn(Pair<'_, Rule>)>(
        production: Pair<'_, Rule>,
        expected: &str,
        checker: F,
    ) {
        let (nonterminal, alternative) = parse_pairs_as!(
            production.into_inner(),
            (Rule::nonterminal, Rule::alternative)
        );

        let (name,) = parse_pairs_as!(nonterminal.into_inner(), (Rule::name,));
        assert_eq!(name.as_str(), expected);

        checker(alternative);
    }

    pub const SIMPLE_GRAMMAR: &str = include_str!("../../tests/grammars/simple.fan");

    #[test]
    fn test_grammar() -> Result<(), Box<dyn Error>> {
        let (grammar,) = parse_pairs_as!(
            Fandango::parse(Rule::fandango, SIMPLE_GRAMMAR)?,
            (Rule::fandango,)
        );
        let (program, _) = parse_pairs_as!(grammar.into_inner(), (Rule::program, Rule::EOI));

        let [start, expr, number, non_zero, digit] =
            program.into_inner().collect::<Vec<_>>().try_into().unwrap();

        // start rule
        {
            let (production,) = parse_pairs_as!(start.into_inner(), (Rule::production,));
            check_production(production, "start", |alternative| {
                let (concatenation,) =
                    parse_pairs_as!(alternative.into_inner(), (Rule::concatenation,));
                let (operator,) = parse_pairs_as!(concatenation.into_inner(), (Rule::operator,));
                check_nonterminal_operator(operator, "expr");
            });
        }

        // expr rule
        {
            let (production,) = parse_pairs_as!(expr.into_inner(), (Rule::production,));
            check_production(production, "expr", |alternative| {
                let (addition, number) = parse_pairs_as!(
                    alternative.into_inner(),
                    (Rule::concatenation, Rule::concatenation)
                );
                let (operator,) = parse_pairs_as!(number.into_inner(), (Rule::operator,));
                check_nonterminal_operator(operator, "number");

                let (number, plus, expr) = parse_pairs_as!(
                    addition.into_inner(),
                    (Rule::operator, Rule::operator, Rule::operator)
                );
                check_nonterminal_operator(number, "number");
                check_string_operator::<true>(plus, "+");
                check_nonterminal_operator(expr, "expr");
            });
        }

        // number rule
        {
            let (production,) = parse_pairs_as!(number.into_inner(), (Rule::production,));
            check_production(production, "number", |alternative| {
                let (zero, number) = parse_pairs_as!(
                    alternative.into_inner(),
                    (Rule::concatenation, Rule::concatenation)
                );

                let (non_zero, digit_star) =
                    parse_pairs_as!(number.into_inner(), (Rule::operator, Rule::operator));
                check_nonterminal_operator(non_zero, "non_zero");
                let (digit_star,) = parse_pairs_as!(digit_star.into_inner(), (Rule::kleene,));
                let (digit,) = parse_pairs_as!(digit_star.into_inner(), (Rule::symbol,));
                check_nonterminal_symbol(digit, "digit");

                let (zero,) = parse_pairs_as!(zero.into_inner(), (Rule::operator,));
                check_string_operator::<true>(zero, "0");
            });
        }

        // non_zero rule
        {
            let (production,) = parse_pairs_as!(non_zero.into_inner(), (Rule::production,));
            check_production(production, "non_zero", |alternative| {
                let mut concats = alternative.into_inner();
                for expected in 1..=9u8 {
                    let concatenation = concats
                        .next()
                        .expect("should not have exhausted alternative");
                    let (operator,) =
                        parse_pairs_as!(concatenation.into_inner(), (Rule::operator,));
                    check_string_operator::<true>(operator, &format!("{expected}"));
                }
                assert_eq!(concats.next(), None);
            });
        }

        // digit rule
        {
            let (production,) = parse_pairs_as!(digit.into_inner(), (Rule::production,));
            check_production(production, "digit", |alternative| {
                let (zero, non_zero) = parse_pairs_as!(
                    alternative.into_inner(),
                    (Rule::concatenation, Rule::concatenation)
                );
                let (operator,) = parse_pairs_as!(zero.into_inner(), (Rule::operator,));
                check_string_operator::<true>(operator, "0");
                let (operator,) = parse_pairs_as!(non_zero.into_inner(), (Rule::operator,));
                check_nonterminal_operator(operator, "non_zero");
            });
        }

        Ok(())
    }

    macro_rules! untag_or_die {
        ($value:ident, $etype:tt :: $variant:tt ( $($names:ident),+ )) => {{
            let $etype :: $variant ($($names),+) = $value.inner else {
                panic!("Invalid untagging.");
            };
            ($($names),+)
        }};

        ($value:ident, $stype:tt { $($names:ident),+ }) => {{
            let $stype { $($names),+ } = $value.inner;
            ($($names),+)
        }};
    }

    #[test]
    fn test_fullparse() -> Result<(), Box<dyn Error>> {
        let program = Tagged::<Program>::try_from(SIMPLE_GRAMMAR)?;

        let [start, expr, number, non_zero, digit] = program.inner.statements.try_into().unwrap();

        let prod = untag_or_die!(start, Statement::Production(prod));
        let (nt, alt) = untag_or_die!(prod, Production {
            nonterminal,
            alternative
        });

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "start");

        let [concat] = untag_or_die!(alt, Alternative { concatenations })
            .try_into()
            .unwrap();
        let [op] = untag_or_die!(concat, Concatenation { operators })
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "expr");

        let prod = untag_or_die!(expr, Statement::Production(prod));
        let (nt, alt) = untag_or_die!(prod, Production {
            nonterminal,
            alternative
        });

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "expr");

        let [c1, c2] = untag_or_die!(alt, Alternative { concatenations })
            .try_into()
            .unwrap();

        let [op1, op2, op3] = untag_or_die!(c1, Concatenation { operators })
            .try_into()
            .unwrap();

        let sym = untag_or_die!(op1, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "number");

        let sym = untag_or_die!(op2, Operator::Symbol(nt));
        let value = untag_or_die!(sym, Symbol::String(sym));
        assert_eq!(value.inner, "+");

        let sym = untag_or_die!(op3, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "expr");

        let [op] = untag_or_die!(c2, Concatenation { operators })
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "number");

        let prod = untag_or_die!(number, Statement::Production(prod));
        let (nt, alt) = untag_or_die!(prod, Production {
            nonterminal,
            alternative
        });

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "number");

        let [c1, c2] = untag_or_die!(alt, Alternative { concatenations })
            .try_into()
            .unwrap();

        let [op] = untag_or_die!(c1, Concatenation { operators })
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let value = untag_or_die!(sym, Symbol::String(sym));
        assert_eq!(value.inner, "0");

        let [op1, op2] = untag_or_die!(c2, Concatenation { operators })
            .try_into()
            .unwrap();

        let sym = untag_or_die!(op1, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "non_zero");

        let sym = untag_or_die!(op2, Operator::Kleene(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "digit");

        let prod = untag_or_die!(non_zero, Statement::Production(prod));
        let (nt, alt) = untag_or_die!(prod, Production {
            nonterminal,
            alternative
        });

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "non_zero");

        let concats = untag_or_die!(alt, Alternative { concatenations });

        for (concat, i) in concats.into_iter().zip(1..=9u8) {
            let [op] = untag_or_die!(concat, Concatenation { operators })
                .try_into()
                .unwrap();
            let sym = untag_or_die!(op, Operator::Symbol(nt));
            let value = untag_or_die!(sym, Symbol::String(sym));
            assert_eq!(value.inner, format!("{}", i));
        }

        let prod = untag_or_die!(digit, Statement::Production(prod));
        let (nt, alt) = untag_or_die!(prod, Production {
            nonterminal,
            alternative
        });

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "digit");

        let [c1, c2] = untag_or_die!(alt, Alternative { concatenations })
            .try_into()
            .unwrap();

        let [op] = untag_or_die!(c1, Concatenation { operators })
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let value = untag_or_die!(sym, Symbol::String(sym));
        assert_eq!(value.inner, "0");

        let [op] = untag_or_die!(c2, Concatenation { operators })
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "non_zero");

        Ok(())
    }
}
