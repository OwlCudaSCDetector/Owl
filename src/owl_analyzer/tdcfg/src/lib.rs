use std::{collections::BTreeMap, ops::AddAssign};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, serde::Serialize, serde::Deserialize,
)]
pub struct Direct<T> {
    pub start: T,
    pub end: T,
}

pub struct Node<T, N> {
    pub id: T,
    // pub mem_access:
    pub inner: N,
}

impl<T, N: AddAssign> Node<T, N> {
    fn merge(&mut self, other: Self) {
        self.inner += other.inner;
    }
}

pub struct Edge<T, E> {
    pub direct: Direct<T>,
    // data:
    pub inner: E,
}

impl<T, E> Edge<T, E> {
    pub fn new(direct: impl Into<Direct<T>>, inner: impl Into<E>) -> Self {
        Self {
            direct: direct.into(),
            inner: inner.into(),
        }
    }
}

impl<T, E: AddAssign> Edge<T, E> {
    pub fn merge(&mut self, other: Self) {
        self.inner += other.inner;
    }
}

pub struct TDcfg<T, N, E> {
    pub nodes: BTreeMap<T, Node<T, N>>,
    // pub edges: BTreeMap<Direct<T>, Edge<T, E>>,
}

impl<T, N, E> Default for TDcfg<T, N, E> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            // edges: Default::default(),
        }
    }
}

impl<T, N, E> TDcfg<T, N, E> {
    pub fn new(
        nodes: impl Into<BTreeMap<T, Node<T, N>>>,
        edges: impl Into<BTreeMap<Direct<T>, Edge<T, E>>>,
    ) -> Self {
        Self {
            nodes: nodes.into(),
            // edges: edges.into(),
        }
    }
}

impl<T: Ord, N: AddAssign, E: AddAssign> TDcfg<T, N, E> {
    pub fn insert_or_add_node(&mut self, node: Node<T, N>) {}

    pub fn merge(&mut self, other: Self) {
        other.nodes.into_iter().for_each(|(id, n)| {
            if self.nodes.contains_key(&id) {
                self.nodes.get_mut(&id).unwrap().merge(n);
            } else {
                self.nodes.insert(id, n);
            }
        });

        other.edges.into_iter().for_each(|(d, e)| {
            if self.edges.contains_key(&d) {
                self.edges.get_mut(&d).unwrap().merge(e);
            } else {
                self.edges.insert(d, e);
            }
        });
    }
}
