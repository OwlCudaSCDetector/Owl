use monitor::cuda::{BBId, InstrId, KernelTy};

use crate::{matrix::CfMatrix, memory::MemAccessRecord, trace::TraceCtx};

// pub trait KernelTest {
//     fn test(&self, other: &Self, n: usize, m: usize) -> KernelResult;
// }

pub trait DeviceTest {
    fn test(self, other: Self, n: usize, m: usize, threshold: f64) -> TestResult;
}

/// Represent kernel leakage
#[derive(Debug)]
pub struct DiffKernelResult {
    pub ty: KernelTy,
    pub ctx: TraceCtx,
    pub l_num: usize,
    pub r_num: usize,
}

#[derive(Debug)]
pub struct EqKernelResult {
    pub ty: KernelTy,
    pub ctx: TraceCtx,
    pub cf: Vec<NodeCfResult>,
    pub df: Vec<NodeDfResult>,
    // pub name: String,
}

impl EqKernelResult {
    pub fn new() -> Self {
        Self {
            ty: 0,
            ctx: TraceCtx::new(),
            cf: Default::default(),
            df: Default::default(),
            // name: String::new(),
        }
    }

    pub fn ty(mut self, ty: KernelTy) -> Self {
        self.ty = ty;
        self
    }

    pub fn ctx(mut self, ctx: TraceCtx) -> Self {
        self.ctx = ctx;
        self
    }

    pub fn push_cf(&mut self, node_cf: NodeCfResult) {
        // todo!()
        self.cf.push(node_cf);
    }

    pub fn push_df(&mut self, node_df: NodeDfResult) {
        self.df.push(node_df);
    }
}

#[derive(Debug)]
pub struct TestResult {
    pub eq_kernel: Vec<EqKernelResult>,
    pub diff_kernel: Vec<DiffKernelResult>,
}

impl TestResult {
    pub fn new() -> Self {
        Self {
            eq_kernel: Default::default(),
            diff_kernel: Default::default(),
        }
    }

    pub fn push_eq(&mut self, kernel: EqKernelResult) {
        self.eq_kernel.push(kernel);
    }

    pub fn push_diff(&mut self, kernel: DiffKernelResult) {
        self.diff_kernel.push(kernel);
    }
}

// #[derive(Debug)]
// pub enum DeviceResult {
//     EqKernel {
//         cf: Vec<NodeCfResult>,
//         df: Vec<NodeDfResult>,
//     },
//     DiffKernel {
//         ty: KernelTy,
//         ctx: TraceCtx,
//         l_num: usize,
//         r_num: usize,
//     },
// }

// impl DeviceResult {
//     pub fn new_eq() -> Self {
//         Self::EqKernel {
//             cf: Default::default(),
//             df: Default::default(),
//         }
//     }

//     pub fn new_diff(ty: KernelTy, ctx: TraceCtx, l_num: usize, r_num: usize) -> Self {
//         Self::DiffKernel {
//             ty,
//             ctx,
//             l_num,
//             r_num,
//         }
//     }

//     pub fn push_cf(&mut self, node_cf: NodeCfResult) {
//         // todo!()
//         match self {
//             DeviceResult::EqKernel { cf, .. } => cf.push(node_cf),
//             _ => {
//                 panic!()
//             }
//         }
//     }

//     pub fn push_df(&mut self, node_df: NodeDfResult) {
//         match self {
//             DeviceResult::EqKernel { df, .. } => df.push(node_df),
//             _ => {
//                 panic!()
//             }
//         }
//     }

//     // pub fn push_df(&mut self, node_d)
// }

// #[derive(Debug)]
// pub enum KernelResult {
//     EqCtx { ctx: TraceCtx, p: f64 },
//     DiffCtx { ctx: TraceCtx, p: f64 },
// }

// #[derive(Debug)]
// pub struct CtxResult {
//     pub ctx: TraceCtx,
//     pub p: f64,
// }

#[derive(Debug)]
pub struct NodeCfResult {
    pub id: BBId,
    pub p_value: f64,

    pub l_flow: CfMatrix,
    pub r_flow: CfMatrix,
}

impl NodeCfResult {
    pub fn new(id: BBId) -> Self {
        Self {
            id,
            p_value: 0.0,

            l_flow: Default::default(),
            r_flow: Default::default(),
        }
    }

    pub fn p_value(mut self, p: f64) -> Self {
        self.p_value = p;
        self
    }

    pub fn l_flow(mut self, flow: CfMatrix) -> Self {
        self.l_flow = flow;
        self
    }

    pub fn r_flow(mut self, flow: CfMatrix) -> Self {
        self.r_flow = flow;
        self
    }
}

#[derive(Debug)]
pub struct NodeDfResult {
    pub id: BBId,
    pub instr: InstrId,
    pub p_value: f64,
    pub ld: MemAccessRecord,
    pub rd: MemAccessRecord,
}

impl NodeDfResult {
    pub fn new(id: BBId, instr: InstrId) -> Self {
        Self {
            id,
            instr,
            p_value: 0.0,
            ld: MemAccessRecord::new(),
            rd: MemAccessRecord::new(),
            // l_flow: Default::default(),
            // r_flow: Default::default(),
        }
    }

    pub fn p_value(mut self, p: f64) -> Self {
        self.p_value = p;
        self
    }

    pub fn fix_mem(mut self, d: MemAccessRecord) -> Self {
        self.ld = d;
        self
    }

    pub fn rnd_mem(mut self, d: MemAccessRecord) -> Self {
        self.rd = d;
        self
    }
}

// pub enum TestResult {
//     Success(TestReport),
//     Fail
// }

// impl TestResult {
//     pub fn success(p_value: f64) -> Self {
//         Self::Success(TestReport {
//             p_value
//         })
//     }
// }

// pub struct TestReport {
//     pub p_value: f64
// }
