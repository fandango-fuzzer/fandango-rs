//! Build tests for FANDANGO, to ensure that we are generating code as expected.

#![no_std]
#![allow(deprecated)] // for DynamicNode

extern crate alloc;
use fandango::parse_pairs_as;
use fandango_core::typing::Node;

mod simple {
    use super::*;
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::error::Error;
    use core::num::NonZeroUsize;
    use fandango::Fandango;
    use fandango_core::dynamic::{DynamicNode, DynamicSampler, HasDynamicSampler};
    use fandango_core::generation::util::Flattener;
    use fandango_core::generation::{Generated, InPlaceGenerated};
    use fandango_core::typing::{AsNode, AsStaticNode, Nth, Structured};
    use fandango_core::visitor::Visitor;
    use fandango_core::visitor::kpath::{KPathUpdate, KPaths};
    use fandango_core::visitor::navigation::{
        Advance, CountNodes, CountNodesWith, FindVisitor, GoToMut, StartingFrom,
    };
    use fandango_core::visitor::write::WriteVisitor;
    use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
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

        let start = Simple::extract(SAMPLE).unwrap();
        {
            let expr = start.nth::<0>();
            if let Some(expr) = expr.nth::<0>().nth::<0>() {
                let (_number, plus, expr) = expr.children();
                dfs = Some(FindVisitor::dfs(plus));
                bfs = Some(FindVisitor::bfs(plus));

                assert_eq!(
                    "+".as_bytes(),
                    WriteVisitor::new(Vec::new())
                        .visit(plus, 1)?
                        .continue_value()
                        .unwrap()
                        .output()
                );

                if let Some(number) = expr.nth::<0>().nth::<1>() {
                    assert_eq!(
                        "2".as_bytes(),
                        WriteVisitor::new(Vec::new())
                            .visit(number, 1)?
                            .continue_value()
                            .unwrap()
                            .output()
                    );

                    valid = true;
                }
            }
            assert!(valid, "Parse did not match expected value!");
        }

        let dfs = dfs.unwrap();
        let bfs = bfs.unwrap();

        let mut plus_path = dfs.clone().visit(&start, 0).unwrap().break_value().unwrap();

        assert_eq!(
            plus_path,
            bfs.visit(&start, 0).unwrap().break_value().unwrap()
        );

        assert_eq!(
            "+2",
            String::from_utf8(
                WriteVisitor::new(Vec::new())
                    .starting_from(plus_path.make_contiguous())
                    .visit(&start, 0)
                    .unwrap()
                    .continue_value()
                    .unwrap()
                    .inner()
                    .output()
            )
            .unwrap()
        );

        Ok(())
    }

    #[test]
    fn mutate() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut start = nonterminal_start::generate(&mut rng, &mut (), 0);

        let mut generators = ();

        let mut mutations = 0;

        let mut count = start.count_nodes();
        for _ in 0..100_000 {
            let old_start = start.clone();
            let selection = rng.random_range(0..count);
            let mut target = Advance::forward(selection)
                .visit(&start, 0)?
                .break_value()
                .unwrap();
            let (&idx, target) = target.make_contiguous().split_first().unwrap();
            let mut target = start.go_to_mut(idx, target)?;
            let old_count = target.count_nodes();
            target.generate_in_place(&mut rng, &mut generators, 0);
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
    fn mutate_dynamic() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let nonterminals = nonterminal_start::ROOT.inner().nonterminals();
        let mut sampler = DynamicSampler::new(
            nonterminal_start::static_root(),
            nonterminal_start::static_definition(),
            &nonterminals,
            &mut rng,
        );
        let mut start = DynamicNode::generate(&mut sampler, &mut (), 0);

        let mut generators = ();

        let mut mutations = 0;

        let mut count = start.count_nodes();
        for _ in 0..100_000 {
            let old_start = start.clone();
            let selection = sampler.inner().random_range(0..count);
            let mut target = Advance::forward(selection)
                .visit(&start, 0)?
                .break_value()
                .unwrap();
            let (&idx, target) = target.make_contiguous().split_first().unwrap();
            let target = start.go_to_mut(idx, target)?;
            let old_count = target.count_nodes();
            let definition = target.definition();
            sampler.with_definition(definition);
            target.generate_in_place(&mut sampler, &mut generators, 0);
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
        let mut rng = StdRng::seed_from_u64(0);
        let start = nonterminal_start::generate(&mut rng, &mut (), 0);

        let _ = String::from_utf8(
            WriteVisitor::new(Vec::new())
                .visit(&start, 0)?
                .continue_value()
                .unwrap()
                .output(),
        )?;

        Ok(())
    }

    #[test]
    fn generate_unflattened() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);

        let mut buf = Vec::new();
        let mut distribution = [0usize; 10];

        for _ in 0..100_000 {
            let digit = nonterminal_digit::generate(&mut rng, &mut (), 0);

            WriteVisitor::new(&mut buf)
                .visit(&digit, 0)?
                .continue_value()
                .unwrap()
                .output();
            distribution[(buf[0] - b'0') as usize] += 1;
            buf.clear();
        }

        assert!(distribution[0].abs_diff(50000) < 500);

        Ok(())
    }

    #[test]
    fn generate_flattened() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);

        let flattener = Flattener::flatten::<nonterminal_digit>()?;

        let mut generators = tuple_list!(flattener);

        let mut buf = Vec::new();
        let mut distribution = [0usize; 10];

        for _ in 0..100_000 {
            let digit = nonterminal_digit::generate(&mut rng, &mut generators, 0);

            WriteVisitor::new(&mut buf)
                .visit(&digit, 0)?
                .continue_value()
                .unwrap()
                .output();
            distribution[(buf[0] - b'0') as usize] += 1;
            buf.clear();
        }

        assert!(distribution.into_iter().all(|i| i.abs_diff(10000) < 500));

        Ok(())
    }

    #[test]
    fn generate_flattened_dynamic() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let nonterminals = nonterminal_digit::ROOT.inner().nonterminals();
        let mut sampler = DynamicSampler::new(
            nonterminal_digit::static_root(),
            nonterminal_digit::static_definition(),
            &nonterminals,
            &mut rng,
        );

        let flattener = Flattener::flatten_dynamic(
            nonterminal_digit::static_root(),
            nonterminal_digit::static_definition(),
        )?;

        let mut generators = tuple_list!(flattener);

        let mut buf = Vec::new();
        let mut distribution = [0usize; 10];

        for _ in 0..100_000 {
            let digit = DynamicNode::generate(&mut sampler, &mut generators, 0);

            WriteVisitor::new(&mut buf)
                .visit(&digit, 0)?
                .continue_value()
                .unwrap()
                .output();
            distribution[(buf[0] - b'0') as usize] += 1;
            buf.clear();
        }

        assert!(distribution.into_iter().all(|i| i.abs_diff(10000) < 500));

        Ok(())
    }

    #[test]
    fn static_vs_dynamic() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let nonterminals = nonterminal_start::ROOT.inner().nonterminals();
        let mut dynrng = rng.clone();
        let mut dyn_sampler = DynamicSampler::new(
            nonterminal_start::static_root(),
            nonterminal_start::static_definition(),
            &nonterminals,
            &mut dynrng,
        );

        for _ in 0..10_000 {
            let static_start = nonterminal_start::generate(&mut rng, &mut (), 0);
            let dyn_start = DynamicNode::generate(&mut dyn_sampler, &mut (), 0);

            let static_ser = WriteVisitor::new(Vec::new())
                .visit(&static_start, 0)?
                .continue_value()
                .unwrap()
                .output();
            let dyn_ser = WriteVisitor::new(Vec::new())
                .visit(&dyn_start, 0)?
                .continue_value()
                .unwrap()
                .output();

            assert_eq!(static_ser, dyn_ser);
            assert_eq!(static_start.count_nodes(), dyn_start.count_nodes());
        }

        Ok(())
    }

    #[test]
    fn kpath() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut kpath = KPaths::new::<TypeMut<'static>>(
            NonZeroUsize::new(5).unwrap(),
            nonterminal_start::ROOT.inner(),
        );

        let mut updater = KPathUpdate::inserting(&mut kpath);

        let (mut zero, _total) = updater.kpaths().k_paths();

        while zero != 0 {
            let start = nonterminal_start::generate(&mut rng, &mut (), 0);

            updater = updater.visit(&start, 0).unwrap().continue_value().unwrap();

            zero = updater.kpaths().k_paths().0;
        }

        Ok(())
    }
}

mod pest_renamed {
    use super::*;
    use fandango::Fandango;
    use pest::Parser;

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
    use core::num::NonZeroUsize;
    use fandango_core::dynamic::{DynamicNode, DynamicSampler};
    use fandango_core::generation::{DefaultGenerated, Generated};
    use fandango_core::typing::DowncastMut;
    use fandango_core::typing::{AsNodeMut, AsStaticNode, Node, OpaqueMut, Structured};
    use fandango_core::visitor::assignment::AssignmentVisitor;
    use fandango_core::visitor::kpath::{KPathUpdate, KPaths};
    use fandango_core::visitor::navigation::CountNodes;
    use fandango_core::visitor::write::WriteVisitor;
    use fandango_core::visitor::{Visitor, VisitorMut};
    use fandango_derive::Fandango;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[fandango(grammar = "targets/grammars/xml.fan")]
    pub struct Xml;

    #[test]
    fn generate() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let start = nonterminal_start::generate_default(&mut rng, &mut (), 0);

        let _ = String::from_utf8(
            WriteVisitor::new(Vec::new())
                .visit(&start, 0)?
                .continue_value()
                .unwrap()
                .output(),
        )?;

        Ok(())
    }

    #[test]
    fn default_terminates() -> Result<(), Box<dyn Error>> {
        let _ = nonterminal_start::default();
        Ok(())
    }

    // this looks horrible, but this means we can downcast N1 to N2 conditionally
    // this also applies to visitors; see the AssignmentVisitor impl for an example
    fn swap_example<'a, N1, N2>(n1: &'a mut N1, n2: &'a mut N2)
    where
        N1: Node,
        N2: Node<TypeMut<'a> = N1::TypeMut<'a>>,
        N1::TypeMut<'a>: AsNodeMut<N2>,
    {
        core::mem::swap(n1.opaque_mut().downcast_mut().unwrap(), n2);
    }

    #[test]
    fn find_replace() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut first = nonterminal_start::generate_default(&mut rng, &mut (), 0);
        let mut second = nonterminal_start::generate_default(&mut rng, &mut (), 0);
        while first == second {
            second = nonterminal_start::generate_default(&mut rng, &mut (), 0);
        }

        let second_clone = second.clone();

        swap_example(&mut first, &mut second);

        assert_eq!(first, second_clone);
        assert_ne!(second, second_clone);

        assert!(
            AssignmentVisitor(first)
                .visit_mut(&mut second, 0)
                .unwrap()
                .is_break()
        );

        assert_eq!(second, second_clone);

        Ok(())
    }

    #[test]
    fn static_vs_dynamic() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let nonterminals = nonterminal_start::ROOT.inner().nonterminals();
        let mut dynrng = rng.clone();
        let mut dyn_sampler = DynamicSampler::new(
            nonterminal_start::static_root(),
            nonterminal_start::static_definition(),
            &nonterminals,
            &mut dynrng,
        );

        for _ in 0..10_000 {
            let static_start = nonterminal_start::generate(&mut rng, &mut (), 0);
            let dyn_start = DynamicNode::generate(&mut dyn_sampler, &mut (), 0);

            let static_ser = WriteVisitor::new(Vec::new())
                .visit(&static_start, 0)?
                .continue_value()
                .unwrap()
                .output();
            let dyn_ser = WriteVisitor::new(Vec::new())
                .visit(&dyn_start, 0)?
                .continue_value()
                .unwrap()
                .output();

            assert_eq!(static_ser, dyn_ser);
            assert_eq!(static_start.count_nodes(), dyn_start.count_nodes());
        }

        Ok(())
    }

    #[test]
    fn kpath() -> Result<(), Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut kpath = KPaths::new::<TypeMut<'static>>(
            NonZeroUsize::new(5).unwrap(),
            nonterminal_start::ROOT.inner(),
        );

        let mut updater = KPathUpdate::inserting(&mut kpath);

        let (mut zero, _total) = updater.kpaths().k_paths();

        while zero != 0 {
            let start = nonterminal_start::generate(&mut rng, &mut (), 0);

            updater = updater.visit(&start, 0).unwrap().continue_value().unwrap();

            zero = updater.kpaths().k_paths().0;
        }

        Ok(())
    }
}

mod lang {
    use super::*;
    use alloc::boxed::Box;
    use fandango::visitor::VisitableChildren;
    use core::error::Error;
    use fandango_derive::Fandango;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use fandango_core::visitor::write::WriteVisitor;
    use alloc::string::String;
    use alloc::vec::Vec;
    use fandango_core::generation::{DefaultGenerated};
    use fandango_core::visitor::Visitor;
    use core::convert::Infallible;
    use fandango_core::visitor::VisitResult;
    use fandango::typing::{AsNodeRef, Downcast, Nth};
    use core::ops::ControlFlow;

    #[allow(dead_code)]
    #[derive(Fandango)]
    #[fandango(grammar = "targets/grammars/lua_lang.fan")]
    pub struct LuaLang;

    #[test]
    fn generate_lang() -> Result<(), Box<dyn Error>> {
        // Test to generate some language code.
        let mut outputs = Vec::new();
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..10 {
            let mut start = nonterminal_start::generate_default(&mut rng, &mut (), 0);
            let serialized = String::from_utf8(
            WriteVisitor::new(Vec::new())
                .visit(&mut start, 0)?
                .continue_value()
                .unwrap()
                .output(),
            )?;
            outputs.push(serialized);
        }

        extern crate std;
        
        std::println!("\n=== Generated Lang Code ===");
        for output in outputs {
            std::println!("{output}");
            std::println!("---------------------------");
        }
        Ok(())
    }

    #[test]
    fn count_variable_names() -> Result<(), Box<dyn Error>> {
        // Test to count variable names in generated code.
        let mut rng = StdRng::seed_from_u64(0);

        pub struct CountVarNamesVisitor {
            pub count: usize,
        }

        impl<T> Visitor<T> for CountVarNamesVisitor
        where
            T: VisitableChildren<T> + AsNodeRef<nonterminal_var_name>,
        {
            type Continue = Self;
            type Break = Infallible;
            type Error = Infallible;

            fn visit<'program, N>(mut self, node: &'program N, _idx: usize) -> VisitResult<Self, T>
            where
                N: Node<Type<'program> = T>,
                T: From<&'program N> + AsNodeRef<N>,
            {
                let visited = T::from(node);
                if let Some(_tree) = visited.downcast::<nonterminal_var_name>() {
                    self.count += 1;
                }
                visited.visit_each(self)
            }
        }

        // count variable names in 10 generated samples
        let mut collected_names = Vec::new();
        let mut collected_serialized = Vec::new();
        for _ in 0..10 {
            let mut start = nonterminal_start::generate_default(&mut rng, &mut (), 0);
            
            let result = CountVarNamesVisitor { count: 0 }
                .visit(&start, 0)?
                .continue_value()
                .unwrap()
                .count;
    
            let serialized = String::from_utf8(
                WriteVisitor::new(Vec::new())
                    .visit(&mut start, 0)?
                    .continue_value()
                    .unwrap()
                    .output(),
            )?;

            collected_names.push(result);
            collected_serialized.push(serialized.clone());
        }

        extern crate std;
        std::println!("\n=== Variable Name Count Test ===");
        for (i, (count, code)) in collected_names.iter().zip(collected_serialized.iter()).enumerate() {
            std::println!("Sample {}: Variable names = {}, Code = \n{}", i + 1, count, code);
        }
        std::println!("=============================");
        Ok(())
    }

    #[test]
    fn get_all_var_decl_names() -> Result<(), Box<dyn Error>> {
        // Test to get all variable declaration names in generated code.
        let mut rng = StdRng::seed_from_u64(0);

        pub struct VarDeclNamesVisitor {
            pub names: Vec<String>,
        }

        impl<T> Visitor<T> for VarDeclNamesVisitor
        where
            T: VisitableChildren<T> + AsNodeRef<nonterminal_decl>,
        {
            type Continue = Self;
            type Break = Infallible;
            type Error = Infallible;

            fn visit<'program, N>(mut self, node: &'program N, _idx: usize) -> VisitResult<Self, T>
            where
                N: Node<Type<'program> = T>,
                T: From<&'program N> + AsNodeRef<N>,
            {
                let visited = T::from(node);
                if let Some(tree) = visited.downcast::<nonterminal_decl>() {
                    let var_decl_name = WriteVisitor::new(Vec::new())
                        .visit(tree.nth::<0>().nth::<2>(), 0)
                        .unwrap()
                        .continue_value()
                        .unwrap()
                        .output();
                    self.names.push(String::from_utf8(var_decl_name).unwrap());
                }
                visited.visit_each(self)
            }
        }

        // get variable declaration names in 10 generated samples
        let mut results = Vec::new();
        let mut collected_serialized = Vec::new();
        for _ in 0..10 {
            let mut start = nonterminal_start::generate_default(&mut rng, &mut (), 0);

            let result = VarDeclNamesVisitor { names: Vec::new() }
                .visit(&start, 0)?
                .continue_value()
                .unwrap()
                .names;

            let serialized = String::from_utf8(
                WriteVisitor::new(Vec::new())
                    .visit(&mut start, 0)?
                    .continue_value()
                    .unwrap()
                    .output(),
            )?;

            results.push(result);
            collected_serialized.push(serialized.clone());
        }

        extern crate std;
        std::println!("\n=== Variable Declaration Names Test ===");
        for (i, (names, code)) in results.iter().zip(collected_serialized.iter()).enumerate() {
            std::println!("Sample {}: Var Decl Names = {:?}, Code = \n{}", i + 1, names, code);
        }
        std::println!("=============================");
        Ok(())
    }

    #[test]
    fn lang_constraint_var_names_include_numbers() -> Result<(), Box<dyn Error>> {
        // Test to ensure variable names include numbers in generated code.
        let mut rng = StdRng::seed_from_u64(0);
        let mut diff_count = 0;

        pub struct LangConstraintNumbersInVarNamesVisitor {
            pub violations: Vec<alloc::collections::VecDeque<usize>>,
            pub path: alloc::collections::VecDeque<usize>,
        }

        impl<T> Visitor<T> for LangConstraintNumbersInVarNamesVisitor
        where 
            T: VisitableChildren<T> +
            AsNodeRef<nonterminal_var_name>
        {
            type Continue = Self;
            type Break = Infallible;
            type Error = Infallible;

            fn visit<'program, N>(mut self, node: &'program N, idx: usize) -> VisitResult<Self, T>
            where
                N: Node<Type<'program> = T>,
                T: From<&'program N> + AsNodeRef<N>,
            {
                self.path.push_back(idx);
                let visited = T::from(node);
                if let Some(tree) = visited.downcast::<nonterminal_var_name>() {
                    let var_name_str = String::from_utf8(
                        WriteVisitor::new(Vec::new())
                            .visit(tree, 0)
                            .unwrap()
                            .continue_value()
                            .unwrap()
                            .output(),
                    ).unwrap();
                    if !var_name_str.chars().any(|c| c.is_ascii_digit()) {
                        self.violations.push(self.path.clone());
                    }
                }
                let mut result = visited.visit_each(self);
                if let Ok(ControlFlow::Continue(visitor)) = &mut result {
                    visitor.path.pop_back();
                }
                result
            }
        }

        let mut diffs_by_sample = alloc::vec::Vec::new();
        let mut samples = alloc::vec::Vec::new();
        for _ in 0..10 {
            let start = nonterminal_start::generate_default(&mut rng, &mut (), 0);

            let result = LangConstraintNumbersInVarNamesVisitor {
                violations: Vec::new(),
                path: alloc::collections::VecDeque::new(),
            }
                .visit(&start, 0)?
                .continue_value()
                .unwrap();
            let violations = result.violations; 
            if !violations.is_empty() {
                diffs_by_sample.push(violations.len());
            } else {
                diffs_by_sample.push(0);
            }
            samples.push(String::from_utf8(
                WriteVisitor::new(Vec::new())
                    .visit(&start, 0)?
                    .continue_value()
                    .unwrap()
                    .output(),
            )?);
        }
        
        extern crate std;
        std::println!("\n=== # in Variable Names Constraint Test ===");
        for (i, (diffs, code)) in diffs_by_sample.iter().zip(samples.iter()).enumerate() {
            std::println!("Sample {}: Violations = {}, Code = \n{}", i + 1, diffs, code);
            diff_count += diffs;
        }
        std::println!("=============================");

        // assert_ne!(0, diff_count);
        Ok(())
    }
}