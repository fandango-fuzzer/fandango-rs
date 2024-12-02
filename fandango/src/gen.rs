//! Rust- and Pest-generation of FANDANGO grammars.

use crate::graph::FandangoNode;
use alloc::borrow::Cow;
use pest::Span;
use petgraph::graphmap::DiGraphMap;
use std::collections::HashSet;
use std::io;
use std::io::Write;

mod typing;

/// Produces a Rust source tree using the provided context.
pub trait IntoRustSource<C, W> {
    /// The error type which is encountered as a result of trying to emit the source code.
    type OutputError;

    /// Emits the types for this structure.
    fn typeinfo(&self, ctx: &mut C, output: &mut W) -> Result<(), Self::OutputError>;
}

impl<'source, W> IntoRustSource<DiGraphMap<Self, Span<'source>>, W> for FandangoNode<'_, 'source>
where
    W: Write,
{
    type OutputError = io::Error;

    fn typeinfo(
        &self,
        graph: &mut DiGraphMap<Self, Span<'source>>,
        output: &mut W,
    ) -> Result<(), Self::OutputError> {
        let FandangoNode::Nonterminal(nt) = self else {
            unimplemented!("Can only transforms non-terminals into source code.")
        };

        let mut edges = graph.edges(*self);
        let (_, child, &weight) = edges
            .next()
            .expect("Nonterminals should have exactly one child.");
        debug_assert!(edges.next().is_none());

        let name = nt.name();
        let mut visited = HashSet::new();

        let child_name = if let FandangoNode::Nonterminal(nt) = child {
            nt.name().clone()
        } else {
            Cow::from(format!("{name}_0"))
        };

        writeln!(
            output,
            r#"
pub struct {name} {{
    child_0: {child_name},
}}

impl ::fandango::gen::typing::Children for {name} {{
    type ChildrenRef<'a> = (&'a {child_name},);
    type ChildrenRefMut<'a> = (&'a mut {child_name},);

    fn children(&self) -> Self::ChildrenRef<'_> {{ (&self.child_0,) }}
    fn children_mut<'a>(&'a mut self) -> Self::ChildrenRefMut<'a> {{ (&mut self.child_0,) }}
}}
        "#
        )?;

        visited.insert(*self);
        child.typeinfo(&mut (child_name, *self, weight, visited, graph), output)
    }
}

type FandangoGenContext<'graph, 'program, 'source> = (
    Cow<'source, str>,
    FandangoNode<'program, 'source>,
    Span<'source>,
    HashSet<FandangoNode<'program, 'source>>,
    &'graph mut DiGraphMap<FandangoNode<'program, 'source>, Span<'source>>,
);

impl<'graph, 'program, 'source, W> IntoRustSource<FandangoGenContext<'graph, 'program, 'source>, W>
    for FandangoNode<'program, 'source>
where
    W: Write,
{
    type OutputError = io::Error;

    fn typeinfo(
        &self,
        ctx: &mut FandangoGenContext<'graph, 'program, 'source>,
        output: &mut W,
    ) -> Result<(), Self::OutputError> {
        let (name, _parent, _index, visited, graph) = ctx;
        if visited.contains(self) {
            return Ok(());
        }
        visited.insert(*self);

        let mut children = graph
            .edges(*self)
            .map(|(n1, n2, &w)| (n1, n2, w))
            .collect::<Vec<_>>();
        children.sort_by_key(|(_, _, w)| w.start());
        let child_names = children
            .iter()
            .enumerate()
            .map(|(i, (_, child, _))| {
                if let FandangoNode::Nonterminal(nt) = child {
                    nt.name().clone()
                } else {
                    Cow::from(format!("{name}_{i}"))
                }
            })
            .collect::<Vec<_>>();

        match self {
            FandangoNode::String(s) => {
                writeln!(
                    output,
                    r#"
pub struct {name};

impl ::fandango::gen::typing::Children for {name} {{
    type ChildrenRef<'a> = (&'static str,);
    type ChildrenRefMut<'a> = (&'static str,);

    fn children(&self) -> Self::ChildrenRef<'_> {{ (&{s:?},) }}
    fn children_mut<'a>(&'a mut self) -> Self::ChildrenRefMut<'a> {{ (&{s:?},) }}
}}
        "#
                )?;
            }
            FandangoNode::Bytes(s) => {
                writeln!(
                    output,
                    r#"
pub struct {name};

impl ::fandango::gen::typing::Children for {name} {{
    type ChildrenRef<'a> = (&'static [u8],);
    type ChildrenRefMut<'a> = (&'static [u8],);

    fn children(&self) -> Self::ChildrenRef<'_> {{ (&{s:?},) }}
    fn children_mut<'a>(&'a mut self) -> Self::ChildrenRefMut<'a> {{ (&{s:?},) }}
}}
        "#
                )?;
            }
            FandangoNode::Alternative(_) => {
                writeln!(output, "pub enum {name} {{")?;
                for (i, ((_, child, _), name)) in children.iter().zip(&child_names).enumerate() {
                    match child {
                        FandangoNode::Nonterminal(_) => {
                            writeln!(output, "    alt_child_{i}(Box<{name}>),")?;
                        }
                        _ => {
                            writeln!(output, "    alt_child_{i}({name}),")?;
                        }
                    }
                }
                writeln!(output, "}}")?;
            }
            _ => {
                writeln!(output, "pub struct {name} {{")?;
                for (i, ((_, child, _), name)) in children.iter().zip(&child_names).enumerate() {
                    match child {
                        FandangoNode::Nonterminal(_) => {
                            writeln!(output, "    child_{i}: Box<{name}>,")?;
                        }
                        _ => {
                            writeln!(output, "    child_{i}: {name},")?;
                        }
                    }
                }
                writeln!(output, "}}")?;
                if !children.is_empty() {
                    writeln!(
                        output,
                        "impl ::fandango::gen::typing::Children for {name} {{"
                    )?;
                    write!(output, "    type ChildrenRef<'a> = (")?;
                    for name in &child_names {
                        write!(output, "&'a {name}, ")?;
                    }
                    writeln!(output, ");")?;
                    write!(output, "    type ChildrenRefMut<'a> = (")?;
                    for name in &child_names {
                        write!(output, "&'a mut {name},")?
                    }
                    writeln!(output, ");")?;
                    write!(
                        output,
                        "    fn children(&self) -> Self::ChildrenRef<'_> {{ ("
                    )?;
                    for i in 0..children.len() {
                        write!(output, "&self.child_{i},")?;
                    }
                    writeln!(output, ") }}")?;
                    write!(
                        output,
                        "    fn children_mut<'a>(&'a mut self) -> Self::ChildrenRefMut<'a> {{ ("
                    )?;
                    for i in 0..children.len() {
                        write!(output, "&mut self.child_{i},")?;
                    }
                    writeln!(output, ") }}")?;
                    writeln!(output, "}}")?;
                }
            }
        }
        for ((_, child, i), name) in children.into_iter().zip(child_names) {
            ctx.0 = name;
            ctx.2 = i;
            child.typeinfo(ctx, output)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::gen::IntoRustSource;
    use crate::graph::{FandangoNode, IntoGraph};
    use crate::lang::test::SIMPLE_GRAMMAR;
    use crate::lang::{Nonterminal, Program, Tagged};
    use alloc::borrow::Cow;
    use std::error::Error;

    #[test]
    fn gen_simple() -> Result<(), Box<dyn Error>> {
        let program = Tagged::<Program>::try_from(SIMPLE_GRAMMAR)?;
        let mut graph = (&program).into_graph();

        let mut generated = Vec::new();

        FandangoNode::Nonterminal(&Nonterminal::new(Cow::Borrowed("start")))
            .typeinfo(&mut graph, &mut generated)?;

        let generated = String::from_utf8(generated)?;

        println!("{generated}");

        Ok(())
    }
}
