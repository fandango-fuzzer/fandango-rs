//! Utilities associated with computing the k-path coverage of a grammar.
//!
//! See: <https://publications.cispa.saarland/3572/1/tosem-codeine-arxiv.pdf>

use crate::lang::{FandangoNode, Operator, Program, Symbol};
use crate::typing::{AsNodeRef, DiscriminantLookup, Node, Opaque};
use crate::visitor::{VisitResult, VisitableChildren, Visitor};
use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::num::NonZeroUsize;
use core::ops::ControlFlow;
use hashbrown::hash_set::Entry;
use hashbrown::{HashMap, HashSet};

use mappable_rc::Mrc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Represents the current state of a k-paths computation.
///
/// Use [`KPathUpdate`] to update the content of this computation.
#[derive(Clone, Debug)]
pub struct KPaths {
    k: NonZeroUsize,
    lookup: HashMap<Mrc<[usize]>, usize>,
}

fn edges_to_serializable(edges: impl Iterator<Item = [usize; 2]>) -> HashMap<usize, Vec<usize>> {
    edges.fold(HashMap::new(), |mut map, window| {
        map.entry(window[0]).or_default().push(window[1]);
        map
    })
}

impl Serialize for KPaths {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let edges = edges_to_serializable(
            self.lookup()
                .keys()
                .flat_map(|k| k.windows(2))
                .map(|w| w.try_into().unwrap()),
        );
        let k = self.k;
        (k, edges).serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for KPaths {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let (k, edges) = <(NonZeroUsize, HashMap<usize, Vec<usize>>)>::deserialize(deserializer)?;
        Ok(Self::from_edges(k, edges))
    }
}

impl KPaths {
    fn from_edges(k: NonZeroUsize, edges: HashMap<usize, Vec<usize>>) -> Self {
        let mut paths_len_n = vec![Vec::from_iter(
            edges
                .keys()
                .chain(edges.values().flatten())
                .copied()
                .collect::<HashSet<_>>()
                .into_iter()
                .map(|n| vec![n]),
        )];
        for _ in 1..k.get() {
            let mut paths_len_i = Vec::new();
            let last = paths_len_n.last_mut().unwrap();
            for path in core::mem::take(last) {
                if let Some(edges) = edges.get(path.last().unwrap()) {
                    // this sequence has more to give
                    for child in edges {
                        let mut with_child = path.clone();
                        with_child.push(*child);
                        paths_len_i.push(with_child);
                    }
                } else {
                    last.push(path); // okay -- this subsequence is too short for length k
                }
            }
            paths_len_n.push(paths_len_i);
        }

        let mut collected: HashSet<Mrc<[usize]>> = HashSet::new();
        let mut queue = VecDeque::new();

        // transform all paths, in reverse order of size
        for paths in paths_len_n.into_iter().rev() {
            for path in paths {
                queue.push_back(path.into());
            }
        }

        // add all nodes recursively, only computing next children
        while let Some(path) = queue.pop_front() {
            // intern!
            if let Entry::Vacant(vacancy) = collected.entry(path) {
                let path = vacancy.get().clone();
                vacancy.insert();

                if path.len() > 1 {
                    // use the subsequences of this interned form to deduplicate allocations
                    queue.push_back(Mrc::map(path.clone(), |s| &s[..(s.len() - 1)]));
                    queue.push_back(Mrc::map(path, |s| &s[1..]));
                }
            }
        }

        Self {
            k,
            lookup: collected.iter().map(|path| (path.clone(), 0)).collect(),
        }
    }

    fn collect_edges<T>(
        definition: FandangoNode<'static, 'static>,
        collected: &mut HashMap<usize, Vec<usize>>,
    ) -> FandangoNode<'static, 'static>
    where
        T: DiscriminantLookup,
    {
        let children;
        match definition {
            nt @ FandangoNode::Nonterminal(_) => return nt, // nothing to do
            FandangoNode::Alternative(alt) => {
                if alt.concatenations().len() == 1 {
                    return Self::collect_edges::<T>(
                        FandangoNode::Concatenation(alt.concatenations()[0].inner()),
                        collected,
                    );
                }
                children = alt
                    .concatenations()
                    .iter()
                    .map(|c| FandangoNode::Concatenation(c.inner()))
                    .collect();
            }
            FandangoNode::Concatenation(concat) => {
                if concat.operators().len() == 1 {
                    return Self::collect_edges::<T>(
                        FandangoNode::Operator(concat.operators()[0].inner()),
                        collected,
                    );
                }
                children = concat
                    .operators()
                    .iter()
                    .map(|c| FandangoNode::Operator(c.inner()))
                    .collect();
            }
            FandangoNode::Operator(op) => {
                // TODO: is there another way we should be computing k-path here?
                let child = match op {
                    Operator::Kleene(kl) => kl,
                    Operator::Plus(pl) => pl,
                    Operator::Option(opt) => opt,
                    Operator::Repeat(rpt, _, _) => rpt,
                    Operator::Symbol(sym) => {
                        return Self::collect_edges::<T>(
                            FandangoNode::Symbol(sym.inner()),
                            collected,
                        );
                    }
                };
                let child = FandangoNode::Symbol(child.inner());
                children = vec![child];
            }
            FandangoNode::Symbol(sym) => {
                let inner = match sym {
                    Symbol::Nonterminal(nt) => FandangoNode::from(nt),
                    Symbol::Alternative(alt) => FandangoNode::from(alt),
                    Symbol::String(s) => FandangoNode::from(s),
                };
                return Self::collect_edges::<T>(inner, collected);
            }
            s @ FandangoNode::String(_) => return s, // nothing to do
            _ => unreachable!("Cannot generate this case."),
        }

        let discriminant = T::lookup_discriminant(&definition);
        let children = children
            .into_iter()
            .map(|child| Self::collect_edges::<T>(child, collected))
            .map(|n| T::lookup_discriminant(&n))
            .collect();

        assert!(collected.insert(discriminant, children).is_none());

        definition
    }

    /// Create a new k-paths state for the given program.
    ///
    /// You need to specify `T` here. If you're using a dynamic implementation, use
    /// `::<DynamicNode>`, otherwise specify the `Type` or `TypeMut` of your static grammar.
    #[must_use] 
    pub fn new<T>(k: NonZeroUsize, program: &'static Program) -> KPaths
    where
        T: DiscriminantLookup,
    {
        let nonterminals = program.nonterminals();
        let mut edges = HashMap::new();
        for (nonterminal, definition) in &nonterminals {
            let definition = Self::collect_edges::<T>(*definition, &mut edges);
            let nonterminal = T::lookup_discriminant(nonterminal);
            let definition = T::lookup_discriminant(&definition);
            assert!(edges.insert(nonterminal, vec![definition],).is_none());
        }

        Self::from_edges(k, edges)
    }

    /// Get the current k-paths totals expressed as `(#uncovered, #total)`.
    #[must_use] 
    pub fn k_paths(&self) -> (usize, usize) {
        (
            self.lookup.values().filter(|v| **v == 0).count(),
            self.lookup.len(),
        )
    }

    /// The `k` of k-path.
    #[must_use] 
    pub fn k(&self) -> NonZeroUsize {
        self.k
    }

    /// Get the lookup table, which maps a particular path to a number of observations of that path.
    #[must_use] 
    pub fn lookup(&self) -> &HashMap<Mrc<[usize]>, usize> {
        &self.lookup
    }

    /// Clear the state of the kpaths table.
    pub fn clear(&mut self) {
        for v in self.lookup.values_mut() {
            *v = 0;
        }
    }
}

/// Visitor used to update the [`KPaths`] values.
pub struct KPathUpdate<'a, const INSERT: bool> {
    kpaths: &'a mut KPaths,
    stack: Vec<usize>,
}

impl<'a, const INSERT: bool> KPathUpdate<'a, INSERT> {
    fn new(kpaths: &'a mut KPaths) -> Self {
        Self {
            kpaths,
            stack: Vec::new(),
        }
    }

    /// The [`KPaths`] contained by this visitor. Useful for multiple usages.
    #[must_use] 
    pub fn kpaths(&self) -> &KPaths {
        &*self.kpaths
    }
}

impl<'a> KPathUpdate<'a, true> {
    /// Update the provided [`KPaths`] by inserting visited paths.
    pub fn inserting(kpaths: &'a mut KPaths) -> Self {
        Self::new(kpaths)
    }
}

impl<'a> KPathUpdate<'a, false> {
    /// Update the provided [`KPaths`] by removing visited paths.
    pub fn removing(kpaths: &'a mut KPaths) -> Self {
        Self::new(kpaths)
    }
}

impl<const INSERT: bool, T> Visitor<T> for KPathUpdate<'_, INSERT>
where
    T: VisitableChildren<T>,
{
    type Continue = Self;
    type Break = Infallible;
    type Error = Infallible;

    fn visit<'program, N>(mut self, node: &'program N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        self.stack.push(node.discriminant());
        for offset in self.stack.len().saturating_sub(self.kpaths.k.get())..self.stack.len() {
            let slice = &self.stack[offset..];
            let (_rcd, count) = self.kpaths.lookup.get_key_value_mut(slice).unwrap();
            if INSERT {
                *count += 1;
            } else {
                *count = count.checked_sub(1).unwrap(); // sanity
            }
        }
        let mut visitor = node
            .opaque()
            .visit_each(self)
            .unwrap()
            .continue_value()
            .unwrap();
        visitor.stack.pop();
        Ok(ControlFlow::Continue(visitor))
    }
}

/// Visitor pattern for all paths within a provided input.
///
/// Use with [`KPathVisit`].
pub trait KPathVisitor {
    /// The value into which this visitor is transformed after a total traversal of an input.
    type Value;
    /// The value which is returned when the visitor has exited traversal early.
    type Break;
    /// The error type returned in the case of an error during traversal.
    type Error;

    /// When traversing an input, this callback is invoked with the current number of times that the
    /// given path has been visited. There are no guarantees on the call order of this visitor.
    fn visit_path(
        &mut self,
        count: usize,
        path: &Mrc<[usize]>,
    ) -> Result<ControlFlow<Self::Break>, Self::Error>;

    /// Convert this visitor into an accrued value.
    fn value(self) -> Self::Value;
}

/// Visitor for inputs which allows for visiting k-paths with [`KPathVisitor`] implementations.
pub struct KPathVisit<'a, V> {
    kpaths: &'a KPaths,
    stack: Vec<usize>,
    and_then: V,
}

impl<'a, V> KPathVisit<'a, V>
where
    V: KPathVisitor,
{
    /// Create a new [`KPathVisit`], calling the provided [`KPathVisitor`] implementation along the
    /// way.
    pub fn new(kpaths: &'a KPaths, and_then: V) -> Self {
        Self {
            kpaths,
            stack: Vec::new(),
            and_then,
        }
    }

    /// Extract the value of the contained [`KPathVisitor`] with [`KPathVisitor::value`].
    pub fn value(self) -> V::Value {
        self.and_then.value()
    }
}

impl<T, V> Visitor<T> for KPathVisit<'_, V>
where
    T: VisitableChildren<T>,
    V: KPathVisitor,
{
    type Continue = Self;
    type Break = V::Break;
    type Error = V::Error;

    fn visit<'program, N>(mut self, node: &'program N, _idx: usize) -> VisitResult<Self, T>
    where
        N: Node<Type<'program> = T>,
        T: From<&'program N> + AsNodeRef<N>,
    {
        self.stack.push(node.discriminant());
        for offset in self.stack.len().saturating_sub(self.kpaths.k.get())..self.stack.len() {
            let slice = &self.stack[offset..];
            let (rcd, count) = self.kpaths.lookup.get_key_value(slice).unwrap();
            if let ControlFlow::Break(b) = self.and_then.visit_path(*count, rcd)? {
                return Ok(ControlFlow::Break(b));
            }
        }
        let mut result = node.opaque().visit_each(self);
        if let Ok(ControlFlow::Continue(visitor)) = &mut result {
            visitor.stack.pop();
        }
        result
    }
}
