//! Build tests for FANDANGO, to ensure that we are generating code as expected.

use fandango::Parser;
use fandango::parse_pairs_as;
use fandango_core::graph::GraphTraverse;
use fandango_core::typing::Node;

mod simple {
    use super::*;
    use fandango::Fandango;
    use fandango_core::graph::IntoGraph;
    use fandango_core::typing::{AsNode, Structured};
    use fandango_core::visitor::Visitor;
    use fandango_core::visitor::write::WriteVisitor;
    use petgraph::dot::{Config, Dot};
    use petgraph::graphmap::DiGraphMap;
    use std::error::Error;
    use tuple_list::tuple_list;

    #[derive(Fandango)]
    #[grammar = "tests/grammars/simple.fan"]
    pub struct Simple;

    #[test]
    fn parse() -> Result<(), Box<dyn Error>> {
        let graph = STRUCTURE.into_graph();

        const SAMPLE: &str = "1+2";

        let mut valid = false;

        let mut start = Simple::extract(SAMPLE)?;
        assert!(graph.contains_node(start.definition()));
        let expr = start.children().0;
        assert!(graph.contains_node(expr.definition()));
        if let nonterminal_expr_0::variant_0(expr) = expr.children().0 {
            assert!(graph.contains_node(expr.definition()));
            let (number, plus, expr) = expr.children();
            assert!(graph.contains_node(number.definition()));
            assert!(graph.contains_node(expr.definition()));
            assert_eq!(number.span().unwrap().as_str(), "1");
            assert_eq!(plus.span().unwrap().as_str(), "+");
            if let nonterminal_expr_0::variant_1(number) = expr.children().0 {
                assert!(graph.contains_node(number.definition()));
                assert_eq!(number.span().unwrap().as_str(), "2");
                valid = true;
            }
        }
        assert!(valid, "Parse did not match expected value!");

        assert_eq!(
            SAMPLE.as_bytes(),
            WriteVisitor::new(Vec::new())
                .visit(&mut start, 0)?
                .continue_value()
                .unwrap()
                .output()
        );
        assert_eq!(
            SAMPLE.as_bytes(),
            WriteVisitor::cacheless(Vec::new())
                .visit(&mut start, 0)?
                .continue_value()
                .unwrap()
                .output()
        );

        Ok(())
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
