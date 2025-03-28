//! Build tests for FANDANGO, to ensure that we are generating code as expected.

#![no_std]

use fandango::parse_pairs_as;
use fandango::Parser;
use fandango_core::typing::Node;

extern crate alloc;

mod simple {
    use super::*;
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::error::Error;
    use fandango::Fandango;
    use fandango_core::generation::util::Flattener;
    use fandango_core::generation::{Generated, InPlaceGenerated};
    use fandango_core::visitor::navigation::{Advance, CountNodes, CountNodesWith, FindVisitor};
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_core::visitor_chain;
    use rand::rng;
    use rand::Rng;
    use tuple_list::tuple_list;

    #[derive(Fandango)]
    #[fandango(grammar = "tests/grammars/simple.fan")]
    pub struct Simple;

    #[test]
    fn parse() -> Result<(), Box<dyn Error>> {
        const SAMPLE: &str = "1+2";

        let mut valid = false;

        let mut dfs = None;
        let mut bfs = None;

        let mut start = Simple::extract(SAMPLE).unwrap();
        {
            let expr = start.children_mut().0;
            if let nonterminal_expr_0::variant_0(expr) = expr.children_mut().0 {
                let (number, plus, expr) = expr.children_mut();
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
                    assert_eq!(number.span().unwrap().as_str(), "2");

                    valid = true;
                }
            }
            assert!(valid, "Parse did not match expected value!");
        }

        let dfs = dfs.unwrap();
        let bfs = bfs.unwrap();

        let plus_path = dfs
            .clone()
            .visit(&mut start, 0)
            .unwrap()
            .break_value()
            .unwrap();

        assert_eq!(
            plus_path,
            bfs.visit(&mut start, 0).unwrap().break_value().unwrap()
        );

        assert_eq!(
            "+2",
            String::from_utf8(
                visitor_chain!(
                    &mut start,
                    0,
                    dfs.clone(),
                    WriteVisitor::caching(Vec::new())
                )
                .continue_value()
                .unwrap()
                .output()
            )
            .unwrap()
        );
        assert_eq!(
            "+2",
            String::from_utf8(
                visitor_chain!(&mut start, 0, dfs, WriteVisitor::caching(Vec::new()))
                    .continue_value()
                    .unwrap()
                    .output()
            )
            .unwrap()
        );

        Ok(())
    }

    #[test]
    fn mutate() -> Result<(), Box<dyn Error>> {
        let mut rng = rng();
        let mut start = nonterminal_start::generate(&mut rng, &mut ());

        let mut generators = ();

        let mut mutations = 0;

        let mut count = start.count_nodes();
        for _ in 0..100_000 {
            let old_start = start.clone();
            let selection = rng.random_range(0..count);
            let mut target = Advance::forward(selection)
                .visit(&mut start, 0)?
                .break_value()
                .unwrap();
            let old_count = target.count_nodes();
            target.generate_in_place(&mut rng, &mut generators);
            let new_count = target.count_nodes();
            count = count - old_count + new_count;
            if old_start != start {
                mutations += 1;
            }
        }

        assert_ne!(0, mutations);

        Ok(())
    }

    #[test]
    fn generate() -> Result<(), Box<dyn Error>> {
        let mut rng = rng();
        let mut start = nonterminal_start::generate(&mut rng, &mut ());

        let serialized = String::from_utf8(
            WriteVisitor::caching(Vec::new())
                .visit(&mut start, 0)?
                .continue_value()
                .unwrap()
                .output(),
        )?;

        extern crate std;

        std::println!("{}", serialized);
        Ok(())
    }

    #[test]
    fn generate_unflattened() -> Result<(), Box<dyn Error>> {
        let mut rng = rng();

        let mut buf = Vec::new();
        let mut distribution = [0usize; 10];

        for _ in 0..100_000 {
            let mut digit = nonterminal_digit::generate(&mut rng, &mut ());

            WriteVisitor::caching(&mut buf)
                .visit(&mut digit, 0)?
                .continue_value()
                .unwrap()
                .output();
            distribution[(buf[0] - b'0') as usize] += 1;
            buf.clear();
        }

        extern crate std;

        std::println!("{distribution:?}");

        Ok(())
    }

    #[test]
    fn generate_flattened() -> Result<(), Box<dyn Error>> {
        let mut rng = rng();

        let flattener = Flattener::new().flatten::<nonterminal_digit>()?;

        let mut generators = tuple_list!(flattener);

        let mut buf = Vec::new();
        let mut distribution = [0usize; 10];

        for _ in 0..100_000 {
            let mut digit = nonterminal_digit::generate(&mut rng, &mut generators);

            WriteVisitor::caching(&mut buf)
                .visit(&mut digit, 0)?
                .continue_value()
                .unwrap()
                .output();
            distribution[(buf[0] - b'0') as usize] += 1;
            buf.clear();
        }

        extern crate std;

        std::println!("{distribution:?}");

        Ok(())
    }
}

mod pest_renamed {
    use super::*;
    use fandango::Fandango;

    #[derive(Fandango)]
    #[fandango(grammar = "tests/grammars/pest-renamed.fan")]
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

        assert_eq!(string, SAMPLE.as_bytes());

        Ok(())
    }
}

mod xml {
    use alloc::boxed::Box;

    use alloc::string::String;
    use alloc::vec::Vec;
    use core::error::Error;
    use fandango_core::generation::DefaultGenerated;
    use fandango_core::typing::{AsNodeMut, Node};
    use fandango_core::visitor::assignment::AssignmentVisitor;

    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::rng;

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[fandango(grammar = "tests/grammars/xml.fan")]
    pub struct Xml;

    #[test]
    fn generate() -> Result<(), Box<dyn Error>> {
        let mut rng = rng();
        let mut start = nonterminal_start::generate_default(&mut rng, &mut ());

        let serialized = String::from_utf8(
            WriteVisitor::caching(Vec::new())
                .visit(&mut start, 0)?
                .continue_value()
                .unwrap()
                .output(),
        )?;

        extern crate std;

        std::println!("{}", serialized);
        Ok(())
    }

    // this looks horrible, but this means we can effectively downcast N1 to N2 conditionally
    // this also applies to visitors; see the AssignmentVisitor impl for an example
    fn swap_example<'a, N1, N2>(n1: &'a mut N1, n2: &'a mut N2)
    where
        N1: Node,
        N2: Node<TypeMut<'a> = N1::TypeMut<'a>>,
        N1::TypeMut<'a>: From<&'a mut N1> + AsNodeMut<N2>,
    {
        core::mem::swap(<N1::TypeMut<'a>>::from(n1).as_node_mut().unwrap(), n2);
    }

    #[test]
    fn find_replace() -> Result<(), Box<dyn Error>> {
        let mut rng = rng();
        let mut first = nonterminal_start::generate_default(&mut rng, &mut ());
        let mut second = nonterminal_start::generate_default(&mut rng, &mut ());
        while first == second {
            second = nonterminal_start::generate_default(&mut rng, &mut ());
        }

        let second_clone = second.clone();

        swap_example(&mut first, &mut second);

        assert_eq!(first, second_clone);
        assert_ne!(second, second_clone);

        assert!(AssignmentVisitor(first)
            .visit(&mut second, 0)
            .unwrap()
            .is_break());

        assert_eq!(second, second_clone);

        Ok(())
    }
}

mod ssl {
    use alloc::boxed::Box;

    use alloc::string::String;
    use alloc::vec::Vec;
    use core::error::Error;
    use fandango_core::generation::DefaultGenerated;
    use fandango_core::typing::{AsNodeMut, Node};
    use fandango_core::visitor::assignment::AssignmentVisitor;

    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::Visitor;
    use fandango_derive::Fandango;
    use rand::rng;

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[fandango(grammar = "tests/grammars/ssl.fan", parse = false)]
    pub struct Ssl;

    #[test]
    fn generate() -> Result<(), Box<dyn Error>> {
        let mut rng = rng();
        let mut start = nonterminal_start::generate_default(&mut rng, &mut ());

        let serialized = WriteVisitor::caching(Vec::new())
            .visit(&mut start, 0)?
            .continue_value()
            .unwrap()
            .output();

        extern crate std;

        std::println!("{:?}", serialized);
        Ok(())
    }
}
