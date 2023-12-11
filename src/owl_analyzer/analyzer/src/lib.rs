use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::Stdio,
    rc::Rc,
};

use evidence::Evidence;
use monitor::{cuda::KernelTy, DataAcceptor};
pub use trace::Trace;

// mod cf;
mod evidence;
mod hist;
mod memory;
mod merge;
mod myers_diff;
mod report;
pub use report::Report;

use crate::dtest::DeviceTest;
mod alloc;
mod dcfg;
mod dtest;
mod kernel;
mod matrix;
mod trace;
// pub use trace::Trace;

mod align;

pub struct Analyzer {
    // pub pipe_path: String,
    pub fix_cmd: String,
    pub rnd_cmd: String,
    pub times: usize,
    pub threshold: f64,

    pub trace_path: String,
    pub kernels: HashMap<KernelTy, Rc<String>>,
}

impl Analyzer {
    pub fn run_fix(&mut self) -> Evidence {
        log::info!("run {} times", self.times);

        let mut evidence = Evidence::default();

        // let mut dict = HashMap::new();

        for idx in 0..self.times {
            println!("-------------- {}/{} --------------", idx + 1, self.times);

            // prepare env, dir before execution
            let acceptor = prepare(&self.trace_path, "fix", idx);

            exec(&self.fix_cmd).unwrap();

            let trace = self.collect_trace(acceptor);

            evidence.merge_trace(trace);
        }

        evidence
    }

    pub fn run_rnd(&mut self) -> Evidence {
        log::info!("run {} times", self.times);

        let mut evidence = Evidence::default();

        for idx in 0..self.times {
            println!("-------------- {}/{} --------------", idx + 1, self.times);

            let acceptor = prepare(&self.trace_path, "rnd", idx);

            exec(&self.rnd_cmd).unwrap();

            // let trace: Trace = acceptor.raw_trace().into();
            let trace = self.collect_trace(acceptor);

            evidence.merge_trace(trace);
        }

        evidence
    }

    fn collect_trace(&mut self, acceptor: DataAcceptor) -> Trace {
        let raw_trace = acceptor.raw_trace();
        raw_trace.kernels.iter().for_each(|k| {
            if !self.kernels.contains_key(&k.ty) {
                self.kernels.insert(k.ty, Rc::new(k.name.clone()));
            }
        });

        raw_trace.into()
    }

    pub fn test(&mut self) -> Report {
        // collect traces of fix command
        log::info!("collect fix input traces");
        let fix = self.run_fix();

        // collect traces of random command
        log::info!("collect rand input traces");
        let rnd = self.run_rnd();

        log::info!("Testing");
        let dc_res = DeviceTest::test(fix, rnd, self.times, self.times, self.threshold);
        log::info!("Test finished");
        // log::debug!("{:?}", dc_res);

        log::info!("Generating report");
        let mut builder = Report::builder(std::mem::take(&mut self.kernels));

        dc_res.diff_kernel.into_iter().for_each(|res| {
            builder.add_diff_kernel(res);
        });

        dc_res.eq_kernel.into_iter().for_each(|res| {
            builder.add_eq_kernel(res);
        });

        builder.build()
    }
}

fn exec(cmd: &str) -> Result<(), ()> {
    log::debug!("execute test program");

    let mut p = std::process::Command::new("sh");
    p.arg("-c");
    p.arg(cmd);
    let stdout = p
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute child")
        .stdout
        .unwrap();

    let reader = BufReader::new(stdout);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));

    Ok(())
}

fn prepare(root_path: &str, stage: &str, idx: usize) -> DataAcceptor {
    let trace_path = format!("{root_path}/{stage}/{idx}/");
    log::info!("Recorded trace path: {}", trace_path);
    std::fs::create_dir_all(&trace_path).unwrap();

    std::env::set_var("OWL_TRACE", &trace_path);

    DataAcceptor::new(trace_path)
}

pub fn get_trace(cmd: &str, root_path: &str) -> Trace {
    let acceptor = prepare(root_path, "fix", 0);
    exec(cmd).unwrap();

    acceptor.raw_trace().into()
}
