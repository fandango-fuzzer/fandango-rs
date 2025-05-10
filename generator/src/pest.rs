use fandango_core::lang::FandangoNode;
use fandango_core::lang::Operator;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Write;
use std::hash::{DefaultHasher, Hash, Hasher};

// re-export to maintain import sanity
pub use pest::*;

pub trait IntoPestSource<C> {
    /// The error type which is encountered as a result of trying to emit the source code.
    type OutputError;

    /// Emits the types for this structure.
    fn emit_pest(&self, ctx: &mut C, output: &mut String) -> Result<(), Self::OutputError>;
}

impl<'source> IntoPestSource<()> for DiGraph<FandangoNode<'_, 'source>, Span<'source>> {
    type OutputError = fmt::Error;

    fn emit_pest(&self, _ctx: &mut (), output: &mut String) -> Result<(), Self::OutputError> {
        let mut hashes = HashSet::new();
        for string in
            self.node_indices()
                .filter_map(|n| match self.node_weight(n).copied().unwrap() {
                    FandangoNode::String(s) => Some(s),
                    _ => None,
                })
        {
            let mut hasher = DefaultHasher::new();
            string.hash(&mut hasher);
            let hash = hasher.finish();
            if hashes.insert(hash) {
                let actual = string.inner();
                if let Ok(actual) = core::str::from_utf8(actual) {
                    let actual = actual.replace('\\', "\\\\").replace('"', "\\\"");
                    writeln!(output, "lit_{hash} = {{ \"{actual}\" }}")?;
                } else {
                    writeln!(output, "lit_{hash} = {{ \"UNIMPLEMENTED\" }}")?;
                }
            }
        }

        let pest_name = "start";
        let start = self
            .node_indices()
            .find(|&n| matches!(self.node_weight(n).unwrap(), FandangoNode::Nonterminal(nt) if nt.name() == "start"))
            .expect("No start node?");

        let mut visited = HashSet::new();
        visited.insert(start);

        let mut children = self.edges(start);
        let child = children
            .next()
            .expect("Start node has exactly one member.")
            .target();
        assert!(children.next().is_none());
        let child_weight = self.node_weight(child).copied().unwrap();

        let pest_child_name = if let FandangoNode::Nonterminal(nt) = child_weight {
            Cow::Borrowed(nt.name())
        } else {
            Cow::Owned(format!("{pest_name}_0"))
        };

        writeln!(output, "{pest_name} = {{ SOI ~ {pest_child_name} ~ EOI }}")?;

        child.emit_pest(&mut (self, HashSet::new(), pest_child_name), output)
    }
}

type PestContext<'graph, 'program, 'source> = (
    &'graph DiGraph<FandangoNode<'program, 'source>, Span<'source>>,
    HashSet<NodeIndex>,
    Cow<'source, str>,
);

impl<'program, 'source> IntoPestSource<PestContext<'_, 'program, 'source>> for NodeIndex {
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
        let mut children = graph
            .edges(*self)
            .map(|e| {
                (
                    e.target(),
                    *graph.node_weight(e.target()).unwrap(),
                    *e.weight(),
                )
            })
            .collect::<Vec<_>>();
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
        match graph.node_weight(*self).unwrap() {
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
            .map(|(child, _, _)| child)
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

        let (_, graph) = (&program).into_graph();

        let mut pest_grammar = String::new();
        graph.emit_pest(&mut (), &mut pest_grammar)?;

        println!("{pest_grammar}");

        Ok(())
    }
}
