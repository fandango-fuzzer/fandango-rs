// re-export to maintain import sanity
use fandango_core::graph::FandangoNode;
use fandango_core::lang::Operator;
pub use pest::*;
use petgraph::graphmap::DiGraphMap;
use std::fmt;
use std::fmt::Write;

pub trait IntoPestSource<C> {
    /// The error type which is encountered as a result of trying to emit the source code.
    type OutputError;

    /// Emits the types for this structure.
    fn emit_pest(&self, ctx: &C, output: &mut String) -> Result<(), Self::OutputError>;
}

impl<'source> IntoPestSource<()> for DiGraphMap<FandangoNode<'_, 'source>, Span<'source>> {
    type OutputError = fmt::Error;

    fn emit_pest(&self, _ctx: &(), output: &mut String) -> Result<(), Self::OutputError> {
        for production in self
            .nodes()
            .filter(|n| matches!(n, FandangoNode::Production(_)))
        {
            let mut edges = self.edges(production);
            let (_, nt, _) = edges
                .next()
                .expect("Exactly one non-terminal per production.");
            debug_assert!(edges.next().is_none());

            if let FandangoNode::Nonterminal(_) = nt {
                nt.emit_pest(self, output)?;
                write!(output, " = {{ ")?;
                let mut edges = self.edges(nt);
                let (_, child, _) = edges.next().expect("Exactly one child of non-terminal.");
                debug_assert!(edges.next().is_none());
                child.emit_pest(self, output)?;
                writeln!(output, " }}")?;
            } else {
                panic!("Invalid child of production.");
            };
        }
        Ok(())
    }
}

impl<'source> IntoPestSource<DiGraphMap<Self, Span<'source>>> for FandangoNode<'_, 'source> {
    type OutputError = fmt::Error;

    fn emit_pest(
        &self,
        ctx: &DiGraphMap<Self, Span<'source>>,
        output: &mut String,
    ) -> Result<(), Self::OutputError> {
        let mut children = ctx.edges(*self).collect::<Vec<_>>();
        children.sort_by_key(|v| v.2.start());
        match self {
            FandangoNode::Nonterminal(nt) => {
                write!(output, "{}", nt.name())
            }
            FandangoNode::Alternative(_) => {
                write!(output, "( ")?;
                let mut alternatives = children.iter().map(|(_, child, _)| child);
                alternatives.next().unwrap().emit_pest(ctx, output)?;
                for alternative in alternatives {
                    write!(output, " | ")?;
                    alternative.emit_pest(ctx, output)?;
                }
                write!(output, " )")
            }
            FandangoNode::Concatenation(_) => {
                write!(output, "( ")?;
                let mut operators = children.iter().map(|(_, child, _)| child);
                operators.next().unwrap().emit_pest(ctx, output)?;
                for operator in operators {
                    write!(output, " ~ ")?;
                    operator.emit_pest(ctx, output)?;
                }
                write!(output, " )")
            }
            FandangoNode::Operator(o) => {
                write!(output, "( ")?;
                let mut edges = children.iter();
                let (_, child, _) = edges.next().expect("Exactly one child of operator.");
                debug_assert!(edges.next().is_none());
                child.emit_pest(ctx, output)?;
                write!(output, " ){}", match o {
                    Operator::Plus(_) => {
                        '*'
                    }
                    Operator::Option(_) => {
                        '?'
                    }
                    Operator::Kleene(_) | Operator::Repeat(_, _) => {
                        // we secretly emit repeats as *, since pest doesn't have a clean way to handle
                        // this for large repetitions
                        // TODO this is instead implemented as a constraint
                        '*'
                    }
                    Operator::Symbol(_) => unimplemented!("Unexpected symbol; should be elided."),
                })
            }
            FandangoNode::String(s) => {
                write!(output, "{s:?}")
            }
            _ => unimplemented!("Unexpected rule emission; should be elided"),
        }
    }
}
