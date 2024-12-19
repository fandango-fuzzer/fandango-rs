use std::borrow::Cow;
use std::collections::HashSet;
// re-export to maintain import sanity
use fandango_core::graph::FandangoNode;
use fandango_core::lang::{Nonterminal, Operator};
pub use pest::*;
use petgraph::graphmap::DiGraphMap;
use std::fmt;
use std::fmt::Write;
use std::hash::{DefaultHasher, Hash, Hasher};

pub trait IntoPestSource<C> {
    /// The error type which is encountered as a result of trying to emit the source code.
    type OutputError;

    /// Emits the types for this structure.
    fn emit_pest(&self, ctx: &mut C, output: &mut String) -> Result<(), Self::OutputError>;
}

impl<'source> IntoPestSource<()> for DiGraphMap<FandangoNode<'_, 'source>, Span<'source>> {
    type OutputError = fmt::Error;

    fn emit_pest(&self, _ctx: &mut (), output: &mut String) -> Result<(), Self::OutputError> {
        let mut hashes = HashSet::new();
        for string in self.nodes().filter_map(|n| match n {
            FandangoNode::String(s) => Some(s),
            _ => None,
        }) {
            let mut hasher = DefaultHasher::new();
            string.hash(&mut hasher);
            let hash = hasher.finish();
            if hashes.insert(hash) {
                let actual = string.inner();
                writeln!(output, "lit_{hash} = {{ {actual:?} }}")?;
            }
        }

        let pest_name = "start";
        let start = Nonterminal::new(pest_name);
        let start = FandangoNode::Nonterminal(&start);

        let mut visited = HashSet::new();
        visited.insert(start);

        let mut children = self.edges(start);
        let (_, child, _) = children.next().expect("Start node has exactly one member.");
        assert!(children.next().is_none());

        let pest_child_name = if let FandangoNode::Nonterminal(nt) = child {
            Cow::Borrowed(nt.name())
        } else {
            Cow::Owned(format!("{pest_name}_0"))
        };

        writeln!(
            output,
            "{pest_name} = {{ SOI ~ {} ~ EOI }}",
            pest_child_name
        )?;

        child.emit_pest(&mut (self, HashSet::new(), pest_child_name), output)
    }
}

type PestContext<'graph, 'program, 'source> = (
    &'graph DiGraphMap<FandangoNode<'program, 'source>, Span<'source>>,
    HashSet<FandangoNode<'program, 'source>>,
    Cow<'source, str>,
);

impl<'program, 'source> IntoPestSource<PestContext<'_, 'program, 'source>>
    for FandangoNode<'program, 'source>
{
    type OutputError = fmt::Error;

    fn emit_pest(
        &self,
        ctx: &mut PestContext<'_, 'program, 'source>,
        output: &mut String,
    ) -> Result<(), Self::OutputError> {
        let (graph, visited, pest_name) = ctx;
        if !visited.insert(*self) {
            return Ok(());
        }
        let mut children = graph.edges(*self).collect::<Vec<_>>();
        children.sort_by_key(|v| v.2.start());
        let pest_child_names = children
            .iter()
            .enumerate()
            .map(|(i, (_, child, _))| {
                if let FandangoNode::Nonterminal(nt) = child {
                    Cow::Borrowed(nt.name())
                } else {
                    Cow::Owned(format!("{pest_name}_{i}"))
                }
            })
            .collect::<Vec<_>>();
        match self {
            FandangoNode::Nonterminal(_) => {
                assert_eq!(children.len(), 1);
                writeln!(output, "{pest_name} = {{ {} }}", pest_child_names[0])?;
            }
            FandangoNode::Alternative(_) => {
                writeln!(
                    output,
                    "{pest_name} = {{ {} }}",
                    pest_child_names.join(" | ")
                )?;
            }
            FandangoNode::Concatenation(_) => {
                writeln!(
                    output,
                    "{pest_name} = {{ {} }}",
                    pest_child_names.join(" ~ ")
                )?;
            }
            FandangoNode::Operator(o) => {
                writeln!(
                    output,
                    "{pest_name} = {{ {}{} }}",
                    pest_child_names.join(" ~ "),
                    match o {
                        Operator::Plus(_) => {
                            '*'
                        }
                        Operator::Option(_) => {
                            '?'
                        }
                        Operator::Kleene(_) | Operator::Repeat(_, _, _) => {
                            // we secretly emit repeats as *, since pest doesn't have a clean way to handle
                            // this for large repetitions
                            '*'
                        }
                        Operator::Symbol(_) =>
                            unimplemented!("Unexpected symbol; should be elided."),
                    }
                )?;
            }
            FandangoNode::String(s) => {
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                let hash = hasher.finish();
                writeln!(output, "{pest_name} = {{ lit_{hash} }}")?;
            }
            _ => {
                unimplemented!("Unexpected rule emission; should be elided")
            }
        }
        for (alternative, pest_child_name) in children
            .iter()
            .map(|(_, child, _)| child)
            .zip(pest_child_names)
        {
            ctx.2 = pest_child_name;
            alternative.emit_pest(ctx, output)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::pest::IntoPestSource;
    use fandango_core::graph::IntoGraph;
    use fandango_core::lang::Program;
    use std::error::Error;

    const SIMPLE_GRAMMAR: &str = include_str!("../../tests/grammars/simple.fan");

    #[test]
    fn produce_grammar() -> Result<(), Box<dyn Error>> {
        let program = Program::try_from(SIMPLE_GRAMMAR)?;

        let graph = (&program).into_graph();

        let mut pest_grammar = String::new();
        graph.emit_pest(&mut (), &mut pest_grammar)?;

        println!("{pest_grammar}");

        Ok(())
    }
}
