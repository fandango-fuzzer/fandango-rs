//! Language definition for FANDANGO grammars. [`Program::try_from`] is what you want. :)

pub mod constraints;

use crate::lang::py_literal::{parse_bytes, parse_string};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::cmp::Ordering;
use core::fmt::{Debug, Display, Formatter, Write};
use core::hash::Hash;
use core::hash::Hasher;
use core::str::FromStr;
use pest::error::{Error as PestError, ErrorVariant};
use pest::iterators::Pair;
use pest::Parser;

#[allow(deprecated)]
use core::hash::SipHasher;

use alloc::format;
use core::fmt;
pub use pest::Span;

mod parser {
    #![allow(missing_docs)]

    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "py_literal/grammar.pest"]
    #[grammar = "fandango.pest"]
    pub struct Fandango;
}

use parser::{Fandango, Rule};

use crate::lang::constraints::{
    Atom, BaseSelection, Comparison, Conjunction, Constraint, ConstraintOperator, Disjunction,
    Expr, Implies, Inversion, Quantifier, QuantifierSpecification, RsPair, RsPairs, RsSlice,
    RsSlices, Selection, Selector, SelectorLength,
};

/// The [`PestError`] specific to FANDANGO.
pub type ParseError = Box<PestError<Rule>>;

/// A source position tag for a given grammar element, plus some extras for performance.
#[derive(Clone)]
pub struct Tagged<'source, T> {
    source: &'source str,
    span: (usize, usize),
    inner: T,
    hash: u64,
}

impl<T> Debug for Tagged<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Tagged")
            .field("inner", &self.inner)
            .field("span", &self.span().as_str())
            .finish()
    }
}

/// Pre-compute the hash for a given [`Tagged`] instance. Only to be used for known hashes during
/// generation; your mileage may vary.
#[allow(deprecated)]
pub fn compute_tag_hash(span: &Span<'_>) -> u64 {
    let mut hasher = SipHasher::new_with_keys(0, 0);
    span.as_str().hash(&mut hasher);
    hasher.finish()
}

impl<'source, T> Tagged<'source, T> {
    /// Create a new [`Tagged`] instance, associating the provided [`Span`] with the node.
    pub fn new(inner: T, span: Span<'source>) -> Self
    where
        T: Hash,
    {
        Self {
            inner,
            source: span.get_input(),
            span: (span.start(), span.end()),
            hash: compute_tag_hash(&span),
        }
    }

    /// The [`Span`] associated with the inner node.
    pub fn span(&self) -> Span<'source> {
        Span::new(self.source, self.span.0, self.span.1)
            .expect("Should never construct Tagged with invalid span bounds")
    }

    /// Access the inner node.
    pub const fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> Tagged<'static, T> {
    /// Construct a [`Tagged`] for a compile-time known instance. This should only be used by
    /// generated code.
    pub const fn known(
        inner: T,
        source: &'static str,
        start: usize,
        end: usize,
        hash: u64,
    ) -> Self {
        Self {
            source,
            span: (start, end),
            inner,
            hash,
        }
    }
}

impl<T> Hash for Tagged<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash)
    }
}

impl<T> Eq for Tagged<'_, T> {}

impl<T> PartialEq<Self> for Tagged<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.span == other.span
    }
}

impl<T> PartialOrd<Self> for Tagged<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Tagged<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.span
            .0
            .cmp(&other.span.0)
            .then(self.span.1.cmp(&other.span.1))
    }
}

impl<'source, T> TryFrom<Pair<'source, Rule>> for Tagged<'source, T>
where
    T: TryFrom<Pair<'source, Rule>> + Hash,
{
    type Error = T::Error;

    fn try_from(value: Pair<'source, Rule>) -> Result<Self, Self::Error> {
        let span = value.as_span();
        Ok(Self::new(value.try_into()?, span))
    }
}

impl<'program, 'source, T, U> From<&'program Tagged<'source, T>> for (U, Span<'program>)
where
    U: From<&'program Tagged<'source, T>>,
{
    fn from(value: &'program Tagged<'source, T>) -> Self {
        (U::from(value), value.span())
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
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Program<'a> {
    /// The statements contained within this grammar.
    pub(crate) statements: Cow<'a, [Tagged<'a, Statement<'a>>]>,
}

impl<'a> Program<'a> {
    /// The [`Statement`]s contained within this program.
    pub const fn statements(&self) -> &Cow<'a, [Tagged<'a, Statement<'a>>]> {
        &self.statements
    }
}

impl Program<'static> {
    /// Construct a known [`Program`] at compile-time -- for use in generated code only.
    pub const fn known(statements: &'static [Tagged<'static, Statement<'static>>]) -> Self {
        Self {
            statements: Cow::Borrowed(statements),
        }
    }
}

impl<'a> TryFrom<&'a str> for Program<'a> {
    type Error = ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut parsed = Fandango::parse(Rule::fandango, value)?;
        let (grammar,) = parse_pairs_as!(parsed, (Rule::fandango,));
        let (program, _) = parse_pairs_as!(grammar.into_inner(), (Rule::program, Rule::EOI));

        Program::try_from(program)
    }
}

impl<'a> TryFrom<&'a String> for Program<'a> {
    type Error = ParseError;

    fn try_from(value: &'a String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
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
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
// Comparison is a very large struct, but memory usage is not a concern for codegen
#[allow(clippy::large_enum_variant)]
pub enum Statement<'a> {
    /// A production representing a rule within the grammar.
    Production(Tagged<'a, Production<'a>>),
    /// A constraint applied within the grammar.
    Constraint(Tagged<'a, Constraint<'a>>),
    /// Python code present in the grammar for the definition of e.g. generators and constraints.
    Python,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Statement<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::statement);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::production => Statement::Production(Pair::try_into(inner)?),
            Rule::constraint => Statement::Constraint(Pair::try_into(inner)?),
            // Rule::python => todo!("Python parsing is not yet implemented"),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

/// A production rule within the grammar.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Production<'a> {
    /// The nonterminal which is defined by this production.
    pub(crate) nonterminal: Tagged<'a, Nonterminal<'a>>,
    /// An alternative which defines the rule.
    pub(crate) alternative: Tagged<'a, Alternative<'a>>,
}

impl<'a> Production<'a> {
    /// Access the [`Nonterminal`] associated with this production.
    pub const fn nonterminal(&self) -> &Tagged<'a, Nonterminal<'a>> {
        &self.nonterminal
    }

    /// Access the [`Alternative`] which defines the associated [`Nonterminal`].
    pub const fn alternative(&self) -> &Tagged<'a, Alternative<'a>> {
        &self.alternative
    }
}

impl Production<'static> {
    /// Construct a known [`Production`] at compile-time -- for use in generated code only.
    pub const fn known(
        nonterminal: Tagged<'static, Nonterminal<'static>>,
        alternative: Tagged<'static, Alternative<'static>>,
    ) -> Self {
        Self {
            nonterminal,
            alternative,
        }
    }
}

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
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Nonterminal<'a> {
    /// The name of the non-terminal.
    name: &'a str,
}

impl<'a> Nonterminal<'a> {
    /// Create a non-terminal (useful for testing and referring to non-terminals directly).
    pub const fn new(name: &'a str) -> Self {
        Self { name }
    }

    /// Get the name of this non-terminal (not including the angle brackets).
    pub const fn name(&self) -> &'a str {
        self.name
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Nonterminal<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::nonterminal);

        let (name,) = parse_pairs_as!(value.into_inner(), (Rule::name,));

        Ok(Self {
            name: name.as_str(),
        })
    }
}

/// A list of potential instantiations.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Alternative<'a> {
    /// The [`Concatenation`]s which represent the possible alternatives.
    pub(crate) concatenations: Cow<'a, [Tagged<'a, Concatenation<'a>>]>,
}

impl<'a> Alternative<'a> {
    /// The [`Concatenation`]s, of which exactly one will be instantiated as child 0.
    pub const fn concatenations(&self) -> &Cow<'a, [Tagged<'a, Concatenation<'a>>]> {
        &self.concatenations
    }
}

impl Alternative<'static> {
    /// Construct a known [`Alternative`] at compile-time -- for use with generated code only.
    pub const fn known(concatenations: &'static [Tagged<'static, Concatenation<'static>>]) -> Self {
        Self {
            concatenations: Cow::Borrowed(concatenations),
        }
    }
}

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
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Concatenation<'a> {
    /// The concatenated operators.
    pub(crate) operators: Cow<'a, [Tagged<'a, Operator<'a>>]>,
}

impl<'a> Concatenation<'a> {
    /// The [`Operator`]s which are concatenated.
    pub const fn operators(&self) -> &Cow<'a, [Tagged<'a, Operator<'a>>]> {
        &self.operators
    }
}

impl Concatenation<'static> {
    /// Construct a known [`Concatenation`] at compile-time -- for use with generated code only.
    pub const fn known(operators: &'static [Tagged<'static, Operator<'static>>]) -> Self {
        Self {
            operators: Cow::Borrowed(operators),
        }
    }
}

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
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Operator<'a> {
    /// Kleene star (0 to many) postfix operation.
    Kleene(Tagged<'a, Symbol<'a>>),
    /// Plus (1 to many) postfix operation.
    Plus(Tagged<'a, Symbol<'a>>),
    /// Optional (0 or 1) postfix operation.
    Option(Tagged<'a, Symbol<'a>>),
    /// Repetition postfix operation, with range specified as `{n}` for exactly `n` repetitions or
    /// `{m,n}` for any number of repetitions between `m` and `n`, inclusive.
    Repeat(Tagged<'a, Symbol<'a>>, usize, usize),
    /// Simple case: exactly 1 [`Symbol`].
    Symbol(Tagged<'a, Symbol<'a>>),
}

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
                Operator::Repeat(symbol, range_start, range_end)
            }
            Rule::symbol => Operator::Symbol(inner.try_into()?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

/// A single symbol within the grammar, or a list of [`Alternative`]s.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Symbol<'a> {
    /// A single non-terminal.
    Nonterminal(Tagged<'a, Nonterminal<'a>>),
    /// A string-like terminal.
    String(Tagged<'a, Cow<'a, [u8]>>),
    /// A list of [`Alternative`]s.
    Alternative(Tagged<'a, Alternative<'a>>),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Symbol<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::symbol);

        let inner = value.into_inner().next().unwrap();
        let rule = inner.as_rule();

        Ok(match rule {
            Rule::nonterminal => Symbol::Nonterminal(inner.try_into()?),
            Rule::string => Symbol::String(parse_string(inner)?),
            Rule::bytes => Symbol::String(parse_bytes(inner)?),
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
    use alloc::boxed::Box;
    use alloc::format;
    use alloc::vec::Vec;
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
            Rule::octal_escape => core::char::from_u32(
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
                core::char::from_u32(u32::from_str_radix(&seq.as_str()[1..], 16).unwrap())
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

    pub fn parse_string(string: Pair<Rule>) -> Result<Tagged<Cow<[u8]>>, ParseError> {
        debug_assert_eq!(string.as_rule(), Rule::string);
        let (string_body,) = parse_pairs_as!(string.into_inner(), (_,));
        match string_body.as_rule() {
            Rule::short_string_body | Rule::long_string_body => {
                let mut out = Vec::new();
                let orig = string_body.as_str();
                let span = string_body.as_span();
                for item in string_body.into_inner() {
                    match item.as_rule() {
                        Rule::short_string_non_escape
                        | Rule::long_string_non_escape
                        | Rule::string_unknown_escape => out.extend(item.as_str().as_bytes()),
                        Rule::line_continuation_seq => (),
                        Rule::string_escape_seq => out.extend(
                            parse_string_escape_seq(item)?
                                .encode_utf8(&mut [0u8; 4])
                                .as_bytes(),
                        ),
                        _ => unreachable!(),
                    }
                }
                // escapes always increase length
                if orig.len() == out.len() {
                    Ok(Tagged::new(Cow::Borrowed(orig.as_bytes()), span))
                } else {
                    Ok(Tagged::new(Cow::Owned(out), span))
                }
            }
            _ => unreachable!(),
        }
    }

    fn parse_bytes_escape_seq(escape_seq: Pair<'_, Rule>) -> Result<u8, ParseError> {
        debug_assert_eq!(escape_seq.as_rule(), Rule::bytes_escape_seq);
        let (seq,) = parse_pairs_as!(escape_seq.into_inner(), (_,));
        match seq.as_rule() {
            Rule::char_escape => Ok(match seq.as_str() {
                "\\" => b'\\',
                "'" => b'\'',
                "\"" => b'"',
                "a" => b'\x07',
                "b" => b'\x08',
                "f" => b'\x0C',
                "n" => b'\n',
                "r" => b'\r',
                "t" => b'\t',
                "v" => b'\x0B',
                _ => unreachable!(),
            }),
            Rule::octal_escape => u8::from_str_radix(seq.as_str(), 8).map_err(|err| {
                Box::new(PestError::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("failed to parse \\{} as u8: {}", seq.as_str(), err,),
                    },
                    seq.as_span(),
                ))
            }),
            Rule::hex_escape => Ok(u8::from_str_radix(&seq.as_str()[1..], 16).unwrap()),
            _ => unreachable!(),
        }
    }

    pub fn parse_bytes(bytes: Pair<Rule>) -> Result<Tagged<Cow<[u8]>>, ParseError> {
        debug_assert_eq!(bytes.as_rule(), Rule::bytes);
        let (bytes_body,) = parse_pairs_as!(bytes.into_inner(), (_,));
        match bytes_body.as_rule() {
            Rule::short_bytes_body | Rule::long_bytes_body => {
                let mut out = Vec::new();
                let orig = bytes_body.as_str().as_bytes();
                let span = bytes_body.as_span();
                for item in bytes_body.into_inner() {
                    match item.as_rule() {
                        Rule::short_bytes_non_escape
                        | Rule::long_bytes_non_escape
                        | Rule::bytes_unknown_escape => {
                            out.extend_from_slice(item.as_str().as_bytes())
                        }
                        Rule::line_continuation_seq => (),
                        Rule::bytes_escape_seq => out.push(parse_bytes_escape_seq(item)?),
                        _ => unreachable!(),
                    }
                }
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
        parse_string, Alternative, Concatenation, Nonterminal, Operator, Production, Program, Rule,
        Statement, Symbol,
    };
    use alloc::borrow::Cow;
    use alloc::boxed::Box;
    use alloc::format;
    use alloc::vec::Vec;
    use core::error::Error;
    use pest::iterators::Pair;
    use pest::Parser;

    fn check_string_operator<const BORROW: bool>(operator: Pair<'_, Rule>, expected: &str) {
        let (symbol,) = parse_pairs_as!(operator.into_inner(), (Rule::symbol,));
        let (string,) = parse_pairs_as!(symbol.into_inner(), (Rule::string,));
        let actual = parse_string(string).expect("Expected valid string");
        assert_eq!(matches!(actual.inner(), Cow::Borrowed(_)), BORROW);
        assert_eq!(*actual.inner(), expected.as_bytes());
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

    pub const SIMPLE_GRAMMAR: &str = include_str!("../../../tests/grammars/simple.fan");
    pub const XML_CONSTRAINED_GRAMMAR: &str =
        include_str!("../../../tests/grammars/xml_constrained.fan");

    #[test]
    fn test_grammar() -> Result<(), Box<dyn Error>> {
        let (grammar,) = parse_pairs_as!(
            Fandango::parse(Rule::fandango, SIMPLE_GRAMMAR).unwrap(),
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
        let program = Program::try_from(SIMPLE_GRAMMAR).unwrap();

        let [start, expr, number, non_zero, digit] =
            program.statements.into_owned().try_into().unwrap();

        let prod = untag_or_die!(start, Statement::Production(prod));
        let (nt, alt) = untag_or_die!(
            prod,
            Production {
                nonterminal,
                alternative
            }
        );

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "start");

        let [concat] = untag_or_die!(alt, Alternative { concatenations })
            .into_owned()
            .try_into()
            .unwrap();
        let [op] = untag_or_die!(concat, Concatenation { operators })
            .into_owned()
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "expr");

        let prod = untag_or_die!(expr, Statement::Production(prod));
        let (nt, alt) = untag_or_die!(
            prod,
            Production {
                nonterminal,
                alternative
            }
        );

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "expr");

        let [c1, c2] = untag_or_die!(alt, Alternative { concatenations })
            .into_owned()
            .try_into()
            .unwrap();

        let [op1, op2, op3] = untag_or_die!(c1, Concatenation { operators })
            .into_owned()
            .try_into()
            .unwrap();

        let sym = untag_or_die!(op1, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "number");

        let sym = untag_or_die!(op2, Operator::Symbol(nt));
        let value = untag_or_die!(sym, Symbol::String(sym));
        assert_eq!(value.inner, b"+".as_slice());

        let sym = untag_or_die!(op3, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "expr");

        let [op] = untag_or_die!(c2, Concatenation { operators })
            .into_owned()
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "number");

        let prod = untag_or_die!(number, Statement::Production(prod));
        let (nt, alt) = untag_or_die!(
            prod,
            Production {
                nonterminal,
                alternative
            }
        );

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "number");

        let [c1, c2] = untag_or_die!(alt, Alternative { concatenations })
            .into_owned()
            .try_into()
            .unwrap();

        let [op] = untag_or_die!(c1, Concatenation { operators })
            .into_owned()
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let value = untag_or_die!(sym, Symbol::String(sym));
        assert_eq!(value.inner, b"0".as_slice());

        let [op1, op2] = untag_or_die!(c2, Concatenation { operators })
            .into_owned()
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
        let (nt, alt) = untag_or_die!(
            prod,
            Production {
                nonterminal,
                alternative
            }
        );

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "non_zero");

        let concats = untag_or_die!(alt, Alternative { concatenations }).into_owned();

        for (concat, i) in concats.into_iter().zip(1..=9u8) {
            let [op] = untag_or_die!(concat, Concatenation { operators })
                .into_owned()
                .try_into()
                .unwrap();
            let sym = untag_or_die!(op, Operator::Symbol(nt));
            let value = untag_or_die!(sym, Symbol::String(sym));
            assert_eq!(value.inner, format!("{}", i).as_bytes());
        }

        let prod = untag_or_die!(digit, Statement::Production(prod));
        let (nt, alt) = untag_or_die!(
            prod,
            Production {
                nonterminal,
                alternative
            }
        );

        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "digit");

        let [c1, c2] = untag_or_die!(alt, Alternative { concatenations })
            .into_owned()
            .try_into()
            .unwrap();

        let [op] = untag_or_die!(c1, Concatenation { operators })
            .into_owned()
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let value = untag_or_die!(sym, Symbol::String(sym));
        assert_eq!(value.inner, b"0".as_slice());

        let [op] = untag_or_die!(c2, Concatenation { operators })
            .into_owned()
            .try_into()
            .unwrap();
        let sym = untag_or_die!(op, Operator::Symbol(nt));
        let nt = untag_or_die!(sym, Symbol::Nonterminal(sym));
        let name = untag_or_die!(nt, Nonterminal { name });
        assert_eq!(name, "non_zero");

        Ok(())
    }

    #[test]
    fn test_fullparse_constraints() {
        let _ = Program::try_from(XML_CONSTRAINED_GRAMMAR).unwrap();
    }
}

/// The node type used to represent the grammar's graph.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(missing_docs)]
pub enum FandangoNode<'program, 'source> {
    // Core grammar components
    Program(&'program Program<'source>),
    Statement(&'program Statement<'source>),
    Production(&'program Production<'source>),
    Nonterminal(&'program Nonterminal<'source>),
    Alternative(&'program Alternative<'source>),
    Concatenation(&'program Concatenation<'source>),
    Operator(&'program Operator<'source>),
    Symbol(&'program Symbol<'source>),
    String(&'program Tagged<'source, Cow<'source, [u8]>>),
    // Constraint components
    Constraint(&'program Constraint<'source>),
    Implies(&'program Implies<'source>),
    Quantifier(&'program Quantifier<'source>),
    QuantifierSpecification(&'program QuantifierSpecification<'source>),
    Disjunction(&'program Disjunction<'source>),
    Conjunction(&'program Conjunction<'source>),
    Atom(&'program Atom<'source>),
    Comparison(&'program Comparison<'source>),
    ConstraintOperator(&'program Tagged<'source, ConstraintOperator>),
    Expr(&'program Expr<'source>),
    SelectorLength(&'program SelectorLength<'source>),
    Selection(&'program Selection<'source>),
    Selector(&'program Selector<'source>),
    BaseSelection(&'program BaseSelection<'source>),
    RsPairs(&'program RsPairs<'source>),
    RsPair(&'program RsPair<'source>),
    RsSlices(&'program RsSlices<'source>),
    RsSlice(&'program Tagged<'source, RsSlice>),
    Inversion(&'program Inversion<'source>),
}

impl Display for FandangoNode<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FandangoNode::Program(_) => f.write_str("PROG"),
            FandangoNode::Statement(_) => f.write_str("STMT"),
            FandangoNode::Production(_) => f.write_str("PROD"),
            FandangoNode::Nonterminal(nt) => {
                f.write_char('<')?;
                f.write_str(nt.name())?;
                f.write_char('>')
            }
            FandangoNode::Alternative(_) => f.write_char('|'),
            FandangoNode::Concatenation(_) => f.write_char('~'),
            FandangoNode::Operator(op) => match op {
                Operator::Kleene(_) => f.write_char('*'),
                Operator::Plus(_) => f.write_char('+'),
                Operator::Option(_) => f.write_char('?'),
                Operator::Repeat(_, start, end) => f.write_str(&format!("{{{},{}}}", start, end)),
                Operator::Symbol(_) => f.write_str("OP"),
            },
            FandangoNode::Symbol(_) => f.write_str("SYM"),
            FandangoNode::String(s) => fmt::Debug::fmt(s, f),
            FandangoNode::Constraint(_) => f.write_str("CNST"),
            FandangoNode::Implies(_) => f.write_str("=>"),
            FandangoNode::Quantifier(Quantifier::Forall(_)) => f.write_char('∀'),
            FandangoNode::Quantifier(Quantifier::Exists(_)) => f.write_char('∃'),
            FandangoNode::Quantifier(Quantifier::Disjunction(_)) => f.write_str("QNTF"),
            FandangoNode::QuantifierSpecification(_) => f.write_str("QSPEC"),
            FandangoNode::Disjunction(_) => f.write_str("DISJ"),
            FandangoNode::Conjunction(_) => f.write_str("CONJ"),
            FandangoNode::Atom(_) => f.write_str("ATOM"),
            FandangoNode::Comparison(_) => f.write_str("CMP"),
            FandangoNode::ConstraintOperator(op) => match op.inner() {
                ConstraintOperator::Neq => f.write_str("!="),
                ConstraintOperator::Lt => f.write_char('<'),
                ConstraintOperator::LtEq => f.write_str("<="),
                ConstraintOperator::Eq => f.write_str("=="),
                ConstraintOperator::GtEq => f.write_str(">="),
                ConstraintOperator::Gt => f.write_char('>'),
            },
            FandangoNode::Expr(_) => f.write_str("EXPR"),
            FandangoNode::SelectorLength(SelectorLength::WithLength(_)) => f.write_str("COUNT"),
            FandangoNode::SelectorLength(SelectorLength::NoLength(_)) => f.write_str("SELECT"),
            FandangoNode::Selector(_) => f.write_str("SELECTOR"),
            FandangoNode::Selection(Selection::OverSlices(_, _))
            | FandangoNode::Selection(Selection::OverPairs(_, _)) => f.write_str("OVER"),
            FandangoNode::Selection(Selection::Basic(_)) => f.write_str("DIRECT"),
            FandangoNode::BaseSelection(BaseSelection::Nonterminal(_)) => f.write_str("EXACT"),
            FandangoNode::BaseSelection(BaseSelection::Selector(_)) => f.write_str("SELECTED"),
            FandangoNode::RsPairs(_) => f.write_str("PAIRS"),
            FandangoNode::RsPair(_) => f.write_str("PAIR"),
            FandangoNode::RsSlices(_) => f.write_str("SLICES"),
            FandangoNode::RsSlice(s) => Display::fmt(s.inner(), f),
            FandangoNode::Inversion(Inversion::Selector(_)) => f.write_str("SELECTED"),
            FandangoNode::Inversion(Inversion::Stringified(_)) => f.write_str("STRINGIFIED"),
        }
    }
}

impl<'program, 'source> From<&'program Tagged<'source, Cow<'source, [u8]>>>
    for FandangoNode<'program, 'source>
{
    fn from(value: &'program Tagged<'source, Cow<'source, [u8]>>) -> Self {
        Self::String(value)
    }
}

impl<'program, 'source> From<&'program Tagged<'source, ConstraintOperator>>
    for FandangoNode<'program, 'source>
{
    fn from(value: &'program Tagged<'source, ConstraintOperator>) -> Self {
        Self::ConstraintOperator(value)
    }
}

impl<'program, 'source> From<&'program Tagged<'source, RsSlice>>
    for FandangoNode<'program, 'source>
{
    fn from(value: &'program Tagged<'source, RsSlice>) -> Self {
        Self::RsSlice(value)
    }
}

macro_rules! impl_node_from {
    ($node:tt) => {
        impl<'program, 'source> From<&'program $node<'source>> for FandangoNode<'program, 'source> {
            fn from(value: &'program $node<'source>) -> Self {
                Self::$node(value)
            }
        }

        impl<'program, 'source> From<&'program Tagged<'source, $node<'source>>>
            for FandangoNode<'program, 'source>
        {
            fn from(value: &'program Tagged<'source, $node<'source>>) -> Self {
                Self::$node(value.inner())
            }
        }
    };
}

impl_node_from!(Program);
impl_node_from!(Statement);
impl_node_from!(Production);
impl_node_from!(Nonterminal);
impl_node_from!(Alternative);
impl_node_from!(Concatenation);
impl_node_from!(Operator);
impl_node_from!(Symbol);
impl_node_from!(Constraint);
impl_node_from!(Implies);
impl_node_from!(Quantifier);
impl_node_from!(QuantifierSpecification);
impl_node_from!(Disjunction);
impl_node_from!(Conjunction);
impl_node_from!(Atom);
impl_node_from!(Comparison);
impl_node_from!(Expr);
impl_node_from!(SelectorLength);
impl_node_from!(Selector);
impl_node_from!(Selection);
impl_node_from!(BaseSelection);
impl_node_from!(RsPairs);
impl_node_from!(RsPair);
impl_node_from!(RsSlices);
impl_node_from!(Inversion);
