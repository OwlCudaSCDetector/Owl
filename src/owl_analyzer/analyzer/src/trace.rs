use crate::{
    alloc::MemPool,
    dcfg::{Node, TDcfg},
    dtest::EqKernelResult,
    kernel::KernelCall,
    matrix::CfMatrix,
    memory::MemAccessRecord,
    merge::Merge,
};
use monitor::{
    cuda::{KernelId, KernelTy},
    raw::{RawCsFrame, RawDCFG, RawKernelTrace, RawTrace},
};
use std::{collections::BTreeMap, fmt::Debug, rc::Rc};

#[derive(Debug)]
pub struct Trace {
    pub kernels: Vec<KernelCall>,
}

impl Trace {
    pub fn same(&self, other: &Self) -> bool {
        if self.kernels.len() != other.kernels.len() {
            return false;
        }

        self.kernels
            .iter()
            .zip(other.kernels.iter())
            .find(|(l, r)| !l.same(&r))
            .is_none()
    }
}

#[derive(Hash, Eq, Debug, Clone)]
pub struct TraceCtx {
    // name: String,
    cs: Rc<Vec<CallFrame>>,
}

impl PartialEq for TraceCtx {
    fn eq(&self, other: &Self) -> bool {
        if self.cs.len() != other.cs.len() {
            return false;
        };

        if self.cs.iter().zip(other.cs.iter()).any(|(l, r)| {
            if l != r {
                log::debug!("Uneq Frame l: {l:?}, r: {r:?}");
                true
            } else {
                false
            }
        }) {
            return false;
        } else {
            return true;
        }
    }
}

impl serde::Serialize for TraceCtx {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let str: String = self
            .cs
            .iter()
            .map(|f| format!("0x{:x}:{}", f.addr, f.name))
            .collect::<Vec<_>>()
            .join("/");

        serializer.serialize_str(&str)
    }
}

impl TraceCtx {
    pub fn new() -> Self {
        Self {
            // name: String::new(),
            cs: Rc::new(Vec::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cs.is_empty()
    }

    pub fn from_raw(value: impl Iterator<Item = impl Into<CallFrame>>) -> Self {
        Self {
            // name: String::new(),
            cs: Rc::new(value.map(|v| v.into()).collect()),
        }
    }
}

#[derive(Hash, Debug, Clone, serde::Serialize)]
pub struct CallFrame {
    pub addr: usize,
    name: String,
    file: Option<String>,
    offset: usize,
}

impl PartialEq for CallFrame {
    fn eq(&self, other: &Self) -> bool {
        if self.addr == other.addr && self.name == other.name {
            true
        } else if self.file == other.file && self.offset == other.offset {
            true
        } else {
            false
        }
    }
}

impl Eq for CallFrame {}

impl From<RawCsFrame> for CallFrame {
    fn from(value: RawCsFrame) -> Self {
        Self {
            addr: value.addr,
            name: value.func,
            file: if value.file.is_empty() {
                None
            } else {
                Some(value.file)
            },
            offset: value.offset,
        }
    }
}

impl From<RawTrace> for Trace {
    fn from(value: RawTrace) -> Self {
        let calls = value
            .kernels
            .into_iter()
            // build mem_pools for kernel trace
            .map(|v| {
                let pools: Vec<_> =
                    v.mp.iter()
                        .map(|d| {
                            let p = MemPool {
                                start: d.addr,
                                size: d.size,
                            };
                            p
                        })
                        .collect();
                (v, pools)
            })
            // build kernel trace
            .map(|(mut v, pools)| {
                let ctx = std::mem::take(&mut v.bt);
                let kernel = KernelTrace::from(v);
                (kernel, ctx, pools)
            })
            // update memory access target address by recorded memory pool
            .map(|(mut kernel, ctx, mem_pools)| {
                kernel.g.nodes.iter_mut().for_each(|(_, node)| {
                    // create a new empty memory record
                    let mut mem_access = MemAccessRecord::new();

                    // for every memory access in the old record
                    for e_instr in node.mem_access.instrs.iter() {
                        for (pos, mem) in e_instr.data.iter().enumerate() {
                            for (addr, num) in mem.iter() {
                                // convert to offset
                                if let Some(addr) = mem_pools.iter().find_map(|p| p.convert(*addr))
                                {
                                    // in-pool memory access
                                    mem_access.add_instr_mem_access(e_instr.instr, pos, addr, *num);
                                } else {
                                    // out-pool memory access
                                    mem_access.add_instr_mem_access(
                                        e_instr.instr,
                                        pos,
                                        *addr,
                                        *num,
                                    );
                                };
                            }
                        }
                    }

                    // update node's mem_access
                    node.mem_access = mem_access;
                });

                KernelCall::new(TraceCtx::from_raw(ctx.into_iter()), kernel)
            })
            .collect();

        Self { kernels: calls }
        // todo!()
    }
}

pub struct KernelTrace {
    pub id: KernelId,
    pub ty: KernelTy,
    pub addr: usize,
    pub g: TDcfg,
}

impl KernelTrace {
    pub fn test(self, other: Self, n: usize, m: usize, threshold: f64) -> EqKernelResult {
        assert_eq!(self.addr, other.addr);
        assert_eq!(self.ty, other.ty);

        log::debug!("Testing DCFG");
        self.g.test(other.g, n, m, threshold).ty(self.ty)
    }

    pub fn same(&self, other: &Self) -> bool {
        self.id == other.id
            && self.ty == other.ty
            && self.addr == other.addr
            && self.g.same(&other.g)
    }
}

impl Debug for KernelTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KernelTrace")
            .field("id", &self.id)
            .field("ty", &self.ty)
            .field("addr", &self.addr)
            // .field("g", &self.g)
            .finish()
    }
}

impl Default for KernelTrace {
    fn default() -> Self {
        Self {
            id: Default::default(),
            ty: Default::default(),
            addr: 0,
            // cs: Default::default(),
            g: TDcfg::default(),
            // name: String::new(),
        }
    }
}

impl Merge for KernelTrace {
    fn merge(&mut self, _: &Self) {
        todo!()
    }

    fn merge_own(&mut self, other: Self) {
        assert_eq!(self.ty, other.ty);
        self.g.merge(other.g);
    }
}

impl From<RawKernelTrace> for KernelTrace {
    fn from(value: RawKernelTrace) -> Self {
        Self {
            id: value.id,
            ty: value.ty,
            addr: 0,
            g: convert_raw_dcfg(value.g),
            // name: value.name,
        }
    }
}

/// Convert RawDCFG to our DCFG
fn convert_raw_dcfg(g: RawDCFG) -> TDcfg {
    TDcfg::new(
        g.nodes
            .into_iter()
            .map(|n| {
                (
                    n.id,
                    Node {
                        id: n.id,
                        cf: CfMatrix::from_raw(n.control_flow),
                        mem_access: MemAccessRecord::from(n.mem_access),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use monitor::raw::RawData;
    use serde_json;

    use super::convert_raw_dcfg;

    #[test]
    pub fn test_convert_raw_dcfg() {
        let reader = BufReader::new(std::fs::File::open("../examples/kernel.json").unwrap());
        let data: RawData = serde_json::from_reader(reader).unwrap();

        let mut ks = data.get_kernel();
        let g = ks.pop().unwrap().g;
        convert_raw_dcfg(g);
    }
}
