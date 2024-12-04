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
            FandangoNode::String(s) => Some(s.clone()),
            _ => None,
        }) {
            let mut hasher = DefaultHasher::new();
            string.hash(&mut hasher);
            let hash = hasher.finish();
            if hashes.insert(hash) {
                writeln!(output, "lit_{hash} = {{ {string:?} }}")?;
            }
        }

        let pest_name = Cow::Borrowed("start");
        let start = Nonterminal::new(pest_name.clone());
        let start = FandangoNode::Nonterminal(&start);
        start.emit_pest(&mut (self, HashSet::new(), pest_name), output)
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
                    nt.name().clone()
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
                        Operator::Kleene(_) | Operator::Repeat(_, _) => {
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
            FandangoNode::Program(_)
            | FandangoNode::Statement(_)
            | FandangoNode::Production(_)
            | FandangoNode::Symbol(_) => {
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
