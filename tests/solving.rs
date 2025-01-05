//! Demonstration of using egg for constraint minimisation.

#![allow(missing_docs)]

use clap::Parser;
use egg::{rewrite as rw, *};
use fandango_core::generation::Generated;
use fandango_core::graph::{FandangoNode, IntoGraph};
use fandango_core::lang::{Nonterminal, Tagged};
use fandango_core::typing::{AsNode, Node};
use fandango_core::visitor::{VisitResult, VisitableChildren, Visitor};
use fandango_derive::Fandango;
use pest::Span;
use petgraph::algo;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{EdgeFiltered, EdgeRef, NodeFiltered};
use rand::rngs::StdRng;
use rand::{thread_rng, SeedableRng};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{ControlFlow, Deref, Index};
use std::str::FromStr;
use std::{fmt, mem};

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
        "ite" = Ite([Id; 3]),
        "X" = Invalid,
        Bool(bool),
        Type(usize),
        Concrete(usize),
        Var(Symbol),
    }
}

pub struct NodesPresent {
    type_map: Vec<NodeIndex>,
    reachable: HashMap<NodeIndex, HashSet<NodeIndex>>,
}

impl NodesPresent {
    fn new(type_graph: &DiGraph<FandangoNode, Span>, type_map: Vec<NodeIndex>) -> Self {
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

#[derive(Debug, Clone)]
enum IteNode {
    Invalid,
    Concrete {
        expr: String,
        node: NodeIndex,
    },
    Ite {
        condition: String,
        truthy: Box<IteNode>,
        falsey: Box<IteNode>,
    },
}

impl Display for IteNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            IteNode::Invalid => f.write_str("(X)"),
            IteNode::Concrete { expr, .. } => f.write_str(expr),
            IteNode::Ite {
                truthy,
                falsey,
                condition,
            } => f.write_fmt(format_args!("(ite {condition} {} {})", truthy, falsey)),
        }
    }
}

#[derive(Debug, Clone)]
struct NodeEntry<'a, 'program, 'source> {
    ite: IteNode,
    type_cache: &'a HashMap<NodeIndex, usize>,
    variant_map: &'a HashMap<NodeIndex, Vec<NodeIndex>>,
    graph: &'a DiGraph<FandangoNode<'program, 'source>, Span<'source>>,
}

impl<'a, 'program, 'source> Display for NodeEntry<'a, 'program, 'source> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.ite, f)
    }
}

impl<'a, 'program, 'source> NodeEntry<'a, 'program, 'source> {
    fn new(
        ite: IteNode,
        type_cache: &'a HashMap<NodeIndex, usize>,
        variant_map: &'a HashMap<NodeIndex, Vec<NodeIndex>>,
        graph: &'a DiGraph<FandangoNode<'program, 'source>, Span<'source>>,
    ) -> Self {
        Self {
            ite,
            type_cache,
            variant_map,
            graph,
        }
    }

    fn concrete(
        expr: String,
        node: NodeIndex,
        type_cache: &'a HashMap<NodeIndex, usize>,
        variant_map: &'a HashMap<NodeIndex, Vec<NodeIndex>>,
        graph: &'a DiGraph<FandangoNode<'program, 'source>, Span<'source>>,
    ) -> Self {
        Self::new(
            IteNode::Concrete { expr, node },
            type_cache,
            variant_map,
            graph,
        )
    }

    fn invalid(
        type_cache: &'a HashMap<NodeIndex, usize>,
        variant_map: &'a HashMap<NodeIndex, Vec<NodeIndex>>,
        graph: &'a DiGraph<FandangoNode<'program, 'source>, Span<'source>>,
    ) -> Self {
        Self::new(IteNode::Invalid, type_cache, variant_map, graph)
    }

    fn or(mut self, condition: String, truthy: Self) -> Self {
        self.ite = IteNode::Ite {
            condition,
            truthy: Box::new(truthy.ite),
            falsey: Box::new(self.ite),
        };
        self
    }

    fn path_as_access(self, path: &[NodeIndex]) -> (String, Self) {
        let mut condition = "true".to_string();
        let mut accessor = self.clone();
        for window in path.windows(2) {
            let &[from, to] = window else {
                unreachable!("Invalid window size");
            };
            let idx = self.variant_map[&from]
                .iter()
                .enumerate()
                .find_map(|(i, &e)| (e == to).then_some(i))
                .unwrap();
            if matches!(
                self.graph.node_weight(from).unwrap(),
                FandangoNode::Alternative(_)
            ) {
                condition = format!("(& (= (variant {accessor}) {idx}) {condition})");
            }
            accessor = accessor.get(idx);
        }
        (condition, accessor)
    }

    fn paths_as_ite(mut self, paths: &Vec<Vec<NodeIndex>>) -> Self {
        match self.ite {
            IteNode::Concrete { .. } => {
                let mut accessor = Self::invalid(self.type_cache, self.variant_map, self.graph);
                for path in paths {
                    let (path_condition, path_accessor) = self.clone().path_as_access(path);
                    accessor = accessor.or(path_condition, path_accessor);
                }
                accessor
            }
            IteNode::Ite {
                truthy,
                falsey,
                condition,
            } => {
                let truthy = Self::new(*truthy, self.type_cache, self.variant_map, self.graph)
                    .paths_as_ite(paths)
                    .ite;
                let falsey = Self::new(*falsey, self.type_cache, self.variant_map, self.graph)
                    .paths_as_ite(paths)
                    .ite;
                Self::new(
                    IteNode::Ite {
                        condition,
                        truthy: Box::new(truthy),
                        falsey: Box::new(falsey),
                    },
                    self.type_cache,
                    self.variant_map,
                    self.graph,
                )
            }
            _ => self,
        }
    }

    fn child(self, node: NodeIndex) -> Self {
        match self.ite {
            IteNode::Invalid => self,
            IteNode::Concrete { node: start, .. } => {
                let graph = self.graph;
                // do not traverse nonterminals that are not ourselves!
                let filtered = EdgeFiltered::from_fn(graph, |e| {
                    e.source() == start
                        || !matches!(
                            graph.node_weight(e.source()).unwrap(),
                            FandangoNode::Nonterminal(_)
                        )
                });

                self.paths_as_ite(
                    &algo::all_simple_paths::<Vec<_>, _>(&filtered, start, node, 1, None).collect(),
                )
            }
            IteNode::Ite {
                truthy,
                falsey,
                condition,
            } => {
                let truthy = Self::new(*truthy, self.type_cache, self.variant_map, self.graph)
                    .child(node)
                    .ite;
                let falsey = Self::new(*falsey, self.type_cache, self.variant_map, self.graph)
                    .child(node)
                    .ite;
                Self::new(
                    IteNode::Ite {
                        condition,
                        truthy: Box::new(truthy),
                        falsey: Box::new(falsey),
                    },
                    self.type_cache,
                    self.variant_map,
                    self.graph,
                )
            }
        }
    }

    fn descendent(self, node: NodeIndex) -> Self {
        match self.ite {
            IteNode::Invalid => self,
            IteNode::Concrete { node: start, .. } => {
                let graph = self.graph;
                self.paths_as_ite(
                    &algo::all_simple_paths::<Vec<_>, _>(graph, start, node, 1, None).collect(),
                )
            }
            IteNode::Ite {
                truthy,
                falsey,
                condition,
            } => {
                let truthy = Self::new(*truthy, self.type_cache, self.variant_map, self.graph)
                    .descendent(node)
                    .ite;
                let falsey = Self::new(*falsey, self.type_cache, self.variant_map, self.graph)
                    .descendent(node)
                    .ite;
                Self::new(
                    IteNode::Ite {
                        condition,
                        truthy: Box::new(truthy),
                        falsey: Box::new(falsey),
                    },
                    self.type_cache,
                    self.variant_map,
                    self.graph,
                )
            }
        }
    }

    fn get(self, idx: usize) -> Self {
        match self.ite {
            IteNode::Invalid => self,
            IteNode::Concrete { node: start, expr } => {
                let node_idx = self.variant_map[&start][idx];
                Self {
                    ite: IteNode::Concrete {
                        expr: format!(
                            "(typed {} (access {} {idx}))",
                            self.type_cache[&node_idx], expr
                        ),
                        node: node_idx,
                    },
                    type_cache: self.type_cache,
                    variant_map: self.variant_map,
                    graph: self.graph,
                }
            }
            IteNode::Ite {
                truthy,
                falsey,
                condition,
            } => {
                let truthy = Self::new(*truthy, self.type_cache, self.variant_map, self.graph)
                    .get(idx)
                    .ite;
                let falsey = Self::new(*falsey, self.type_cache, self.variant_map, self.graph)
                    .get(idx)
                    .ite;
                Self::new(
                    IteNode::Ite {
                        condition,
                        truthy: Box::new(truthy),
                        falsey: Box::new(falsey),
                    },
                    self.type_cache,
                    self.variant_map,
                    self.graph,
                )
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct NodeCost {
    is_access: bool,
    untyped_access: HashSet<Id>,
    observed: HashSet<NodeIndex>,
    ast_size: usize,
}

impl NodeCost {
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

impl PartialEq<Self> for NodeCost {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other).unwrap().is_eq()
    }
}

impl PartialOrd for NodeCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for NodeCost {}

impl Ord for NodeCost {
    fn cmp(&self, other: &Self) -> Ordering {
        self.observed
            .len()
            .cmp(&other.observed.len())
            .then_with(|| self.ast_size.cmp(&other.ast_size))
    }
}

impl CostFunction<FandangoConstraintLang> for &'_ NodesPresent {
    type Cost = NodeCost;

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

fn gen_rules<N: Analysis<FandangoConstraintLang>>(
    graph: &DiGraph<FandangoNode, Span>,
    variant_map: &mut HashMap<NodeIndex, Vec<NodeIndex>>,
    type_cache: &HashMap<NodeIndex, usize>,
) -> Result<
    Vec<Rewrite<FandangoConstraintLang, N>>,
    <Pattern<FandangoConstraintLang> as FromStr>::Err,
> {
    let mut rules: Vec<Rewrite<FandangoConstraintLang, _>> = vec![
        rw!("eq"; "(= ?x ?x)" => "true"),
        rw!("not-1"; "(! true)" => "false"),
        rw!("not-2"; "(! false)" => "true"),
        rw!("commute-eq"; "(= ?x ?y)" => "(= ?y ?x)"),
        rw!("unsat-and"; "(& ?x (! ?x))" => "false"),
        rw!("commute-eq-and"; "(& (= ?x ?y) (= ?y ?z))" => "(& (= ?x ?y) (= ?x ?z))"),
        rw!("commute-and"; "(& ?x ?y)" => "(& ?y ?x)"),
        rw!("exchange-and"; "(& ?x (& ?y ?z))" => "(& ?y (& ?x ?z))"),
        rw!("dist-and"; "(& ?x (| ?y ?z))" => "(| (& ?x ?y) (& ?x ?z))"),
        rw!("factor-and"; "(| (& ?x ?y) (& ?x ?z))" => "(& ?x (| ?y ?z))"),
        rw!("factor-simp-and-1"; "(| (& ?x ?y) ?x)" => "?x"),
        rw!("factor-simp-and-2"; "(| (& ?x ?y) (! ?x))" => "(| ?y (! ?x))"),
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
        rw!("ite-lift-1"; "(ite true ?x ?y)" => "?x"),
        rw!("ite-lift-2"; "(ite false ?x ?y)" => "?y"),
        rw!("ite-swap"; "(ite (! ?a) ?x ?y)" => "(ite ?a ?y ?x)"),
        rw!("ite-lift-access"; "(ite ?a (typed ?t (access ?x ?i)) (typed ?t (access ?y ?i)))" => "(typed ?t (access (ite ?a ?x y) ?i))"),
        rw!("ite-simp"; "(ite ?a ?x ?x)" => "?x"),
        rw!("ite-solve"; "(ite true ?x ?y)" => "?x"),
        rw!("simp-str-deriv"; "(= (str (typed ?a ?x)) (str (typed ?a ?y)))" => "(= (typed ?a ?x) (typed ?a ?y))"),
        rw!("simp-typed"; "(typed ?a (typed ?a ?x))" => "(typed ?a ?x)"),
    ];

    for (node, children) in variant_map {
        let &node_type = type_cache.get(node).unwrap();

        let mut base;
        children.sort(); // into visible order
        match graph.node_weight(*node).unwrap() {
            FandangoNode::Alternative(_) => {
                // "the variants and values of those variants are equal"
                base = "false".to_string();
                for (idx, child) in children.iter().enumerate() {
                    let &child_type = type_cache.get(child).unwrap();
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
                    let &child_type = type_cache.get(child).unwrap();
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

        rules.push(
            Rewrite::new(
                format!("node-{node_type}-eq"),
                format!("(= (typed {node_type} ?x) (typed {node_type} ?y))")
                    .parse::<Pattern<_>>()?,
                base.parse::<Pattern<_>>()?,
            )
            .unwrap(),
        );
    }

    Ok(rules)
}

#[derive(Debug, Default, Clone)]
struct ConcreteInputAnalysis {
    symbols: HashMap<Symbol, usize>,
    indexer: Vec<HashMap<usize, usize>>,
}

impl ConcreteInputAnalysis {
    fn new<'a, N>(node: &'a mut N, idx: usize) -> Self
    where
        N: Node,
        <N as Node>::TypeMut<'a>: From<&'a mut N> + VisitableChildren<<N as Node>::TypeMut<'a>>,
    {
        let mut collected = vec![HashMap::new()];
        let mut choices = HashMap::new();
        let builder = ConcreteInputAnalysisBuilder {
            collected: &mut collected,
            choices: &mut choices,
        };
        let _ = builder.visit(node, idx);

        ConcreteInputAnalysis {
            symbols: HashMap::new(),
            indexer: collected,
        }
    }

    fn define_symbol(mut self, symbol: Symbol, node_idx: usize) -> Self {
        self.symbols.insert(symbol, node_idx);
        self
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum ConcreteInputAnalysisData {
    Concrete(usize),
    Variant(usize),
    Symbol(Symbol),
    Unknown,
}

impl Analysis<FandangoConstraintLang> for ConcreteInputAnalysis {
    type Data = ConcreteInputAnalysisData;

    fn make(
        egraph: &EGraph<FandangoConstraintLang, Self>,
        enode: &FandangoConstraintLang,
    ) -> Self::Data {
        match enode {
            FandangoConstraintLang::Typed([_, id]) => egraph[*id].data,
            FandangoConstraintLang::Variant(id) => {
                if let ConcreteInputAnalysisData::Concrete(concrete) = egraph[*id].data {
                    ConcreteInputAnalysisData::Variant(
                        *egraph.analysis.indexer[concrete].keys().next().unwrap(),
                    )
                } else {
                    ConcreteInputAnalysisData::Unknown
                }
            }
            FandangoConstraintLang::Access([id, idx]) => {
                if let ConcreteInputAnalysisData::Concrete(concrete) = egraph[*id].data {
                    if let Some(idx) = egraph[*idx].nodes.iter().find_map(|n| {
                        if let FandangoConstraintLang::Concrete(idx) = n {
                            Some(idx)
                        } else {
                            None
                        }
                    }) {
                        if let Some(id) = egraph.analysis.indexer[concrete].get(idx).copied() {
                            return ConcreteInputAnalysisData::Concrete(id);
                        }
                    }
                }
                ConcreteInputAnalysisData::Unknown
            }
            FandangoConstraintLang::Concrete(value) => ConcreteInputAnalysisData::Concrete(*value),
            FandangoConstraintLang::Var(name) => ConcreteInputAnalysisData::Symbol(*name),
            _ => ConcreteInputAnalysisData::Unknown,
        }
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        match (a, b) {
            (
                ConcreteInputAnalysisData::Concrete(first),
                ConcreteInputAnalysisData::Concrete(second),
            ) => {
                assert_eq!(*first, second);
                DidMerge(false, false)
            }
            (
                ConcreteInputAnalysisData::Variant(first),
                ConcreteInputAnalysisData::Variant(second),
            ) => {
                assert_eq!(*first, second);
                DidMerge(false, false)
            }
            (
                ConcreteInputAnalysisData::Symbol(first),
                ConcreteInputAnalysisData::Symbol(second),
            ) => {
                let mut old = second;
                mem::swap(&mut old, first);
                DidMerge(old == *first, false)
            }
            (ConcreteInputAnalysisData::Unknown, ConcreteInputAnalysisData::Unknown) => {
                DidMerge(false, false)
            }
            (_, ConcreteInputAnalysisData::Unknown) => DidMerge(false, true),
            (first @ ConcreteInputAnalysisData::Unknown, second) => {
                *first = second;
                DidMerge(true, false)
            }
            (first, second) => panic!("Illegal merge between {first:?} and {second:?}."),
        }
    }

    fn modify(egraph: &mut EGraph<FandangoConstraintLang, Self>, id: Id) {
        let class = &egraph[id];
        if class
            .iter()
            .any(|n| matches!(n, FandangoConstraintLang::Typed(_)))
        {
            return; // we don't want to replace typed nodes because this would clear our cost info
        }
        match class.data {
            ConcreteInputAnalysisData::Concrete(value)
            | ConcreteInputAnalysisData::Variant(value) => {
                let other = egraph.add(FandangoConstraintLang::Concrete(value));
                egraph.union(id, other);
            }
            ConcreteInputAnalysisData::Symbol(sym) => {
                if let Some(&value) = egraph.analysis.symbols.get(&sym) {
                    let other = egraph.add(FandangoConstraintLang::Concrete(value));
                    egraph.union(id, other);
                }
            }
            ConcreteInputAnalysisData::Unknown => {
                // nothing to do; we don't know what this node is
            }
        }
    }
}

struct ConcreteInputAnalysisBuilder<'a, 'b> {
    collected: &'a mut Vec<HashMap<usize, usize>>,
    choices: &'b mut HashMap<usize, usize>,
}

impl ConcreteInputAnalysisBuilder<'_, '_> {
    fn descendent<'a, 'b>(
        &'a mut self,
        choices: &'b mut HashMap<usize, usize>,
    ) -> ConcreteInputAnalysisBuilder<'a, 'b> {
        ConcreteInputAnalysisBuilder {
            collected: self.collected,
            choices,
        }
    }
}

impl<T> Visitor<T> for ConcreteInputAnalysisBuilder<'_, '_>
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program mut N, idx: usize) -> VisitResult<Self, T>
    where
        N: Node<TypeMut<'program> = T>,
        T: From<&'program mut N>,
    {
        let node_idx = self.collected.len();
        self.collected.push(HashMap::new());
        self.choices.insert(idx, node_idx); // we don't have all indices in alternations
        let mut choices = HashMap::new();
        T::from(node).visit_each(self.descendent(&mut choices))?;
        self.collected[node_idx] = choices;
        Ok(ControlFlow::Continue(self))
    }
}

#[test]
fn egg_demo() -> Result<(), Box<dyn Error>> {
    let (lookup, graph) = nonterminal_start::root().into_graph();
    let mut variant_map = graph.node_indices().map(|n1| (n1, graph.edges(n1))).fold(
        HashMap::<_, Vec<_>>::new(),
        |mut collected, (n1, edges)| {
            let old = collected.insert(n1, {
                let mut edges = edges.collect::<Vec<_>>();
                edges.sort_by_key(|e| e.weight().start());
                edges.into_iter().map(|e| e.target()).collect()
            });
            assert!(old.is_none());
            collected
        },
    );

    let mut type_map = Vec::new();
    let mut type_cache = HashMap::new();

    for node in graph.node_indices() {
        type_cache.insert(node, type_map.len());
        type_map.push(node);
    }

    let uninit_analysis = ConcreteInputAnalysis::default();
    let rules = gen_rules(&graph, &mut variant_map, &type_cache)?;
    let costs = NodesPresent::new(&graph, type_map);

    // let xml_attr_node = lookup[&FandangoNode::Nonterminal(&Nonterminal::new("xml_attribute"))];
    // let first_typed = NodeEntry::concrete(
    //     format!("(typed {} first)", type_cache[&xml_attr_node]),
    //     xml_attr_node,
    //     &type_cache,
    //     &variant_map,
    //     &graph,
    // );
    // let second_typed = NodeEntry::concrete(
    //     format!("(typed {} second)", type_cache[&xml_attr_node]),
    //     xml_attr_node,
    //     &type_cache,
    //     &variant_map,
    //     &graph,
    // );
    //
    // let attr_quant = format!(
    //     r#"(|
    //         (=
    //             (str {first_typed})
    //             (str {second_typed})
    //         )
    //         (! (=
    //             (str {})
    //             (str {})
    //         ))
    //     )"#,
    //     first_typed
    //         .clone()
    //         .child(lookup[&FandangoNode::Nonterminal(&Nonterminal::new("id"))]),
    //     second_typed
    //         .clone()
    //         .child(lookup[&FandangoNode::Nonterminal(&Nonterminal::new("id"))]),
    // );
    //
    // let start = attr_quant.parse()?;
    // let orig_cost = (&costs).cost_rec(&start);
    // let runner = Runner::<_, _>::new(uninit_analysis.clone())
    //     .with_expr(&start)
    //     .run(&rules);
    // let extractor = Extractor::new(&runner.egraph, &costs);
    // let (cost, best_attr_expr) = extractor.find_best(runner.roots[0]);
    //
    // println!(
    //     "{} controlled nodes => {} controlled nodes",
    //     orig_cost.observed.len(),
    //     cost.observed.len()
    // );
    // println!("{} => {}", start.pretty(2), best_attr_expr.pretty(2));

    let xml_tree_node = lookup[&FandangoNode::Nonterminal(&Nonterminal::new("xml_tree"))];
    let tree_typed = NodeEntry::concrete(
        format!("(typed {} tree)", type_cache[&xml_tree_node]),
        xml_tree_node,
        &type_cache,
        &variant_map,
        &graph,
    );
    let tree_quant = format!(
        r#"(=
            {}
            {}
        )"#,
        tree_typed
            .clone()
            .child(lookup[&FandangoNode::Nonterminal(&Nonterminal::new("xml_open_tag"))])
            .child(lookup[&FandangoNode::Nonterminal(&Nonterminal::new("id"))]),
        tree_typed
            .clone()
            .child(lookup[&FandangoNode::Nonterminal(&Nonterminal::new("xml_close_tag"))])
            .child(lookup[&FandangoNode::Nonterminal(&Nonterminal::new("id"))]),
    );

    let start = tree_quant.parse()?;
    let orig_cost = (&costs).cost_rec(&start);
    let mut runner = Runner::<_, _>::new(uninit_analysis)
        .with_expr(&start)
        .run(&rules);
    let extractor = Extractor::new(&runner.egraph, &costs);
    let (cost, best_tree_expr) = extractor.find_best(runner.roots[0]);

    println!(
        "{} controlled nodes => {} controlled nodes",
        orig_cost.observed.len(),
        cost.observed.len()
    );
    println!("{} => {}", start.pretty(2), best_tree_expr.pretty(2));

    // let mut rng = StdRng::seed_from_u64(0);
    // let mut concrete = nonterminal_start::generate(&mut rng, &mut ());
    // let init_analysis = ConcreteInputAnalysis::new(&mut concrete, 0);
    //
    // let mut runner = Runner::<_, _>::new(init_analysis)
    //     .with_expr(&best_tree_expr)
    //     .run(&rules);
    //
    // let extractor = Extractor::new(&runner.egraph, &costs);
    // let (_, best_tree_expr) = extractor.find_best(runner.roots[0]);
    //
    // println!(
    //     "{} controlled nodes => {} controlled nodes",
    //     orig_cost.observed.len(),
    //     cost.observed.len()
    // );
    // println!("{} => {}", start.pretty(2), best_tree_expr.pretty(2));

    Ok(())
}
