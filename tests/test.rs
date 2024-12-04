//! Build tests for FANDANGO, to ensure that we are generating code as expected.

mod macros {
    use fandango::Parser;
    use fandango::parse_pairs_as;
    use fandango_core::typing::{Children, Node};

    mod simple {
        use fandango::Fandango;

        #[derive(Fandango)]
        #[grammar = "tests/grammars/simple.fan"]
        pub struct Simple;
    }

    #[test]
    fn parse() -> Result<(), simple::ParseError> {
        use simple::*;

        println!("{}", _PEST_SOURCE);

        const SAMPLE: &str = "1+2";

        let start = Simple::extract(SAMPLE)?;
        let expr = start.children().0;
        if let nonterminal_expr_0::variant_0(expr) = expr.children().0 {
            let (number, plus, expr) = expr.children();
            assert_eq!(number.span().unwrap().as_str(), "1");
            assert_eq!(plus.span().unwrap().as_str(), "+");
            if let nonterminal_expr_0::variant_1(number) = expr.children().0 {
                assert_eq!(number.span().unwrap().as_str(), "2");
                return Ok(());
            }
        }

        panic!("Parse did not match expected value!");
    }

    mod pest_renamed {
        use fandango::Fandango;

        #[derive(Fandango)]
        #[grammar = "tests/grammars/pest-renamed.fan"]
        pub struct PestRenamed;
    }

    #[test]
    fn pest_name_sanity() -> Result<(), pest_renamed::ParseError> {
        use pest_renamed::*;

        println!("{}", _PEST_SOURCE);

        const SAMPLE: &str = "hello!";

        let (start,) = parse_pairs_as!(PestRenamed::parse(Rule::start, SAMPLE)?, (Rule::start,));
        let (actual,) = parse_pairs_as!(start.into_inner(), (Rule::pest,));
        assert_eq!(actual.as_span().as_str(), SAMPLE);

        let start = PestRenamed::extract(SAMPLE)?;
        let pest = start.children().0;
        let string = pest.children().0.children().0;

        assert_eq!(string, SAMPLE);

        Ok(())
    }
}
