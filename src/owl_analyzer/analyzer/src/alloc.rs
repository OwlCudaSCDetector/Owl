use monitor::raw::RawAlloc;

use crate::memory::TargetAddr;

#[derive(Debug)]
pub struct MemPool {
    pub start: u64,
    pub size: u64,
}

impl From<RawAlloc> for MemPool {
    fn from(value: RawAlloc) -> Self {
        Self {
            start: value.addr as u64,
            size: value.size as u64,
        }
    }
}

impl MemPool {
    pub fn convert(&self, mut addr: TargetAddr) -> Option<TargetAddr> {
        if addr.offset >= self.start && addr.offset < self.start + self.size {
            addr.offset = addr.offset - self.start;
            addr.pool = Some(self.start);
            return Some(addr);
        } else {
            return None;
        }
    }
}
