use crate::{
    align::VecAlignOwned,
    dtest::{DeviceTest, DiffKernelResult, TestResult},
    kernel::KernelCall,
    trace::Trace,
};

#[derive(Debug)]
pub struct Evidence {
    // ctxs: HashMap<TraceCtx, CtxCall>,
    kernels: Vec<KernelCall>,
}

impl Default for Evidence {
    fn default() -> Self {
        Self {
            kernels: Default::default(),
        }
    }
}

impl Evidence {
    pub fn merge_trace(&mut self, mut trace: Trace) {
        // assert_eq!(self.ctx, other.ctx);
        log::debug!("Merge new trace");

        let la = std::mem::take(&mut self.kernels);
        let ra = std::mem::take(&mut trace.kernels);

        log::debug!("evidence len: {}", la.len());
        log::debug!("new trace len: {}", ra.len());

        // align kernel call
        let (la, ra) = la.align_own(ra);

        assert_eq!(la.len(), ra.len());
        log::debug!("aligned length: {}", la.len());
        // log::debug!("la: {}")

        let kernels = la
            .into_iter()
            .zip(ra.into_iter())
            .map(|(l, r)| match (l, r) {
                // same kernel call, further merge
                (Some(mut l), Some(r)) => {
                    // assert_eq!(l.trace.addr, r.trace.addr);
                    log::debug!("Eq kernel call");
                    assert_eq!(r.num, 1);
                    l.num += r.num;
                    l.merge_owned(r);
                    l
                }
                // call in evidence, but not in the new trace
                (Some(l), None) => {
                    log::debug!("Noeq kernel call");
                    l
                }
                // call not in evidence, but in the new trace
                (None, Some(r)) => {
                    log::debug!("New kernel call");
                    r
                }
                _ => {
                    panic!()
                }
            })
            .collect::<Vec<_>>();

        self.kernels = kernels;
    }
}

impl DeviceTest for Evidence {
    fn test(self, other: Self, n: usize, m: usize, threshold: f64) -> TestResult {
        let mut res = TestResult::new();

        // align kernel by ctx
        let (l, r) = self.kernels.align_own(other.kernels);

        log::debug!("Aligned two evidence");

        l.into_iter().zip(r.into_iter()).for_each(|(l, r)| {
            match (l, r) {
                (Some(l), Some(r)) => {
                    log::debug!("Eq kernel call");
                    if l.num != r.num {
                        res.push_diff(DiffKernelResult {
                            ty: l.trace.ty,
                            ctx: l.ctx.clone(),
                            l_num: l.num,
                            r_num: r.num,
                        })
                    }
                    let eq = l.test_owned(r, n, m, threshold);
                    res.push_eq(eq);
                }
                (Some(l), None) => {
                    // kernel not in other
                    log::debug!("Uneq, right is None");
                    res.push_diff(DiffKernelResult {
                        ty: l.trace.ty,
                        ctx: l.ctx.clone(),
                        l_num: l.num,
                        r_num: 0,
                    })
                }
                (None, Some(r)) => {
                    // kernel not in self
                    log::debug!("Uneq, left is None");
                    res.push_diff(DiffKernelResult {
                        ty: r.trace.ty,
                        ctx: r.ctx.clone(),
                        l_num: 0,
                        r_num: r.num,
                    })
                }
                (None, None) => {
                    panic!()
                }
            }
        });

        res
    }
}
