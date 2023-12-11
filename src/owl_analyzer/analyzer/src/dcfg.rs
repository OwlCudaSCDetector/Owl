use std::collections::BTreeMap;

use monitor::{
    cuda::{BBId, InstrId},
    raw::RawNode,
};

use crate::{
    dtest::{EqKernelResult, NodeCfResult, NodeDfResult},
    hist::ks_test_p_value,
    matrix::CfMatrix,
    memory::MemAccessRecord,
};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, serde::Serialize, serde::Deserialize,
)]
pub struct Direct {
    pub start: u32,
    pub end: u32,
}

pub struct Node {
    pub id: BBId,
    pub cf: CfMatrix,
    pub mem_access: MemAccessRecord,
}

impl Node {
    pub fn cf_test(&self, other: &Self, m: usize, n: usize) -> f64 {
        self.cf.test(&other.cf, m, n)
    }

    /// Implement data flow test and return p value of every memory access instruction
    ///
    /// Return None if there is no memory access instruction
    pub fn df_test(&self, other: &Self, m: usize, n: usize) -> Option<Vec<(f64, InstrId)>> {
        if self.mem_access.instrs.is_empty() && other.mem_access.instrs.is_empty() {
            return None;
        }

        log::debug!("Test mem access in node: {}", self.id);

        let p = self
            .mem_access
            .instrs
            .iter()
            .zip(other.mem_access.instrs.iter())
            .map(|(l, r)| (l.test(r, m, n), l.instr))
            .collect();
        Some(p)
    }

    pub fn same(&self, other: &Self) -> bool {
        self.id == other.id && self.cf == other.cf && self.mem_access == other.mem_access
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            id: Default::default(),
            cf: CfMatrix::default(),
            mem_access: MemAccessRecord::new(),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<RawNode> for Node {
    fn from(value: RawNode) -> Self {
        Self {
            id: value.id,
            cf: CfMatrix::from_raw(value.control_flow),
            mem_access: value.mem_access.into(),
        }
    }
}

impl Node {
    fn merge(&mut self, other: Self) {
        self.mem_access += other.mem_access;
        self.cf += other.cf;
    }
}

pub struct TDcfg {
    pub nodes: BTreeMap<BBId, Node>,
}

impl Default for TDcfg {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            // edges: Default::default(),
        }
    }
}

impl TDcfg {
    pub fn test(self, other: Self, n: usize, m: usize, threshold: f64) -> EqKernelResult {
        use crate::align::VecAlign;

        let mut res = EqKernelResult::new();

        // test cf matrix in every node
        let ln: Vec<_> = self.nodes.values().collect();
        let rn: Vec<_> = other.nodes.values().collect();

        let (l, r) = ln.align(&rn);
        assert_eq!(l.len(), r.len());

        l.into_iter().zip(r.into_iter()).for_each(|(l, r)| {
            match (l, r) {
                (Some(l), Some(r)) => {
                    // control flow test
                    log::debug!("Eq node");
                    let p = l.cf_test(r, n, m);
                    if p < threshold {
                        res.push_cf(
                            NodeCfResult::new(l.id).p_value(p), // .l_flow(l.cf.clone())
                                                                // .r_flow(r.cf.clone()),
                        );
                    }

                    // data flow test
                    if let Some(dfp) = l.df_test(r, n, m) {
                        dfp.into_iter()
                            .filter(|(p, _)| *p < threshold)
                            .for_each(|(p, instr)| {
                                res.push_df(
                                    NodeDfResult::new(l.id, instr).p_value(p), // .fix_mem(l.mem_access.clone())
                                                                               // .rnd_mem(r.mem_access.clone()),
                                )
                            })
                    };
                }
                (None, Some(r)) => {
                    log::debug!("Uneq node, left is None");
                    let p = ks_test_p_value(1.0, n, m);
                    res.push_cf(NodeCfResult::new(r.id).p_value(p));
                }
                (Some(l), None) => {
                    log::debug!("Uneq node, right is None");
                    let p = ks_test_p_value(1.0, n, m);
                    res.push_cf(NodeCfResult::new(l.id).p_value(p));
                }
                (None, None) => {
                    panic!("Never happened")
                }
            }
        });

        res
    }

    pub fn same(&self, other: &Self) -> bool {
        if self.nodes.len() != other.nodes.len() {
            return false;
        }

        self.nodes
            .iter()
            .zip(other.nodes.iter())
            .find(|(l, r)| {
                if l.0 != r.0 {
                    return true;
                }

                if l.1 != r.1 {
                    return true;
                }
                return false;
            })
            .is_none()
    }
}

impl TDcfg {
    pub fn new(
        nodes: impl Into<BTreeMap<BBId, Node>>,
        // edges: impl Into<BTreeMap<Direct<T>, Edge<T, E>>>,
    ) -> Self {
        Self {
            nodes: nodes.into(),
            // edges: edges.into(),
        }
    }
}

impl TDcfg {
    pub fn merge(&mut self, other: Self) {
        other.nodes.into_iter().for_each(|(id, n)| {
            if self.nodes.contains_key(&id) {
                self.nodes.get_mut(&id).unwrap().merge(n);
            } else {
                self.nodes.insert(id, n);
            }
        });
    }
}
