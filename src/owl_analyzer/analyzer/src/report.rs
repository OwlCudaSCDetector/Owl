use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

use monitor::cuda::{BBId, InstrId, KernelTy};
use serde::{ser::SerializeStruct, Serialize};

use crate::{
    dtest::{DiffKernelResult, EqKernelResult},
    matrix::CfMatrix,
    memory::MemAccessRecord,
    trace::TraceCtx,
};

// #[derive(Serialize)]
pub struct KernelLeakage {
    pub ctx: TraceCtx,
    pub kernel: Rc<String>,
    pub fix_num: usize,
    pub rnd_num: usize,
    // pub p: f64,
}

impl Serialize for KernelLeakage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("KernelLeakage", 4)?;
        state.serialize_field("ctx", &self.ctx)?;
        state.serialize_field("kernel", self.kernel.as_str())?;
        state.serialize_field("fix_num", &self.fix_num)?;
        state.serialize_field("rnd_num", &self.rnd_num)?;

        state.end()
    }
}

impl PartialEq for KernelLeakage {
    fn eq(&self, other: &Self) -> bool {
        self.kernel == other.kernel
    }
}

impl Eq for KernelLeakage {}

impl Hash for KernelLeakage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kernel.hash(state);
    }
}

// #[derive(Serialize)]
pub struct CFleakage {
    pub kernel: Rc<String>,
    // pub ctx: TraceCtx,
    pub bb: BBId,
    pub p: f64,
    // pub kernel_name: String,
    pub l_flow: CfMatrix,
    pub r_flow: CfMatrix,
}

impl Serialize for CFleakage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("CFleakage", 4)?;
        state.serialize_field("kernel", self.kernel.as_str())?;
        state.serialize_field("bb", &self.bb)?;
        state.serialize_field("l_flow", &self.l_flow)?;
        state.serialize_field("r_flow", &self.r_flow)?;
        state.serialize_field("p", &self.p)?;

        state.end()
    }
}

impl PartialEq for CFleakage {
    fn eq(&self, other: &Self) -> bool {
        // self.kernel == other.kernel &&
        self.bb == other.bb
    }
}

impl Eq for CFleakage {}

impl Hash for CFleakage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // self.kernel.hash(state);
        self.bb.hash(state)
    }
}

pub struct DFLeakage {
    pub kernel: Rc<String>,
    pub instr: InstrId,
    pub bb: BBId,
    pub p: f64,
    pub ld: MemAccessRecord,
    pub rd: MemAccessRecord,
}

impl PartialEq for DFLeakage {
    fn eq(&self, other: &Self) -> bool {
        // self.kernel == other.kernel &&
        self.bb == other.bb
    }
}

impl Eq for DFLeakage {}

impl Hash for DFLeakage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // self.kernel.hash(state);
        self.bb.hash(state);
        self.instr.hash(state);
    }
}

impl Serialize for DFLeakage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("DFLeakage", 4)?;
        state.serialize_field("kernel", &self.kernel.as_str())?;
        state.serialize_field("instr", &self.instr)?;
        state.serialize_field("bb", &self.bb)?;
        state.serialize_field("p", &self.p)?;
        // state.serialize_field("ld", &self.ld)?;
        // state.serialize_field("rd", &self.rd)?;

        state.end()
    }
}

// #[derive(Serialize)]
// pub struct MemoryPosLeakage {
//     pub kernel: KernelTy,
//     pub bb: BBId,
//     pub p: f64,
// }

// impl PartialEq for MemoryPosLeakage {
//     fn eq(&self, other: &Self) -> bool {
//         self.kernel == other.kernel && self.bb == other.bb
//     }
// }

// impl Eq for MemoryPosLeakage {}

// impl Hash for MemoryPosLeakage {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.kernel.hash(state);
//         self.bb.hash(state)
//     }
// }

#[derive(Serialize)]
pub struct MemoryAddrLeakage {
    pub kernel: KernelTy,
    pub bb: BBId,
    pub p: f64,
}

impl PartialEq for MemoryAddrLeakage {
    fn eq(&self, other: &Self) -> bool {
        self.kernel == other.kernel && self.bb == other.bb
    }
}

impl Eq for MemoryAddrLeakage {}

impl Hash for MemoryAddrLeakage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kernel.hash(state);
        self.bb.hash(state)
    }
}

pub struct ReportBuilder {
    // threshold: f64,
    report: Report,
    kernels: HashMap<KernelTy, Rc<String>>,
}

impl ReportBuilder {
    pub fn build(self) -> Report {
        self.report
    }

    pub fn add_diff_kernel(&mut self, res: DiffKernelResult) {
        let leakage = KernelLeakage {
            ctx: res.ctx,
            kernel: self.kernels.get(&res.ty).unwrap().clone(),
            fix_num: res.l_num,
            rnd_num: res.r_num,
        };

        self.report.kernel_leak.insert(leakage);
    }

    /// Save equal kernel test result
    pub fn add_eq_kernel(&mut self, res: EqKernelResult) {
        let ctx = res.ctx;

        if !self.report.cf_leak.contains_key(&ctx) {
            self.report.cf_leak.insert(ctx.clone(), HashSet::default());
        }

        let set = self.report.cf_leak.get_mut(&ctx).unwrap();

        // add cf leakage
        set.extend(
            res.cf
                .into_iter()
                // .filter(|cf| cf.p_value < self.threshold)
                .map(|cf| CFleakage {
                    bb: cf.id,
                    p: cf.p_value,
                    l_flow: cf.l_flow,
                    r_flow: cf.r_flow,
                    kernel: self.kernels.get(&res.ty).unwrap().clone(),
                }),
        );

        if !self.report.df_leak.contains_key(&ctx) {
            self.report.df_leak.insert(ctx.clone(), HashSet::default());
        }

        let set = self.report.df_leak.get_mut(&ctx).unwrap();

        // add df leakage
        set.extend(
            res.df
                .into_iter()
                // .filter(|df| df.p_value < self.threshold)
                .map(|df| DFLeakage {
                    kernel: self.kernels.get(&res.ty).unwrap().clone(),
                    instr: df.instr,
                    bb: df.id,
                    p: df.p_value,
                    ld: df.ld,
                    rd: df.rd,
                }),
        );
    }
}

#[derive(Serialize)]
pub struct Report {
    pub kernel_leak: HashSet<KernelLeakage>,
    pub cf_leak: HashMap<TraceCtx, HashSet<CFleakage>>,
    pub df_leak: HashMap<TraceCtx, HashSet<DFLeakage>>,
    // pub name_map: HashMap<TraceCtx, String>,
}

impl Report {
    pub fn builder(kernels: HashMap<KernelTy, Rc<String>>) -> ReportBuilder {
        ReportBuilder {
            // threshold: 100.0,
            report: Report::new(),
            kernels,
        }
    }

    pub fn new() -> Self {
        Self {
            // leakages: Default::default(),
            kernel_leak: Default::default(),
            cf_leak: Default::default(),
            df_leak: Default::default(),
            // name_map: Default::default(),
        }
    }
}
