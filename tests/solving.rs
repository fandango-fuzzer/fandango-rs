//! Demonstration of using egg for constraint minimisation.

#![allow(missing_docs)]

use egg::{rewrite as rw, *};
use fandango_core::graph::{FandangoNode, IntoGraph};
use fandango_core::lang::{Nonterminal, Tagged};
use fandango_core::typing::AsNode;
use fandango_derive::Fandango;
use pest::Span;
use petgraph::algo;
use petgraph::graphmap::DiGraphMap;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;

#[allow(dead_code)]
#[derive(Fandango)]
#[grammar = "tests/grammars/xml.fan"]
pub struct Xml;

define_language! {
    enum FandangoConstraintLang {
        "=" = Eq([Id; 2]),
        "&" = And([Id; 2]),
        "|" = Or([Id; 2]),
        "!" = Not(Id),
        "length" = Length(Id),
        "typed" = Typed([Id; 2]),
        "str" = Stringified(Id),
        "concat" = Concatenation([Id; 2]),
        "variant" = Variant(Id),
        "access" = Access([Id; 2]),
        Lit(bool),
        Type(usize),
        Var(Symbol),
    }
}

pub struct NodesPresent<'program, 'source> {
    type_map: Vec<FandangoNode<'program, 'source>>,
    reachable: HashMap<FandangoNode<'program, 'source>, HashSet<FandangoNode<'program, 'source>>>,
}

impl<'program, 'source> NodesPresent<'program, 'source> {
    fn new(
        type_graph: &DiGraphMap<FandangoNode<'program, 'source>, Span<'source>>,
        type_map: Vec<FandangoNode<'program, 'source>>,
    ) -> Self {
        let mut reachable = HashMap::new();
        for &node in &type_map {
            let mut weights = algo::dijkstra(&type_graph, node, None, |_| 1)
                .into_keys()
                .collect::<HashSet<_>>();
            weights.remove(&node);
            reachable.insert(node, weights);
        }
        Self {
            type_map,
            reachable,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct NodeCost<'program, 'source> {
    is_access: bool,
    untyped_access: HashSet<Id>,
    observed: HashSet<FandangoNode<'program, 'source>>,
    ast_size: usize,
}

impl NodeCost<'_, '_> {
    fn union(&self, id: Id, other: Self) -> Self {
        Self {
            is_access: false,
            untyped_access: self
                .untyped_access
                .union(&other.untyped_access)
                .copied()
                .chain(self.is_access.then_some(id))
                .chain(other.is_access.then_some(id))
                .collect(),
            observed: self.observed.union(&other.observed).copied().collect(),
            ast_size: self.ast_size.saturating_add(other.ast_size),
        }
    }
}

impl PartialEq<Self> for NodeCost<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other).unwrap().is_eq()
    }
}

impl PartialOrd for NodeCost<'_, '_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for NodeCost<'_, '_> {}

impl Ord for NodeCost<'_, '_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.untyped_access
            .len()
            .cmp(&other.untyped_access.len())
            .then_with(|| self.observed.len().cmp(&other.observed.len()))
            .then_with(|| self.ast_size.cmp(&other.ast_size))
    }
}

impl<'program, 'source> CostFunction<FandangoConstraintLang> for NodesPresent<'program, 'source> {
    type Cost = NodeCost<'program, 'source>;

    fn cost<C>(&mut self, enode: &FandangoConstraintLang, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        match enode {
            FandangoConstraintLang::Typed([_, value]) => {
                let mut cost = enode.fold(
                    NodeCost {
                        ast_size: 1,
                        ..Default::default()
                    },
                    |sum, id| sum.union(id, costs(id)),
                );
                cost.untyped_access.remove(value);
                cost
            }
            FandangoConstraintLang::Access(_) => {
                NodeCost {
                    // we have to infer from the type what the cost of the access is
                    is_access: true,
                    ..Default::default()
                }
            }
            FandangoConstraintLang::Type(node) => {
                NodeCost {
                    // we have to infer from the type what the cost of the access is
                    observed: self.reachable[&self.type_map[*node]]
                        .iter()
                        .copied()
                        .collect(),
                    ..Default::default()
                }
            }
            _ => enode.fold(
                NodeCost {
                    ast_size: 1,
                    ..Default::default()
                },
                |sum, id| sum.union(id, costs(id)),
            ),
        }
    }
}

#[test]
fn egg_demo() -> Result<(), Box<dyn Error>> {
    let mut rules: Vec<Rewrite<FandangoConstraintLang, _>> = vec![
        rw!("eq"; "(= ?x ?x)" => "true"),
        rw!("not-1"; "(! true)" => "false"),
        rw!("not-2"; "(! false)" => "true"),
        rw!("commute-eq"; "(= ?x ?y)" => "(= ?y ?x)"),
        rw!("commute-eq-and"; "(& (= ?x ?y) (= ?y ?z))" => "(& (= ?x ?y) (= ?x ?z))"),
        rw!("commute-and"; "(& ?x ?y)" => "(& ?y ?x)"),
        rw!("exchange-and"; "(& ?x (& ?y ?z))" => "(& ?y (& ?x ?z))"),
        rw!("dist-and"; "(& ?x (| ?y ?z))" => "(| (& ?x ?y) (& ?x ?z))"),
        rw!("factor-and"; "(| (& ?x ?y) (& ?x ?z))" => "(& ?x (| ?y ?z))"),
        rw!("factor-simp-and-1"; "(| (& ?x ?y) ?x)" => "?x"),
        rw!("factor-simp-and-2"; "(| (& ?x ?y) (! ?x))" => "(| ?y (! ?x))"),
        rw!("unsat-and"; "(& ?x (! ?x))" => "false"),
        rw!("commute-or"; "(| ?x ?y)" => "(| ?y ?x)"),
        rw!("exchange-or"; "(| ?x (| ?y ?z))" => "(| ?y (| ?x ?z))"),
        rw!("sat-or"; "(| ?x (! ?x))" => "true"),
        rw!("simp-and-1"; "(& false ?x)" => "false"),
        rw!("simp-and-2"; "(& true ?x)" => "?x"),
        rw!("simp-or-1"; "(| true ?x)" => "true"),
        rw!("simp-or-2"; "(| false ?x)" => "?x"),
        rw!("demorgan-1"; "(! (| ?x ?y))" => "(& (! ?x) (! ?y))"),
        rw!("demorgan-1-inv"; "(& (! ?x) (! ?y))" => "(! (| ?x ?y))"),
        rw!("demorgan-2"; "(! (& ?x ?y))" => "(| (! ?x) (! ?y))"),
        rw!("demorgan-2-inv"; "(| (! ?x) (! ?y))" => "(! (& ?x ?y))"),
        rw!("simp-str-deriv"; "(= (str (typed ?a ?x)) (str (typed ?a ?y)))" => "(= (typed ?a ?x) (typed ?a ?y))"),
        rw!("simp-typed"; "(= (typed ?a (typed ?a ?x)) (typed ?a (typed ?a ?y)))" => "(= (typed ?a ?x) (typed ?a ?y))"),
    ];

    let graph = nonterminal_start::root().into_graph();
    let mut variant_map = graph.nodes().map(|n1| (n1, graph.edges(n1))).fold(
        HashMap::<_, Vec<_>>::new(),
        |mut collected, (n1, edges)| {
            collected
                .entry(n1)
                .or_default()
                .extend(edges.map(|(_, n2, &e)| Tagged::new(n2, e)));
            collected
        },
    );

    let mut type_map = Vec::new();
    let mut type_cache = HashMap::new();

    for node in graph.nodes() {
        type_cache.insert(node, type_map.len());
        type_map.push(node);
    }

    for (node, children) in &mut variant_map {
        let &node_type = type_cache.get(node).unwrap();

        let mut base;
        children.sort(); // into visible order
        match node {
            FandangoNode::Alternative(_) => {
                // "the variants and values of those variants are equal"
                base = "false".to_string();
                for (idx, child) in children.iter().enumerate() {
                    let &child_type = type_cache.get(child.inner()).unwrap();
                    base = format!(
                        r#"(|
                            (=
                                (typed
                                    {child_type}
                                    (access
                                        (typed {node_type} ?x)
                                        {idx}
                                    )
                                )
                                (typed
                                    {child_type}
                                    (access
                                        (typed {node_type} ?y)
                                        {idx}
                                    )
                                )
                            )
                            {base}
                        )"#
                    );
                }
                base = format!(
                    r#"(&
                        (=
                            (variant (typed {node_type} ?x))
                            (variant (typed {node_type} ?y))
                        )
                        {base}
                    )"#
                );
            }
            FandangoNode::Nonterminal(_) | FandangoNode::Concatenation(_) => {
                // "each child is equal"
                base = "true".to_string();
                for (idx, child) in children.iter().enumerate() {
                    let &child_type = type_cache.get(child.inner()).unwrap();
                    base = format!(
                        r#"(&
                            (=
                                (typed
                                    {child_type}
                                    (access
                                        (typed
                                            {node_type}
                                            ?x
                                        )
                                        {idx}
                                    )
                                )
                                (typed
                                    {child_type}
                                    (access
                                        (typed
                                            {node_type}
                                            ?y
                                        )
                                        {idx}
                                    )
                                )
                            )
                            {base}
                        )"#
                    );
                }
            }
            FandangoNode::String(_) => {
                base = "true".to_string(); // string comparisons of the same type are always equal
            }
            FandangoNode::Operator(_) => {
                continue; // no rewriting; this could be infinitely large
            }
            FandangoNode::Production(_) => continue, // nothing to do
            _ => unimplemented!("Only implemented for unelided language items."),
        }

        rules.push(Rewrite::new(
            format!("node-{node_type}-eq"),
            format!("(= (typed {node_type} ?x) (typed {node_type} ?y))").parse::<Pattern<_>>()?,
            base.parse::<Pattern<_>>()?,
        )?);
    }

    let xml_attr_nonterminal = Nonterminal::new("xml_attribute");
    let xml_attr_nonterminal = FandangoNode::Nonterminal(&xml_attr_nonterminal);
    let xml_attr = type_cache[&xml_attr_nonterminal];
    let xml_attr_0 = type_cache[variant_map[&xml_attr_nonterminal][0].inner()];
    let xml_attr_0_0 =
        type_cache[variant_map[variant_map[&xml_attr_nonterminal][0].inner()][0].inner()];

    let attr_quant = format!(
        r#"(|
            (=
                (str (typed {xml_attr} first))
                (str (typed {xml_attr} second))
            )
            (! (=
                (str (typed {xml_attr_0_0} (access (typed {xml_attr_0} (access (typed {xml_attr} first) 0)) 0)))
                (str (typed {xml_attr_0_0} (access (typed {xml_attr_0} (access (typed {xml_attr} second) 0)) 0)))
            ))
        )"#
    );

    let cost = NodesPresent::new(&graph, type_map);

    let start = attr_quant.parse()?;
    let runner = Runner::<_, _, ()>::new(()).with_expr(&start).run(&rules);
    let extractor = Extractor::new(&runner.egraph, cost);
    let (_, best_expr) = extractor.find_best(runner.roots[0]);

    println!("{best_expr}");

    Ok(())
}
