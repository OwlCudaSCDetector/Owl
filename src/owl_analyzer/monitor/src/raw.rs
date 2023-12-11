use serde::{Deserialize, Serialize};

use crate::cuda::{KernelId, KernelTy, MemType};

type Addr = u32;

#[derive(Deserialize, Serialize, Debug)]
pub struct RawDirect {
    pub start: Addr,
    pub end: Addr,
}

#[derive(Debug, Deserialize)]
pub struct RawTrace {
    // pub datas: Vec<RawData>
    pub kernels: Vec<RawKernelTrace>,
    // pub context: Vec<Vec<RawCsFrame>>,
    // pub alloc: Vec<RawAlloc>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RawData {
    Alloc(Vec<RawAlloc>),
    Context(Vec<Vec<RawCsFrame>>),
    Kernel(Vec<RawKernelTrace>),
}

impl RawData {
    pub fn get_kernel(self) -> Vec<RawKernelTrace> {
        let a = if let RawData::Kernel(kernels) = self {
            kernels
        } else {
            panic!("Not kernel")
        };
        a
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawAlloc {
    pub addr: u64,
    // pub name: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawCsFrame {
    // pub cs: Vec<(usize, String)>,
    pub addr: usize,
    // pub name: String,
    pub func: String,
    pub file: String,
    pub offset: usize,
}

// #[derive(Debug, Dser)]
pub type RawContext = Vec<RawCsFrame>;

#[derive(Debug, Deserialize, Serialize)]
pub struct RawKernelTrace {
    pub id: KernelId,
    pub ty: KernelTy,
    pub g: RawDCFG,
    pub bt: RawContext,
    pub mp: Vec<RawAlloc>,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawCf {
    pub from: isize,
    pub to: isize,
    pub num: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawDCFG {
    pub nodes: Vec<RawNode>,
    // pub edges: Vec<RawEdge>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawEdge {
    pub direct: RawDirect,
    // pub positions: Vec<RawPos>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawNode {
    pub id: u32,
    pub control_flow: Vec<RawCf>,
    pub mem_access: RawMemAccessRecord,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawPos {
    pub pos: usize,
    pub count: usize,
}

pub type RawMemAccessRecord = Vec<RawMemAccessInstr>;

#[derive(Debug, Deserialize, Serialize)]
pub struct RawMemAccessInstr {
    pub addr: u64,
    pub data: Vec<RawMemAccessPos>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawMemAccessPos {
    pub pos: usize,
    pub access: Vec<RawMemAccessWithType>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawMemAccessWithType {
    pub memory: Vec<RawMemAccess>,
    #[serde(rename = "type")]
    pub ty: MemType,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawMemAccess {
    pub addr: u64,
    pub count: usize,
}
