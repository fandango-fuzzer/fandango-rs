use crate::graph::{FandangoNode, Traverse};
use crate::impl_traverse;
use crate::lang::py_literal::{parse_bytes, parse_string};
use getset::Getters;
use pest::error::{Error as PestError, ErrorVariant};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::RangeInclusive;
use std::str::FromStr;

pub type ParseError = PestError<Rule>;

#[derive(Parser)]
#[grammar = "py_literal/grammar.pest"]
#[grammar = "fandango.pest"]
struct Fandango;

#[derive(Debug, Clone, Eq, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Program<'a> {
    statements: Vec<Statement<'a>>,
}

impl_fandango_traverse!(Program, [statements]);

impl<'a> TryFrom<&'a str> for Program<'a> {
    type Error = ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (grammar,) =
            parse_pairs_as!(Fandango::parse(Rule::fandango, value)?, (Rule::fandango,));
        let (program, _) = parse_pairs_as!(grammar.into_inner(), (Rule::program, Rule::EOI));

        Program::try_from(program)
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Program<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::program);

        Ok(Self {
            statements: value
                .into_inner()
                .map(Statement::try_from)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement<'a> {
    Production(Production<'a>),
    Constraint,
    Python,
}

impl_fandango_traverse!(Statement, match { Production(prod), Constraint, Python });

impl<'a> TryFrom<Pair<'a, Rule>> for Statement<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::statement);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::production => Statement::Production(Production::try_from(inner)?),
            Rule::constraint => todo!("Constraints are not yet implemented"),
            Rule::python => todo!("Python parsing is not yet implemented"),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Production<'a> {
    nonterminal: Nonterminal<'a>,
    alternative: Alternative<'a>,
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

#[derive(Debug, Clone, Eq, Ord, PartialOrd, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Nonterminal<'a> {
    name: Cow<'a, str>,
}

impl<'a> Nonterminal<'a> {
    pub fn new(name: Cow<'a, str>) -> Self {
        Self { name }
    }
}

impl<'program, 'source> Traverse for &'program Nonterminal<'source> {
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

#[derive(Debug, Clone, Eq, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Alternative<'a> {
    concatenations: Vec<Concatenation<'a>>,
}

impl_fandango_traverse!(Alternative, [concatenations]);

impl<'a> TryFrom<Pair<'a, Rule>> for Alternative<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::alternative);

        Ok(Self {
            concatenations: value
                .into_inner()
                .map(Concatenation::try_from)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Concatenation<'a> {
    operators: Vec<Operator<'a>>,
}

impl_fandango_traverse!(Concatenation, [operators]);

impl<'a> TryFrom<Pair<'a, Rule>> for Concatenation<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::concatenation);

        Ok(Self {
            operators: value
                .into_inner()
                .map(Operator::try_from)
                .collect::<Result<_, ParseError>>()?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operator<'a> {
    Kleene(Symbol<'a>),
    Plus(Symbol<'a>),
    Option(Symbol<'a>),
    Repeat(Symbol<'a>, RangeInclusive<usize>),
    Symbol(Symbol<'a>),
}

impl_fandango_traverse!(Operator, match { Kleene(sym), Plus(sym), Option(sym), Repeat(sym, _), Symbol(sym) });

fn parse_range(pair: Pair<Rule>) -> Result<usize, ParseError> {
    usize::from_str(pair.as_str()).map_err(|_| {
        PestError::new_from_span(
            ErrorVariant::CustomError {
                message: "invalid range specifier".to_string(),
            },
            pair.as_span(),
        )
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Symbol<'a> {
    Nonterminal(Nonterminal<'a>),
    String(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
    Alternative(Alternative<'a>),
}

impl_fandango_traverse!(Symbol, match { Nonterminal(nt), String(s), Bytes(b), Alternative(alt) });

impl<'a> TryFrom<Pair<'a, Rule>> for Symbol<'a> {
    type Error = ParseError;

    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::symbol);

        let inner = value.into_inner().next().unwrap();

        Ok(match inner.as_rule() {
            Rule::nonterminal => Symbol::Nonterminal(inner.try_into()?),
            Rule::string => Symbol::String(parse_string(inner)?),
            Rule::bytes => Symbol::Bytes(parse_bytes(inner)?),
            Rule::alternative => Symbol::Alternative(inner.try_into()?),
            _ => unreachable!("This case is not represented within the grammar."),
        })
    }
}

/// This section is mostly copied from py_literal: https://github.com/jturner314/py_literal/releases/tag/0.4.0
/// This is necessary because pest does not easily allow for grammar + extract dependencies.
mod py_literal {
    use crate::lang::{ParseError, Rule};
    use alloc::borrow::Cow;
    use pest::error::ErrorVariant;
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
                ParseError::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("Octal escape is invalid: \\{}", seq.as_str()),
                    },
                    seq.as_span(),
                )
            }),
            Rule::hex_escape | Rule::unicode_hex_escape => {
                ::std::char::from_u32(u32::from_str_radix(&seq.as_str()[1..], 16).unwrap())
                    .ok_or_else(|| {
                        ParseError::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!("Hex escape is invalid: \\x{}", seq.as_str()),
                            },
                            seq.as_span(),
                        )
                    })
            }
            Rule::name_escape => Err(ParseError::new_from_span(
                ErrorVariant::CustomError {
                    message: "Unicode name escapes are not supported.".into(),
                },
                seq.as_span(),
            )),
            _ => unreachable!(),
        }
    }

    pub fn parse_string(string: Pair<Rule>) -> Result<Cow<str>, ParseError> {
        debug_assert_eq!(string.as_rule(), Rule::string);
        let (string_body,) = parse_pairs_as!(string.into_inner(), (_,));
        match string_body.as_rule() {
            Rule::short_string_body | Rule::long_string_body => {
                let mut out = String::new();
                let orig = string_body.as_str();
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
                    Ok(Cow::Borrowed(orig))
                } else {
                    Ok(Cow::Owned(out))
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
                ParseError::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("failed to parse \\{} as u8: {}", seq.as_str(), err,),
                    },
                    seq.as_span(),
                )
            }),
            Rule::hex_escape => Ok(u8::from_str_radix(&seq.as_str()[1..], 16).unwrap()),
            _ => unreachable!(),
        }
    }

    pub fn parse_bytes(bytes: Pair<Rule>) -> Result<Cow<[u8]>, ParseError> {
        debug_assert_eq!(bytes.as_rule(), Rule::bytes);
        let (bytes_body,) = parse_pairs_as!(bytes.into_inner(), (_,));
        match bytes_body.as_rule() {
            Rule::short_bytes_body | Rule::long_bytes_body => {
                let mut out = Vec::new();
                let orig = bytes_body.as_str().as_bytes();
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
                    Ok(Cow::Borrowed(orig))
                } else {
                    Ok(Cow::Owned(out))
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::lang::{
        parse_string, Alternative, Concatenation, Fandango, Nonterminal, Operator, Production,
        Program, Rule, Statement, Symbol,
    };
    use alloc::borrow::Cow;
    use alloc::boxed::Box;
    use pest::iterators::Pair;
    use pest::Parser;
    use std::error::Error;

    fn check_string_operator<const BORROW: bool>(operator: Pair<'_, Rule>, expected: &str) {
        let (symbol,) = parse_pairs_as!(operator.into_inner(), (Rule::symbol,));
        let (string,) = parse_pairs_as!(symbol.into_inner(), (Rule::string,));
        let actual = parse_string(string).expect("Expected valid string");
        assert_eq!(matches!(actual, Cow::Borrowed(_)), BORROW);
        assert_eq!(actual, expected);
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

    pub const SIMPLE_GRAMMAR: &str = r#"
    <start> ::= <expr>;
    <expr> ::= <number> | <number> "+" <expr>;
    <number> ::= <non_zero><digit>* | "0";
    <non_zero> ::=
                  "1"
                | "2"
                | "3"
                | "4"
                | "5"
                | "6"
                | "7"
                | "8"
                | "9"
                ;
    <digit> ::= "0" | <non_zero>;
"#;

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
                let (number, addition) = parse_pairs_as!(
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
                let (number, zero) = parse_pairs_as!(
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

    #[test]
    fn test_fullparse() -> Result<(), Box<dyn Error>> {
        let program = Program::try_from(SIMPLE_GRAMMAR)?;

        assert_eq!(
            program,
            Program {
                statements: vec![
                    Statement::Production(Production {
                        nonterminal: Nonterminal {
                            name: Cow::Borrowed("start")
                        },
                        alternative: Alternative {
                            concatenations: vec![Concatenation {
                                operators: vec![Operator::Symbol(Symbol::Nonterminal(
                                    Nonterminal {
                                        name: Cow::Borrowed("expr")
                                    }
                                ))],
                            },]
                        },
                    }),
                    Statement::Production(Production {
                        nonterminal: Nonterminal {
                            name: Cow::Borrowed("expr")
                        },
                        alternative: Alternative {
                            concatenations: vec![
                                Concatenation {
                                    operators: vec![Operator::Symbol(Symbol::Nonterminal(
                                        Nonterminal {
                                            name: Cow::Borrowed("number")
                                        }
                                    ))],
                                },
                                Concatenation {
                                    operators: vec![
                                        Operator::Symbol(Symbol::Nonterminal(Nonterminal {
                                            name: Cow::Borrowed("number")
                                        })),
                                        Operator::Symbol(Symbol::String(Cow::Borrowed("+"))),
                                        Operator::Symbol(Symbol::Nonterminal(Nonterminal {
                                            name: Cow::Borrowed("expr")
                                        }))
                                    ],
                                },
                            ]
                        },
                    }),
                    Statement::Production(Production {
                        nonterminal: Nonterminal {
                            name: Cow::Borrowed("number")
                        },
                        alternative: Alternative {
                            concatenations: vec![
                                Concatenation {
                                    operators: vec![
                                        Operator::Symbol(Symbol::Nonterminal(Nonterminal {
                                            name: Cow::Borrowed("non_zero")
                                        })),
                                        Operator::Kleene(Symbol::Nonterminal(Nonterminal {
                                            name: Cow::Borrowed("digit")
                                        }))
                                    ],
                                },
                                Concatenation {
                                    operators: vec![Operator::Symbol(Symbol::String(
                                        Cow::Borrowed("0")
                                    ))],
                                }
                            ]
                        },
                    }),
                    Statement::Production(Production {
                        nonterminal: Nonterminal {
                            name: Cow::Borrowed("non_zero")
                        },
                        alternative: Alternative {
                            concatenations: (1..=9u8)
                                .map(|i| Concatenation {
                                    operators: vec![Operator::Symbol(Symbol::String(Cow::Owned(
                                        i.to_string()
                                    )))]
                                })
                                .collect()
                        },
                    }),
                    Statement::Production(Production {
                        nonterminal: Nonterminal {
                            name: Cow::Borrowed("digit")
                        },
                        alternative: Alternative {
                            concatenations: vec![
                                Concatenation {
                                    operators: vec![Operator::Symbol(Symbol::String(
                                        Cow::Borrowed("0")
                                    )),]
                                },
                                Concatenation {
                                    operators: vec![Operator::Symbol(Symbol::Nonterminal(
                                        Nonterminal {
                                            name: Cow::Borrowed("non_zero")
                                        }
                                    ))]
                                }
                            ]
                        },
                    })
                ],
            }
        );

        Ok(())
    }
}
