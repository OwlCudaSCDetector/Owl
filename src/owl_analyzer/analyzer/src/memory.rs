use std::{collections::BTreeMap, ops::AddAssign};

use monitor::{
    cuda::{InstrId, MemType},
    raw::{RawMemAccessInstr, RawMemAccessRecord},
};

use crate::hist::ks_test_p_value;

pub type MemAccess = BTreeMap<TargetAddr, usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct MemAccessInstr {
    pub instr: InstrId,
    pub data: Vec<MemAccess>,
}

impl Default for MemAccessInstr {
    fn default() -> Self {
        Self {
            instr: InstrId::default(),
            data: Default::default(),
        }
    }
}

impl MemAccessInstr {
    pub fn add_mem_access(&mut self, pos: usize, addr: TargetAddr, num: usize) {
        self.data.resize(pos + 1, MemAccess::default());
        let map = self.data.get_mut(pos).unwrap();
        if !map.contains_key(&addr) {
            map.insert(addr, num);
        } else {
            *map.get_mut(&addr).unwrap() += num;
        }
    }

    pub fn instr(mut self, instr: InstrId) -> Self {
        self.instr = instr;
        self
    }

    pub fn access_sum(&self) -> Vec<usize> {
        self.data
            .iter()
            .map(|m| m.values().sum::<usize>())
            .collect()
    }

    pub fn test(&self, other: &Self, m: usize, n: usize) -> f64 {
        assert_eq!(self.instr, other.instr);
        // log::info!()
        log::debug!("{:?}", self.data);
        log::debug!("{:?}", other.data);
        let mut minimum = 100.0;
        self.data
            .iter()
            .zip(other.data.iter())
            .map(|(l, r)| test_mem_impl(l, r, m, n))
            .for_each(|p| {
                if p < minimum {
                    minimum = p;
                }
            });

        minimum
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct TargetAddr {
    pub offset: u64,
    pub pool: Option<u64>,
    // host: bool,
    pub ty: MemType,
}

impl TargetAddr {
    pub fn new(offset: u64, pool: Option<u64>, ty: MemType) -> Self {
        Self { offset, pool, ty }
    }

    pub fn is_valid(&self) -> bool {
        // self.pool != 0
        // self.pool.is_some()
        true
    }
}

// impl Eq for

impl PartialOrd for TargetAddr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // self.offset.partial_cmp(other)

        match self.offset.partial_cmp(&other.offset) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        self.ty.partial_cmp(&other.ty)
    }
}

impl Ord for TargetAddr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // self.partial_cmp()
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for TargetAddr {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.ty == other.ty
    }
}

impl Eq for TargetAddr {}

#[derive(Debug, Clone, PartialEq)]
pub struct MemAccessRecord {
    pub instrs: Vec<MemAccessInstr>,
}

impl MemAccessRecord {
    pub fn new() -> Self {
        Self { instrs: Vec::new() }
    }

    /// Add memory access record of instruction
    pub fn add_instr_mem_access(
        &mut self,
        instr: InstrId,
        pos: usize,
        addr: TargetAddr,
        num: usize,
    ) {
        self.instrs
            .iter_mut()
            .find(|mem| mem.instr == instr)
            // instr exist
            .and_then(|mem| {
                mem.add_mem_access(pos, addr, num);
                Some(())
            })
            // instr not exist
            .or_else(|| {
                let mut new = MemAccessInstr::default().instr(instr);
                new.add_mem_access(pos, addr, num);
                self.instrs.push(new);
                Some(())
            });
    }
}

fn test_mem_impl(l: &MemAccess, r: &MemAccess, m: usize, n: usize) -> f64 {
    match (l.is_empty(), r.is_empty()) {
        (true, true) => return ks_test_p_value(0.0, m, n),
        (false, false) => {}
        _ => return ks_test_p_value(1.0, m, n),
    }

    let l_weight = 1.0
        / l.iter()
            .filter(|(v, _)| v.is_valid())
            .map(|(_, c)| *c)
            .sum::<usize>() as f64;
    let r_weight = 1.0
        / r.iter()
            .filter(|(v, _)| v.is_valid())
            .map(|(_, c)| *c)
            .sum::<usize>() as f64;

    let mut l_cdf = 0.0;
    let mut r_cdf = 0.0;
    let mut max_diff = 0.0;

    let mut l_iter = l.iter().filter(|(v, _)| v.is_valid());
    let mut r_iter = r.iter().filter(|(v, _)| v.is_valid());

    let mut l_cur = l_iter.next();
    let mut r_cur = r_iter.next();

    loop {
        match (l_cur, r_cur) {
            (None, None) => break,
            (None, Some((_, r_num))) => {
                // log::deb!("r: {}: {r_num}", r_addr.offset);
                r_cur = r_iter.next();
                r_cdf += *r_num as f64 * r_weight;
            }
            (Some((_, l_num)), None) => {
                l_cur = l_iter.next();
                l_cdf += *l_num as f64 * l_weight;
            }
            (Some((l_addr, l_num)), Some((r_addr, r_num))) => {
                if l_addr < r_addr {
                    l_cur = l_iter.next();
                    l_cdf += *l_num as f64 * l_weight;
                } else if l_addr > r_addr {
                    r_cur = r_iter.next();
                    r_cdf += *r_num as f64 * r_weight;
                } else {
                    l_cur = l_iter.next();
                    l_cdf += *l_num as f64 * l_weight;
                    r_cur = r_iter.next();
                    r_cdf += *r_num as f64 * r_weight;
                }
            }
        }
        if (l_cdf - r_cdf).abs() > max_diff {
            max_diff = (l_cdf - r_cdf).abs();
        }
    }

    ks_test_p_value(max_diff, m, n)
}

impl AddAssign for MemAccessInstr {
    fn add_assign(&mut self, mut rhs: Self) {
        let mut length = 0;

        self.data
            .iter_mut()
            .zip(rhs.data.iter())
            .enumerate()
            .for_each(|(idx, (l, r))| {
                r.into_iter().for_each(|(addr, num)| {
                    if l.contains_key(&addr) {
                        *l.get_mut(&addr).unwrap() += num;
                    } else {
                        l.insert(*addr, *num);
                    }
                });
                length = idx + 1;
            });

        // push remain memory access
        if rhs.data.len() > length {
            for v in rhs.data[length..].iter_mut() {
                self.data.push(std::mem::take(v));
            }
        }
    }
}

impl AddAssign for MemAccessRecord {
    fn add_assign(&mut self, rhs: Self) {
        self.instrs
            .iter_mut()
            .zip(rhs.instrs.into_iter())
            .for_each(|(l, r)| {
                l.add_assign(r);
            })
    }
}

impl From<RawMemAccessRecord> for MemAccessRecord {
    fn from(value: RawMemAccessRecord) -> Self {
        let mut mem_access = MemAccessRecord::new();

        // convert MemAccessInstr
        value.into_iter().for_each(|r_m| {
            mem_access.instrs.push(MemAccessInstr::from(r_m));
        });

        // sort by instr
        mem_access.instrs.sort_by(|l, r| l.instr.cmp(&r.instr));

        mem_access
    }
}

impl From<RawMemAccessInstr> for MemAccessInstr {
    fn from(value: RawMemAccessInstr) -> Self {
        let mut new = MemAccessInstr::default().instr(value.addr);
        value.data.into_iter().enumerate().for_each(|(pos, r_m)| {
            r_m.access.into_iter().for_each(|mty| {
                let ty = mty.ty;
                mty.memory.into_iter().for_each(|m| {
                    new.add_mem_access(pos, TargetAddr::new(m.addr, None, ty), m.count);
                });
            })
        });

        new
    }
}
