use crate::{
    dtest::EqKernelResult,
    merge::Merge,
    trace::{KernelTrace, TraceCtx},
};

#[derive(Debug)]
pub struct KernelCall {
    pub ctx: TraceCtx,
    pub trace: KernelTrace,
    pub num: usize,
}

impl PartialEq for KernelCall {
    fn eq(&self, other: &Self) -> bool {
        self.ctx == other.ctx
        // && self.trace.ty == other.trace.ty
    }
}

impl Default for KernelCall {
    fn default() -> Self {
        Self {
            ctx: TraceCtx::new(),
            trace: Default::default(),
            num: Default::default(),
            // name: Rc::new(String::new()),
        }
    }
}

impl KernelCall {
    pub fn new(ctx: TraceCtx, kernel: KernelTrace) -> Self {
        Self {
            ctx,
            trace: kernel,
            num: 1,
            // name: Rc::new(String::new())
        }
    }

    pub fn test_owned(self, other: Self, n: usize, m: usize, threshold: f64) -> EqKernelResult {
        assert_eq!(self.ctx, other.ctx);
        assert_eq!(self.trace.ty, other.trace.ty);

        self.trace.test(other.trace, n, m, threshold).ctx(self.ctx)
    }

    pub fn merge_owned(&mut self, other: Self) {
        self.trace.merge_own(other.trace);
    }

    pub fn same(&self, other: &Self) -> bool {
        self.ctx == other.ctx && self.num == other.num && self.trace.same(&other.trace)
    }
}
