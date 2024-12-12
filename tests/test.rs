//! Build tests for FANDANGO, to ensure that we are generating code as expected.

use fandango::Parser;
use fandango::parse_pairs_as;
use fandango_core::typing::Node;

mod simple {
    use super::*;
    use fandango::Fandango;
    use fandango_core::generation::DefaultGenerated;
    use fandango_core::graph::IntoGraph;
    use fandango_core::typing::AsNode;
    use fandango_core::visitor::Visitor;
    use fandango_core::visitor::navigation::FindVisitor;
    use fandango_core::visitor::write::WriteVisitor;
    use rand::thread_rng;
    use std::error::Error;

    #[derive(Fandango)]
    #[grammar = "tests/grammars/simple.fan"]
    pub struct Simple;

    #[test]
    fn parse() -> Result<(), Box<dyn Error>> {
        let graph = STRUCTURE.into_graph();

        const SAMPLE: &str = "1+2";

        let mut valid = false;

        let mut dfs = None;
        let mut bfs = None;

        let mut start = Simple::extract(SAMPLE)?;
        {
            assert!(graph.contains_node(start.definition()));
            let expr = start.children_mut().0;
            assert!(graph.contains_node(expr.definition()));
            if let nonterminal_expr_0::variant_0(expr) = expr.children_mut().0 {
                assert!(graph.contains_node(expr.definition()));
                let (number, plus, expr) = expr.children_mut();
                assert!(graph.contains_node(number.definition()));
                assert!(graph.contains_node(expr.definition()));
                assert_eq!(number.span().unwrap().as_str(), "1");
                assert_eq!(plus.span().unwrap().as_str(), "+");

                dfs = Some(FindVisitor::dfs(plus));
                bfs = Some(FindVisitor::bfs(plus));

                assert_eq!(
                    "+".as_bytes(),
                    WriteVisitor::caching(Vec::new())
                        .visit(plus, 1)?
                        .continue_value()
                        .unwrap()
                        .output()
                );
                assert_eq!(
                    "+".as_bytes(),
                    WriteVisitor::cacheless(Vec::new())
                        .visit(plus, 1)?
                        .continue_value()
                        .unwrap()
                        .output()
                );

                if let nonterminal_expr_0::variant_1(number) = expr.children_mut().0 {
                    assert!(graph.contains_node(number.definition()));
                    assert_eq!(number.span().unwrap().as_str(), "2");

                    valid = true;
                }
            }
            assert!(valid, "Parse did not match expected value!");
        }

        let plus_path = dfs
            .unwrap()
            .visit(&mut start, 0)
            .unwrap()
            .break_value()
            .unwrap();

        assert_eq!(
            plus_path,
            bfs.unwrap()
                .visit(&mut start, 0)
                .unwrap()
                .break_value()
                .unwrap()
        );

        assert_eq!(
            "+2",
            String::from_utf8(
                WriteVisitor::caching_from(Vec::new(), plus_path.clone())
                    .visit(&mut start, 0)?
                    .continue_value()
                    .unwrap()
                    .output()
            )
            .unwrap()
        );
        assert_eq!(
            "+2",
            String::from_utf8(
                WriteVisitor::cacheless_from(Vec::new(), plus_path)
                    .visit(&mut start, 0)?
                    .continue_value()
                    .unwrap()
                    .output()
            )
            .unwrap()
        );

        Ok(())
    }

    #[test]
    fn generate() -> Result<(), Box<dyn Error>> {
        let mut rng = thread_rng();
        let mut start = nonterminal_start::generate_default(&mut rng);

        let serialized = String::from_utf8(
            WriteVisitor::caching(Vec::new())
                .visit(&mut start, 0)?
                .continue_value()
                .unwrap()
                .output(),
        )?;

        println!("{}", serialized);
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

mod xml {
    use fandango::typing::Node;
    use fandango_core::generation::DefaultGenerated;
    use fandango_core::visitor::Visitor;
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_derive::Fandango;
    use rand::thread_rng;
    use std::error::Error;

    #[derive(Fandango)]
    #[grammar = "tests/grammars/xml.fan"]
    pub struct Xml;

    #[test]
    fn generate() -> Result<(), Box<dyn Error>> {
        let mut rng = thread_rng();
        let mut start = nonterminal_start::generate_default(&mut rng);

        let serialized = String::from_utf8(
            WriteVisitor::caching(Vec::new())
                .visit(&mut start, 0)?
                .continue_value()
                .unwrap()
                .output(),
        )?;

        println!("{}", serialized);
        Ok(())
    }
}
