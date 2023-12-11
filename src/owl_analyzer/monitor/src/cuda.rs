pub type KernelId = usize;
pub type KernelTy = u64;

pub type BBId = u32;

pub type InstrId = u64;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemType {
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "LOCAL")]
    Local,
    #[serde(rename = "GENERIC")]
    Generic,
    #[serde(rename = "GLOBAL")]
    Global,
    #[serde(rename = "SHARED")]
    Shared,
    #[serde(rename = "CONSTANT")]
    Constant,
    #[serde(rename = "GLOBAL_TO_SHARED")]
    GlobalToShared,
    #[serde(rename = "SURFACE")]
    Surface,
    #[serde(rename = "TEXTURE")]
    Texture,
}