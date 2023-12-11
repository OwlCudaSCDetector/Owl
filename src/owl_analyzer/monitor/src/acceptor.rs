use std::{fs::File, io::BufReader};

use crate::raw::{RawAlloc, RawCsFrame, RawData, RawKernelTrace, RawTrace};

// pub struct Acceptor {
//     p1: String,
//     p2: String,
// }

// impl Acceptor {
//     pub fn new(p1: impl Into<String>, p2: impl Into<String>) -> Self {
//         Self {
//             p1: p1.into(),
//             p2: p2.into(),
//         }
//     }

//     // pub fn run(mut self, p2: &str) -> RawTrace {
//     //     let mem_pools: Arc<Mutex<Vec<GPUMemAlloc>>> = Default::default();
//     //     let mp_clone = mem_pools.clone();

//     //     let p1 = self.p1.to_owned();

//     //     log::info!("recording");
//     //     let th = std::thread::spawn(move || {
//     //         let mut reader = BufReader::new(File::open(p1).unwrap());
//     //         let mut type_buf = [0u8; 1];
//     //         loop {
//     //             match accept(&mut reader, &mut type_buf) {
//     //                 Ok((ty, content)) => {
//     //                     assert_eq!(ty, TYPE_GPU_MEM_ALLOC);
//     //                     mp_clone.lock().unwrap().push(content.into());
//     //                 }
//     //                 Err(_) => {}
//     //             }
//     //         }
//     //     });

//     //     let reader = BufReader::new(std::fs::File::open(p2).unwrap());

//     //     // let mem_pools: RawData = serde_json::from_reader(reader).unwrap();
//     //     let trace: Vec<RawData> = serde_json::from_reader(reader).unwrap();

//     //     // let trace = RawTrace::from_kernels(kernels, mem_pools.lock().unwrap().deref());

//     //     log::debug!("Recording finish");

//     //     RawTrace {
//     //         datas: trace
//     //     }
//     // }
// }

pub struct DataAcceptor {
    path: String,
    // kernels: HashMap<KernelTy, String>,
}

impl DataAcceptor {
    pub fn new(path: String) -> Self {
        Self {
            path,
            // kernels: HashMap::default(),
        }
    }

    pub fn kernel(&self) -> Vec<RawKernelTrace> {
        let file = File::open(format!("{}/kernel.json", self.path)).unwrap();
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader).unwrap();
        if let RawData::Kernel(d) = data {
            // d.iter().for_each(|k| {
            //     self.kernels.insert(k.ty, k.name.clone());
            // });
            d
        } else {
            panic!()
        }
    }

    pub fn context(&self) -> Vec<Vec<RawCsFrame>> {
        let file = File::open(format!("{}/context.json", self.path)).unwrap();
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader).unwrap();
        if let RawData::Context(d) = data {
            d
        } else {
            panic!()
        }
    }

    pub fn alloc(&self) -> Vec<RawAlloc> {
        if let Ok(file) = File::open(format!("{}/alloc.json", self.path)) {
            let reader = BufReader::new(file);
            let data = serde_json::from_reader(reader).unwrap();
            if let RawData::Alloc(d) = data {
                d
            } else {
                panic!()
            }
        } else {
            Vec::new()
        }
    }

    pub fn raw_trace(&self) -> RawTrace {
        let trace = RawTrace {
            kernels: self.kernel(),
            // context: self.context(),
            // alloc: self.alloc(),
        };

        // println!("{:?}", trace);

        trace
    }
}
