use std::{
    collections::{BTreeMap, BTreeSet},
    ops::AddAssign,
};

use monitor::raw::RawCf;
use ndarray::{Array1, Array2};

use crate::{align::VecAlign, hist::ks_test_p_value};
// use polars::prelude::*;

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct CfMatrix {
    flows: BTreeMap<isize, BTreeMap<isize, usize>>,
}

impl Default for CfMatrix {
    fn default() -> Self {
        Self {
            flows: BTreeMap::default(),
        }
    }
}

impl AddAssign for CfMatrix {
    fn add_assign(&mut self, rhs: Self) {
        rhs.flows.into_iter().for_each(|(src, dst_map)| {
            dst_map.into_iter().for_each(|(dst, count)| {
                self.insert_cf(src, dst, count);
            })
        });
    }
}

impl CfMatrix {
    pub fn is_empty(&self) -> bool {
        self.flows.is_empty()
    }

    // convert rawcf to flows
    pub fn from_raw(value: Vec<RawCf>) -> Self {
        let mut flows = BTreeMap::new();
        value.into_iter().for_each(|v| {
            insert_cf_impl(&mut flows, v.from, v.to, v.num);
        });

        Self { flows }
    }

    pub fn insert_cf(&mut self, src: isize, dst: isize, count: usize) {
        insert_cf_impl(&mut self.flows, src, dst, count);
    }

    pub fn build_matrix(&self) -> Array2<f64> {
        build_matrix_impl(&self.flows)
    }

    pub fn align(&self, other: &Self) {
        let mut l_dst = BTreeSet::new();
        for f in self.flows.values() {
            f.keys().for_each(|dst| {
                l_dst.insert(*dst);
            })
        }

        let mut r_dst = BTreeSet::new();
        for f in other.flows.values() {
            f.keys().for_each(|dst| {
                r_dst.insert(*dst);
            })
        }
    }

    pub fn test(&self, other: &Self, n: usize, m: usize) -> f64 {
        struct SrcFlow<'a, T> {
            id: &'a isize,
            inner: &'a T,
        }

        impl<'a, T> PartialEq for SrcFlow<'a, T> {
            fn eq(&self, other: &Self) -> bool {
                *self.id == *other.id
            }
        }

        /// calcuate p(src, dst)
        fn normlize_src_dst(
            left: &BTreeMap<isize, usize>,
            right: &BTreeMap<isize, usize>,
        ) -> Vec<(f64, f64)> {
            log::debug!("normlize");
            let l_sum: f64 = left.values().sum::<usize>() as f64;
            let r_sum: f64 = right.values().sum::<usize>() as f64;

            let l: Vec<_> = left
                .iter()
                .map(|(id, inner)| SrcFlow { id, inner })
                .collect();
            let r: Vec<_> = right
                .iter()
                .map(|(id, inner)| SrcFlow { id, inner })
                .collect();

            let (l, r) = l.align(&r);
            // let (lh, rh): (Vec<_>, Vec<_>) =
            l.into_iter()
                .zip(r.into_iter())
                .map(|(l, r)| match (l, r) {
                    (Some(l), Some(r)) => (*l.inner as f64 / l_sum, *r.inner as f64 / r_sum),
                    (Some(l), None) => (*l.inner as f64 / l_sum, 0.0),
                    (None, Some(r)) => (0.0, *r.inner as f64 / r_sum),
                    (None, None) => {
                        panic!()
                    }
                })
                .collect()
            // .unzip();

            // let p = ks_test_aligned_norm(&lh, &rh, n, m);
            // p
        }

        // let mut l_sum = 0.0;
        // let mut r_sum = 0.0;
        let mut lv = Vec::new();
        let mut rv = Vec::new();

        // if self.flows.is_empty() && other.flows.is_empty() {
        //     println!("{:?}", self.flows);
        // }
        log::debug!("{}, {}", self.flows.len(), other.flows.len());

        // align flow by source
        let l: Vec<_> = self
            .flows
            .iter()
            .map(|(id, inner)| SrcFlow { id, inner })
            .collect();
        let r: Vec<_> = other
            .flows
            .iter()
            .map(|(id, inner)| SrcFlow { id, inner })
            .collect();

        let (l, r) = l.align(&r);

        l.into_iter().zip(r.into_iter()).for_each(|(l, r)| {
            match (l, r) {
                (Some(l), Some(r)) => {
                    log::debug!("eq src");
                    let p = normlize_src_dst(l.inner, r.inner);
                    p.into_iter().for_each(|(l, r)| {
                        // l_sum += l;
                        // r_sum += r;
                        // v.push((l, r));
                        if l != r {
                            log::trace!("different cf on node")
                        }
                        lv.push(l);
                        rv.push(r);
                    });
                }
                (Some(_), None) => {
                    log::trace!("Uneq src, r is none");
                    lv.push(1.0);
                    rv.push(0.0);
                }
                (None, Some(_)) => {
                    log::trace!("Uneq src, l is none");
                    lv.push(0.0);
                    rv.push(1.0);
                }
                (None, None) => {}
            }
            // if (l_sum - r_sum).abs() > max_diff {
            //     max_diff = (l_sum - r_sum).abs()
            // }
        });

        log::debug!("calc max diff");

        let mut max_diff = 0.0;

        let (l_sum, r_sum): (f64, f64) = (lv.iter().sum(), rv.iter().sum());

        lv.iter().zip(rv.iter()).for_each(|(l, r)| {
            if (l / l_sum - r / r_sum).abs() > max_diff {
                max_diff = (l / l_sum - r / r_sum).abs();
            }
        });

        let p = ks_test_p_value(max_diff, n, m);
        p
        // .collect();
    }
}

fn insert_cf_impl(
    flows: &mut BTreeMap<isize, BTreeMap<isize, usize>>,
    src: isize,
    dst: isize,
    count: usize,
) {
    // update current cf with new bbid
    flows.values_mut().for_each(|dst_map| {
        insert_dst(dst_map, dst, 0);
    });

    // generate new cf: from -> to
    if !flows.contains_key(&src) {
        let empty_map = if flows.is_empty() {
            BTreeMap::from([(dst, 0)])
        } else {
            // generate new BTreeMap with same dst
            flows
                .values()
                .next()
                .unwrap()
                .iter()
                .map(|(dst, _)| (*dst, 0))
                .collect()
        };

        flows.insert(src, empty_map);
    }
    let map = flows.get_mut(&src).unwrap();
    *map.get_mut(&dst).unwrap() += count;
}

fn insert_dst(to_map: &mut BTreeMap<isize, usize>, to: isize, val: usize) {
    if !to_map.contains_key(&to) {
        to_map.insert(to, 0);
    }

    *to_map.get_mut(&to).unwrap() += val;
}

fn build_matrix_impl(data: &BTreeMap<isize, BTreeMap<isize, usize>>) -> Array2<f64> {
    let v: Vec<_> = data
        .values()
        .map(|d| btree_map_2_column(d))
        .collect::<Vec<_>>();

    let width = v.len();
    let flattened: Array1<_> = v.into_iter().flat_map(|row| row.to_vec()).collect();
    let height = flattened.len() / width;
    flattened.into_shape((width, height)).unwrap()
}

fn btree_map_2_column(data: &BTreeMap<isize, usize>) -> Array1<f64> {
    let array: Vec<_> = data.iter().map(|(_, value)| *value as f64).collect();
    array.into()
}

#[cfg(test)]
mod test {
    use ndarray::Array2;
    use std::collections::BTreeMap;

    use super::{build_matrix_impl, insert_cf_impl};

    #[test]
    pub fn test_insert_cf() {
        let mut flows = BTreeMap::default();

        insert_cf_impl(&mut flows, 1, 3, 3);
        insert_cf_impl(&mut flows, 1, 3, 3);
        insert_cf_impl(&mut flows, 1, 4, 1);
        insert_cf_impl(&mut flows, 2, 5, 2);
        insert_cf_impl(&mut flows, 2, 4, 2);
        insert_cf_impl(&mut flows, 2, 4, 2);

        let m = build_matrix_impl(&flows);
        // println!("{:?}", m);

        assert_eq!(
            Array2::from(Vec::from([[6.0, 1.0, 0.0], [0.0, 4.0, 2.0]])),
            m
        );
    }
}
