//! Build tests for FANDANGO, to ensure that we are generating code as expected.

mod macros {
    use fandango::Parser;
    use fandango::parse_pairs_as;

    mod simple {
        use fandango::Fandango;

        #[derive(Fandango)]
        #[grammar = "tests/grammars/simple.fan"]
        pub struct Simple;
    }

    #[test]
    fn parse() {
        use simple::*;

        println!("{}", _PEST_SOURCE);

        const SAMPLE: &str = "1+2";

        let (start,) = parse_pairs_as!(Simple::parse(Rule::start, SAMPLE).unwrap(), (Rule::start,));

        assert_eq!(start.as_span().as_str(), SAMPLE); // consume whole string

        let (expr,) = parse_pairs_as!(start.into_inner(), (Rule::expr,));
        let (number, expr) = parse_pairs_as!(expr.into_inner(), (Rule::number, Rule::expr));

        let (non_zero,) = parse_pairs_as!(number.into_inner(), (Rule::non_zero,));
        assert_eq!(non_zero.as_span().as_str(), "1");

        let (number,) = parse_pairs_as!(expr.into_inner(), (Rule::number,));
        let (non_zero,) = parse_pairs_as!(number.into_inner(), (Rule::non_zero,));
        assert_eq!(non_zero.as_span().as_str(), "2");
    }

    mod pest_renamed {
        use fandango::Fandango;

        #[derive(Fandango)]
        #[grammar = "tests/grammars/pest-renamed.fan"]
        pub struct PestRenamed;
    }

    #[test]
    fn pest_name_sanity() {
        use pest_renamed::*;

        println!("{}", _PEST_SOURCE);

        const SAMPLE: &str = "hello!";

        let (start,) = parse_pairs_as!(
            PestRenamed::parse(Rule::start, SAMPLE).unwrap(),
            (Rule::start,)
        );
        let (actual,) = parse_pairs_as!(start.into_inner(), (Rule::pest,));
        assert_eq!(actual.as_span().as_str(), SAMPLE);
    }
}
