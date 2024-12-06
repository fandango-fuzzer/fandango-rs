//! Build tests for FANDANGO, to ensure that we are generating code as expected.

use fandango::Parser;
use fandango::parse_pairs_as;
use fandango_core::graph::Traverse;
use fandango_core::typing::{Children, Node};

mod simple {
    use super::*;
    use fandango::Fandango;
    use fandango_core::graph::{FandangoNode, IntoGraph};
    use fandango_core::typing::Structured;
    use petgraph::dot::{Config, Dot};
    use petgraph::graphmap::DiGraphMap;

    #[derive(Fandango)]
    #[grammar = "tests/grammars/simple.fan"]
    pub struct Simple;

    #[test]
    fn parse() -> Result<(), ParseError> {
        println!("{}", _PEST_SOURCE);

        let graph = STRUCTURE.into_graph();

        let renderable = DiGraphMap::from_edges(graph.all_edges().map(|(n1, n2, weight)| {
            let (start_line, start_col) = weight.start_pos().line_col();
            let (end_line, end_col) = weight.end_pos().line_col();
            let rendered = if start_line == end_line {
                format!("{start_line}:{start_col}-{end_col}")
            } else {
                format!("{start_line}:{start_col}-{end_line}:{end_col}")
            };
            (n1, n2, rendered)
        }));

        let rendered = Dot::with_attr_getters(
            &renderable,
            &[Config::NodeNoLabel, Config::EdgeNoLabel],
            &|_, (_, _, weight)| format!("label = {:?}", weight),
            &|_, (_, node)| format!("label = {:?}", format!("{}", node)),
        );

        println!("{rendered}");

        const SAMPLE: &str = "1+2";

        let start = Simple::extract(SAMPLE)?;
        assert!(graph.contains_node(start.as_node()));
        let expr = start.children().0;
        assert!(graph.contains_node(expr.as_node()));
        if let nonterminal_expr_0::variant_0(expr) = expr.children().0 {
            // assert!(graph.contains_node(expr.as_node()));
            let (number, plus, expr) = expr.children();
            assert!(graph.contains_node(number.as_node()));
            assert!(graph.contains_node(expr.as_node()));
            assert_eq!(number.span().unwrap().as_str(), "1");
            assert_eq!(plus.span().unwrap().as_str(), "+");
            if let nonterminal_expr_0::variant_1(number) = expr.children().0 {
                assert!(graph.contains_node(number.as_node()));
                assert_eq!(number.span().unwrap().as_str(), "2");
                return Ok(());
            }
        }

        panic!("Parse did not match expected value!");
    }
}

mod pest_renamed {
    use super::*;
    use fandango::Fandango;

    #[derive(Fandango)]
    #[grammar = "tests/grammars/pest-renamed.fan"]
    pub struct PestRenamed;

    #[test]
    fn pest_name_sanity() -> Result<(), ParseError> {
        use pest_renamed::*;

        println!("{}", _PEST_SOURCE);

        STRUCTURE.recurse(|_, _, _| {});

        const SAMPLE: &str = "hello!";

        let (start,) = parse_pairs_as!(PestRenamed::parse(Rule::start, SAMPLE)?, (Rule::start,));
        let (actual, _) = parse_pairs_as!(start.into_inner(), (Rule::pest, Rule::EOI));
        assert_eq!(actual.as_span().as_str(), SAMPLE);

        let start = PestRenamed::extract(SAMPLE)?;
        let pest = start.children().0;
        let string = pest.children().0.children().0;

        assert_eq!(string, SAMPLE);

        Ok(())
    }
}
