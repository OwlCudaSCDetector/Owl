# Owl Leak Detection: Complete Code Walkthrough

This document traces through the **actual code execution** to show exactly how Owl detects side-channel leaks, from trace collection to statistical analysis to reporting.

---

## Table of Contents

1. [Execution Flow Overview](#execution-flow-overview)
2. [Step 1: Command Line Entry Point](#step-1-command-line-entry-point)
3. [Step 2: Trace Collection (Fixed + Random)](#step-2-trace-collection-fixed--random)
4. [Step 3: Statistical Testing](#step-3-statistical-testing)
5. [Step 4: Report Generation](#step-4-report-generation)
6. [Complete Call Stack](#complete-call-stack)

---

## Execution Flow Overview

```
┌─────────────────────────────────────────────────────────────┐
│ main.rs::main()                                             │
│ - Parse command line arguments                              │
│ - Setup logging                                             │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ main.rs::stage3() → leakage_test()                          │
│ - Create Analyzer instance                                  │
│ - Call analyzer.test()                                      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ lib.rs::Analyzer::test()                                    │
│ - Collect fixed traces (run_fix)                            │
│ - Collect random traces (run_rnd)                           │
│ - Perform statistical testing (DeviceTest::test)            │
│ - Build report                                              │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ dtest.rs::DeviceTest::test()                                │
│ - Align kernels from fixed and random evidence              │
│ - For each kernel pair, call test_owned()                   │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ trace.rs::KernelTrace::test()                               │
│ - Test DCFG (control + data flow graph)                     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ dcfg.rs::TDcfg::test()                                      │
│ - For each basic block (node):                              │
│   • cf_test() - Control flow testing                        │
│   • df_test() - Data flow testing                           │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ memory.rs::test_mem_impl()                                  │
│ - Build CDFs for fixed and random distributions             │
│ - Compute maximum CDF difference                            │
│ - Calculate p-value using KS test                           │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ hist.rs::ks_test_p_value()                                  │
│ - Apply Kolmogorov-Smirnov formula                          │
│ - Return p-value                                            │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ report.rs::Report::build()                                  │
│ - Format results as JSON                                    │
│ - Save to owl_results/0/report.json                         │
└─────────────────────────────────────────────────────────────┘
```

---

## Step 1: Command Line Entry Point

**File:** `src/owl_analyzer/src/main.rs`

### Function: `main()`

```rust
#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    env_logger::builder()
        .filter_level(match cli.log {
            3 => log::LevelFilter::Info,
            // ... other levels
        })
        .init();
    
    // Parse commands
    let cmds = parse_commands(&cli);
    let rnd_cmd = cli.rand_cmd.as_ref().unwrap();
    
    // Determine threshold for statistical significance
    let threshold = if let Some(sign) = cli.sign {
        1.0 - sign  // Convert significance level to threshold
    } else {
        2.0  // Default (not used in our case)
    };
    
    // Run Stage 3: Leakage testing
    stage3(cmds, rnd_cmd, &res_root, cli.test_times.unwrap_or(2), threshold);
    
    Ok(())
}
```

**What happens:**
1. Parse command line: `owl_analyzer -c "..." -r "..." -t 10`
2. Extract:
   - Fixed command: `cmds`
   - Random command: `rnd_cmd`
   - Test times: `10`
   - Threshold: `0.05` (default for significance testing)

### Function: `stage3()`

```rust
fn stage3(cmds: Vec<&str>, rnd_cmd: &str, res_root: &str, times: usize, threshold: f64) {
    for (idx, cmd) in cmds.iter().enumerate() {
        let trace_path = format!("{}/{}", &res_root, idx);
        
        // Call the main leakage test function
        let report = leakage_test(&trace_path, cmd, rnd_cmd, times, threshold);
        
        // Save report
        let mut f = std::fs::File::create(&format!("{}/report.json", &trace_path)).unwrap();
        f.write_all(serde_json::to_string_pretty(&report).unwrap().as_bytes()).unwrap();
        
        log::info!("Report saved to {}/report.json", &trace_path);
    }
}
```

### Function: `leakage_test()`

```rust
pub fn leakage_test(
    trace_path: &str,
    cmd: &str,
    rand_cmd: &str,
    times: usize,
    threshold: f64,
) -> Report {
    // Create analyzer instance
    let mut analyzer = Analyzer {
        fix_cmd: cmd.to_owned(),
        rnd_cmd: rand_cmd.to_owned(),
        times,
        threshold,
        trace_path: trace_path.to_owned(),
        kernels: Default::default(),
    };
    
    // Run the test
    analyzer.test()
}
```

**Key data:**
- `fix_cmd`: Command with `-f` flag (fixed key)
- `rnd_cmd`: Command without `-f` flag (random keys)
- `times`: 10 (number of iterations)
- `threshold`: 0.05 (p-value threshold for significance)
- `trace_path`: `owl_results/0/`

---

## Step 2: Trace Collection (Fixed + Random)

**File:** `src/owl_analyzer/analyzer/src/lib.rs`

### Function: `Analyzer::test()`

```rust
pub fn test(&mut self) -> Report {
    // PHASE 1: Collect fixed-key traces
    log::info!("collect fix input traces");
    let fix = self.run_fix();
    
    // PHASE 2: Collect random-key traces
    log::info!("collect rand input traces");
    let rnd = self.run_rnd();
    
    // PHASE 3: Statistical testing
    log::info!("Testing");
    let dc_res = DeviceTest::test(fix, rnd, self.times, self.times, self.threshold);
    log::info!("Test finished");
    
    // PHASE 4: Build report
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
```

### Function: `Analyzer::run_fix()` - Collect Fixed Traces

```rust
pub fn run_fix(&mut self) -> Evidence {
    log::info!("run {} times", self.times);
    
    let mut evidence = Evidence::default();
    
    // Run the fixed-key command N times (e.g., 10)
    for idx in 0..self.times {
        println!("-------------- {}/{} --------------", idx + 1, self.times);
        
        // Prepare trace directory
        let acceptor = prepare(&self.trace_path, "fix", idx);
        
        // Execute the test program
        exec(&self.fix_cmd).unwrap();
        
        // Collect the trace
        let trace = self.collect_trace(acceptor);
        
        // Merge this trace into evidence
        evidence.merge_trace(trace);
    }
    
    evidence
}
```

**What happens:**
1. Loop 10 times (for `-t 10`)
2. Each iteration:
   - Set `OWL_TRACE=owl_results/0/fix/{idx}/`
   - Execute: `owl-wrapper bin/aes_test -m ENC -l 1024 -f`
   - NVBit intercepts GPU kernel and records memory accesses
   - Save trace to: `owl_results/0/fix/{idx}/kernel.json`
3. Merge all 10 traces into one `Evidence` structure

### Function: `prepare()` - Setup Trace Directory

```rust
fn prepare(root_path: &str, stage: &str, idx: usize) -> DataAcceptor {
    // Build path: owl_results/0/fix/5/
    let mut path = PathBuf::from(root_path);
    path.push(stage);
    path.push(idx.to_string());
    
    std::fs::create_dir_all(&path).unwrap();
    let abs = std::fs::canonicalize(&path).unwrap_or(path);
    let trace_path = abs.to_string_lossy().to_string() + "/";
    
    // Set environment variable for NVBit
    log::info!("Recorded trace path: {}", trace_path);
    std::env::set_var("OWL_TRACE", &trace_path);
    
    DataAcceptor::new(trace_path)
}
```

### Function: `exec()` - Execute Test Program

```rust
fn exec(cmd: &str) -> Result<(), ()> {
    log::debug!("execute test program");
    
    // Execute shell command
    let mut builder = std::process::Command::new("sh");
    builder.arg("-c").arg(cmd);
    builder.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = builder.spawn().expect("failed to execute child");
    
    // Wait for completion
    let status = child.wait().expect("failed to wait on child");
    log::debug!("child exit status: {:?}", status.code());
    
    // Give tracer time to flush files
    thread::sleep(Duration::from_millis(500));
    
    Ok(())
}
```

**What runs:**
```bash
sh -c "owl-wrapper bin/aes_test -m ENC -l 1024 -f"
```

This triggers:
1. `owl-wrapper` loads NVBit instrumentation
2. `aes_test` runs AES encryption
3. NVBit intercepts GPU kernel launch
4. Records every memory access
5. Saves to `$OWL_TRACE/kernel.json`

### Data Structure: `Evidence`

**File:** `src/owl_analyzer/analyzer/src/evidence.rs`

```rust
pub struct Evidence {
    kernels: Vec<KernelCall>,  // List of kernel invocations
}

impl Evidence {
    pub fn merge_trace(&mut self, mut trace: Trace) {
        // Align kernels from new trace with existing evidence
        let (la, ra) = la.align_own(ra);
        
        // Merge aligned kernels
        let kernels = la.into_iter()
            .zip(ra.into_iter())
            .map(|(l, r)| match (l, r) {
                (Some(mut l), Some(r)) => {
                    l.num += r.num;  // Increment count
                    l.merge_owned(r);  // Merge memory access data
                    l
                }
                // ... handle unmatched cases
            })
            .collect();
        
        self.kernels = kernels;
    }
}
```

**After 10 fixed runs, `Evidence` contains:**
```rust
Evidence {
    kernels: [
        KernelCall {
            ctx: "cuLaunchKernel/.../main",
            trace: KernelTrace {
                ty: "AES_cbc_128_encrypt_kernel_SharedMem(...)",
                g: TDcfg {
                    nodes: {
                        42: Node {
                            mem_access: MemAccessRecord {
                                instrs: [
                                    MemAccessInstr {
                                        instr: 0x12345,
                                        data: [
                                            {
                                                offset_0xac: 10,  // ← Accessed 10 times (all runs)
                                                offset_0x14: 0,   // ← Never accessed
                                            }
                                        ]
                                    }
                                ]
                            }
                        },
                        // ... more nodes (basic blocks)
                    }
                }
            },
            num: 10  // ← Kernel launched 10 times
        }
    ]
}
```

### Function: `Analyzer::run_rnd()` - Collect Random Traces

```rust
pub fn run_rnd(&mut self) -> Evidence {
    log::info!("run {} times", self.times);
    
    let mut evidence = Evidence::default();
    
    for idx in 0..self.times {
        println!("-------------- {}/{} --------------", idx + 1, self.times);
        
        let acceptor = prepare(&self.trace_path, "rnd", idx);
        
        // Execute with RANDOM key (no -f flag)
        exec(&self.rnd_cmd).unwrap();
        
        let trace = self.collect_trace(acceptor);
        evidence.merge_trace(trace);
    }
    
    evidence
}
```

**After 10 random runs, `Evidence` contains:**
```rust
Evidence {
    kernels: [
        KernelCall {
            trace: KernelTrace {
                g: TDcfg {
                    nodes: {
                        42: Node {
                            mem_access: {
                                instrs: [
                                    {
                                        instr: 0x12345,
                                        data: [
                                            {
                                                offset_0xac: 1,  // ← Run 1 accessed this
                                                offset_0x14: 2,  // ← Runs 2,3 accessed this
                                                offset_0x28: 1,  // ← Run 4 accessed this
                                                offset_0xa7: 1,  // ← Run 5 accessed this
                                                // ... different offsets from different keys
                                            }
                                        ]
                                    }
                                ]
                            }
                        }
                    }
                }
            },
            num: 10
        }
    ]
}
```

---

## Step 3: Statistical Testing

**File:** `src/owl_analyzer/analyzer/src/evidence.rs`

### Function: `DeviceTest::test()` for Evidence

```rust
impl DeviceTest for Evidence {
    fn test(self, other: Self, n: usize, m: usize, threshold: f64) -> TestResult {
        let mut res = TestResult::new();
        
        // Align kernels by call context
        let (l, r) = self.kernels.align_own(other.kernels);
        
        l.into_iter().zip(r.into_iter()).for_each(|(l, r)| {
            match (l, r) {
                (Some(l), Some(r)) => {
                    // Check if same number of runs
                    if l.num != r.num {
                        res.push_diff(DiffKernelResult {
                            ty: l.trace.ty,
                            ctx: l.ctx.clone(),
                            l_num: l.num,
                            r_num: r.num,
                        })
                    }
                    
                    // Perform detailed testing
                    let eq = l.test_owned(r, n, m, threshold);
                    res.push_eq(eq);
                }
                // ... handle unmatched kernels
            }
        });
        
        res
    }
}
```

**File:** `src/owl_analyzer/analyzer/src/kernel.rs`

### Function: `KernelCall::test_owned()`

```rust
pub fn test_owned(self, other: Self, n: usize, m: usize, threshold: f64) -> EqKernelResult {
    assert_eq!(self.ctx, other.ctx);
    assert_eq!(self.trace.ty, other.trace.ty);
    
    // Test the kernel trace (DCFG comparison)
    self.trace.test(other.trace, n, m, threshold).ctx(self.ctx)
}
```

**File:** `src/owl_analyzer/analyzer/src/trace.rs`

### Function: `KernelTrace::test()`

```rust
pub fn test(self, other: Self, n: usize, m: usize, threshold: f64) -> EqKernelResult {
    assert_eq!(self.addr, other.addr);
    assert_eq!(self.ty, other.ty);
    
    log::debug!("Testing DCFG");
    
    // Test the control + data flow graph
    self.g.test(other.g, n, m, threshold).ty(self.ty)
}
```

**File:** `src/owl_analyzer/analyzer/src/dcfg.rs`

### Function: `TDcfg::test()` - Core Testing Logic

```rust
pub fn test(self, other: Self, n: usize, m: usize, threshold: f64) -> EqKernelResult {
    let mut res = EqKernelResult::new();
    
    // Get all basic block nodes
    let ln: Vec<_> = self.nodes.values().collect();
    let rn: Vec<_> = other.nodes.values().collect();
    
    // Align nodes by ID
    let (l, r) = ln.align(&rn);
    assert_eq!(l.len(), r.len());
    
    // Test each pair of nodes
    l.into_iter().zip(r.into_iter()).for_each(|(l, r)| {
        match (l, r) {
            (Some(l), Some(r)) => {
                // ===== CONTROL FLOW TEST =====
                log::debug!("Eq node");
                let p = l.cf_test(r, n, m);
                if p < threshold {
                    res.push_cf(NodeCfResult::new(l.id).p_value(p));
                }
                
                // ===== DATA FLOW TEST =====
                if let Some(dfp) = l.df_test(r, n, m) {
                    dfp.into_iter()
                        .filter(|(p, _)| *p < threshold)
                        .for_each(|(p, instr)| {
                            res.push_df(NodeDfResult::new(l.id, instr).p_value(p))
                        })
                };
            }
            // ... handle unmatched nodes
        }
    });
    
    res
}
```

### Function: `Node::df_test()` - Data Flow Testing

```rust
pub fn df_test(&self, other: &Self, m: usize, n: usize) -> Option<Vec<(f64, InstrId)>> {
    if self.mem_access.instrs.is_empty() && other.mem_access.instrs.is_empty() {
        return None;
    }
    
    log::debug!("Test mem access in node: {}", self.id);
    
    // Test each memory instruction
    let p = self.mem_access.instrs.iter()
        .zip(other.mem_access.instrs.iter())
        .map(|(l, r)| {
            let p_value = l.test(r, m, n);
            (p_value, l.instr)
        })
        .collect();
    
    Some(p)
}
```

**File:** `src/owl_analyzer/analyzer/src/memory.rs`

### Function: `MemAccessInstr::test()`

```rust
pub fn test(&self, other: &Self, m: usize, n: usize) -> f64 {
    assert_eq!(self.instr, other.instr);
    
    log::debug!("{:?}", self.data);
    log::debug!("{:?}", other.data);
    
    let mut minimum = 100.0;
    
    // Test each operand's memory access distribution
    self.data.iter()
        .zip(other.data.iter())
        .map(|(l, r)| test_mem_impl(l, r, m, n))
        .for_each(|p| {
            if p < minimum {
                minimum = p;
            }
        });
    
    minimum  // Return lowest (most significant) p-value
}
```

### Function: `test_mem_impl()` - Kolmogorov-Smirnov Test Implementation

```rust
fn test_mem_impl(l: &MemAccess, r: &MemAccess, m: usize, n: usize) -> f64 {
    // Handle empty cases
    match (l.is_empty(), r.is_empty()) {
        (true, true) => return ks_test_p_value(0.0, m, n),
        (false, false) => {}
        _ => return ks_test_p_value(1.0, m, n),
    }
    
    // Calculate weights for normalization (create probability distribution)
    let l_weight = 1.0 / l.iter()
        .filter(|(v, _)| v.is_valid())
        .map(|(_, c)| *c)
        .sum::<usize>() as f64;
    
    let r_weight = 1.0 / r.iter()
        .filter(|(v, _)| v.is_valid())
        .map(|(_, c)| *c)
        .sum::<usize>() as f64;
    
    // Build CDFs and find maximum difference
    let mut l_cdf = 0.0;
    let mut r_cdf = 0.0;
    let mut max_diff = 0.0;
    
    let mut l_iter = l.iter().filter(|(v, _)| v.is_valid());
    let mut r_iter = r.iter().filter(|(v, _)| v.is_valid());
    
    let mut l_cur = l_iter.next();
    let mut r_cur = r_iter.next();
    
    // Iterate through all addresses in sorted order
    loop {
        match (l_cur, r_cur) {
            (None, None) => break,
            
            (None, Some((_, r_num))) => {
                // Only right has this address
                r_cur = r_iter.next();
                r_cdf += *r_num as f64 * r_weight;
            }
            
            (Some((_, l_num)), None) => {
                // Only left has this address
                l_cur = l_iter.next();
                l_cdf += *l_num as f64 * l_weight;
            }
            
            (Some((l_addr, l_num)), Some((r_addr, r_num))) => {
                if l_addr < r_addr {
                    // Left address is smaller
                    l_cur = l_iter.next();
                    l_cdf += *l_num as f64 * l_weight;
                } else if l_addr > r_addr {
                    // Right address is smaller
                    r_cur = r_iter.next();
                    r_cdf += *r_num as f64 * r_weight;
                } else {
                    // Same address in both
                    l_cur = l_iter.next();
                    l_cdf += *l_num as f64 * l_weight;
                    r_cur = r_iter.next();
                    r_cdf += *r_num as f64 * r_weight;
                }
            }
        }
        
        // Update maximum difference
        if (l_cdf - r_cdf).abs() > max_diff {
            max_diff = (l_cdf - r_cdf).abs();
        }
    }
    
    // Calculate and return p-value
    ks_test_p_value(max_diff, m, n)
}
```

**Example execution for AES:**

```
Fixed distribution:
  offset_0xac: 10 accesses → probability = 10/10 = 1.0

Random distribution:
  offset_0xa3: 1 access  → probability = 1/10 = 0.1
  offset_0x61: 1 access  → probability = 1/10 = 0.1
  offset_0xd5: 1 access  → probability = 1/10 = 0.1
  ... (10 different addresses, each with 1 access)

Build CDFs:
  Address    | Fixed CDF | Random CDF | Difference
  -----------|-----------|------------|------------
  0xa3       | 0.0       | 0.1        | 0.1
  0x61       | 0.0       | 0.2        | 0.2
  0xd5       | 0.0       | 0.3        | 0.3
  ...
  0xac       | 1.0       | 0.1        | 0.9  ← Maximum!
  ...
  
  max_diff = 0.9
```

**File:** `src/owl_analyzer/analyzer/src/hist.rs`

### Function: `ks_test_p_value()` - Calculate P-value

```rust
pub fn ks_test_p_value(x: f64, m: usize, n: usize) -> f64 {
    let m = m as f64;
    let n = n as f64;
    
    // Kolmogorov-Smirnov formula
    2f64 * E.powf(-2f64 * x.powi(2) * ((n * m) / (n + m)))
}
```

**Example calculation:**
```
x = 0.9 (max_diff)
m = 10 (number of fixed traces)
n = 10 (number of random traces)

p = 2 * e^(-2 * 0.9^2 * (10*10)/(10+10))
  = 2 * e^(-2 * 0.81 * 5)
  = 2 * e^(-8.1)
  = 2 * 0.0003
  = 0.0006

Since 0.0006 < 0.05 (threshold) → LEAK DETECTED!
```

---

## Step 4: Report Generation

**File:** `src/owl_analyzer/analyzer/src/report.rs`

### Structure: `Report`

```rust
pub struct Report {
    kernel_leak: Vec<KernelLeakEntry>,
    cf_leak: BTreeMap<String, Vec<CfLeakEntry>>,
    df_leak: BTreeMap<String, Vec<DfLeakEntry>>,
}

#[derive(serde::Serialize)]
struct KernelLeakEntry {
    ctx: TraceCtx,
    kernel: String,
    fix_num: usize,
    rnd_num: usize,
}

#[derive(serde::Serialize)]
struct DfLeakEntry {
    kernel: String,
    instr: usize,
    bb: usize,
    p: f64,
}
```

### Function: `Report::builder()`

```rust
impl Report {
    pub fn builder(kernels: HashMap<KernelTy, Rc<String>>) -> ReportBuilder {
        ReportBuilder {
            kernel_leak: Vec::new(),
            cf_leak: BTreeMap::new(),
            df_leak: BTreeMap::new(),
            kernels,
        }
    }
}
```

### Function: `ReportBuilder::add_eq_kernel()`

```rust
impl ReportBuilder {
    pub fn add_eq_kernel(&mut self, res: EqKernelResult) {
        let kernel_name = self.kernels.get(&res.ty).unwrap().clone();
        let ctx_str = serde_json::to_string(&res.ctx).unwrap();
        
        // Add control flow leaks
        if !res.cf.is_empty() {
            let entry = self.cf_leak.entry(ctx_str.clone()).or_insert(Vec::new());
            res.cf.into_iter().for_each(|cf| {
                entry.push(CfLeakEntry {
                    kernel: kernel_name.to_string(),
                    bb: cf.id,
                    p: cf.p_value,
                });
            });
        }
        
        // Add data flow leaks
        if !res.df.is_empty() {
            let entry = self.df_leak.entry(ctx_str).or_insert(Vec::new());
            res.df.into_iter().for_each(|df| {
                entry.push(DfLeakEntry {
                    kernel: kernel_name.to_string(),
                    instr: df.instr,
                    bb: df.id,
                    p: df.p_value,
                });
            });
        }
    }
    
    pub fn build(self) -> Report {
        Report {
            kernel_leak: self.kernel_leak,
            cf_leak: self.cf_leak,
            df_leak: self.df_leak,
        }
    }
}
```

**Final JSON output:**

```json
{
  "kernel_leak": [],
  "cf_leak": {},
  "df_leak": {
    "0x74c76059d6c5:cuLaunchKernel/...": [
      {
        "kernel": "AES_cbc_128_encrypt_kernel_SharedMem(...)",
        "instr": 11984,
        "bb": 1056,
        "p": 0.0006
      },
      {
        "kernel": "AES_cbc_128_encrypt_kernel_SharedMem(...)",
        "instr": 6752,
        "bb": 1056,
        "p": 0.00009
      }
      // ... 377 more entries
    ]
  }
}
```

---

## Complete Call Stack

Here's the complete function call stack for one leak detection:

```
main()                                          [main.rs]
  ↓
  stage3()                                      [main.rs]
    ↓
    leakage_test()                              [main.rs]
      ↓
      Analyzer::test()                          [lib.rs]
        ↓
        ├─→ Analyzer::run_fix()                 [lib.rs]
        │     ↓
        │     ├─→ prepare()                     [lib.rs]
        │     │     └─→ set OWL_TRACE env var
        │     │
        │     ├─→ exec()                        [lib.rs]
        │     │     └─→ Execute: bin/aes_test -f
        │     │           └─→ NVBit records traces
        │     │
        │     ├─→ collect_trace()               [lib.rs]
        │     │     └─→ Load kernel.json
        │     │
        │     └─→ Evidence::merge_trace()       [evidence.rs]
        │           └─→ Merge into evidence
        │
        ├─→ Analyzer::run_rnd()                 [lib.rs]
        │     └─→ (same steps, without -f flag)
        │
        ├─→ DeviceTest::test()                  [evidence.rs]
        │     ↓
        │     └─→ KernelCall::test_owned()      [kernel.rs]
        │           ↓
        │           └─→ KernelTrace::test()     [trace.rs]
        │                 ↓
        │                 └─→ TDcfg::test()     [dcfg.rs]
        │                       ↓
        │                       ├─→ Node::cf_test()
        │                       │
        │                       └─→ Node::df_test()   [dcfg.rs]
        │                             ↓
        │                             └─→ MemAccessInstr::test()  [memory.rs]
        │                                   ↓
        │                                   └─→ test_mem_impl()   [memory.rs]
        │                                         ↓
        │                                         ├─→ Build CDFs
        │                                         ├─→ Find max_diff
        │                                         │
        │                                         └─→ ks_test_p_value()  [hist.rs]
        │                                               └─→ Return p-value
        │
        └─→ Report::builder()                   [report.rs]
              ↓
              ├─→ add_eq_kernel()
              │     └─→ Add DF/CF leaks
              │
              └─→ build()
                    └─→ Return Report
```

---

## Summary

**The complete leak detection process:**

1. **main.rs::main()** - Parse command line arguments
2. **lib.rs::Analyzer::run_fix()** - Collect 10 fixed-key traces
3. **lib.rs::Analyzer::run_rnd()** - Collect 10 random-key traces
4. **evidence.rs::Evidence::merge_trace()** - Aggregate memory access data
5. **dcfg.rs::TDcfg::test()** - Compare fixed vs random for each basic block
6. **memory.rs::test_mem_impl()** - Build CDFs, compute max difference
7. **hist.rs::ks_test_p_value()** - Calculate statistical p-value
8. **report.rs::Report::build()** - Generate JSON report

**Key algorithms:**
- **Trace merging:** Combine multiple runs into histograms
- **CDF comparison:** Kolmogorov-Smirnov two-sample test
- **Statistical threshold:** p < 0.05 indicates significant leak

**For AES:**
- Fixed traces: All access Te0[0xac] → CDF jumps to 1.0 at offset 0xac
- Random traces: Each accesses different offset → CDF gradual increase
- Max difference: 0.9
- P-value: 0.0006
- **Result: LEAK DETECTED!**
