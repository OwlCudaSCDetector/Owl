# OWL Quick Reference Card

**Purpose**: Fast lookup for critical functions, data structures, and file locations

---

## Critical File Locations

| Component | File | Lines |
|-----------|------|-------|
| **Main Entry** | `src/owl_analyzer/src/main.rs` | 1-186 |
| **Core Analyzer** | `src/owl_analyzer/analyzer/src/lib.rs` | 1-191 |
| **Trace Collection** | `src/owl_analyzer/analyzer/src/evidence.rs` | 23-70 |
| **Memory Testing** | `src/owl_analyzer/analyzer/src/memory.rs` | 191-238 |
| **KS Test** | `src/owl_analyzer/analyzer/src/hist.rs` | 3-7 |
| **Report Builder** | `src/owl_analyzer/analyzer/src/report.rs` | 208-247 |
| **DCFG Testing** | `src/owl_analyzer/analyzer/src/dcfg.rs` | 106-161 |
| **Kernel Management** | `src/owl_analyzer/analyzer/src/kernel.rs` | 42-46 |
| **Trace Conversion** | `src/owl_analyzer/analyzer/src/trace.rs` | 135-187 |

---

## Execution Flow (One Line)

```
main() → stage3() → leakage_test() → Analyzer::test() → 
run_fix()/run_rnd() → Evidence::merge_trace() → DeviceTest::test() → 
TDcfg::test() → Node::df_test() → MemAccessInstr::test() → 
test_mem_impl() → ks_test_p_value() → Report::builder()
```

---

## Key Functions

### 1. prepare() - Set up trace directory
**File**: `lib.rs:170-183`
```rust
fn prepare(root_path: &str, stage: &str, idx: usize) -> DataAcceptor
```
**Does**: Creates `owl_results/0/{stage}/{idx}/`, sets `$OWL_TRACE`

### 2. exec() - Run target program
**File**: `lib.rs:132-159`
```rust
fn exec(cmd: &str) -> Result<(), ()>
```
**Does**: Spawns process, waits for tracer to write `kernel.json`

### 3. collect_trace() - Load trace
**File**: `lib.rs:92-101`
```rust
fn collect_trace(&mut self, acceptor: DataAcceptor) -> Trace
```
**Does**: Reads `kernel.json`, converts `RawTrace` → `Trace`

### 4. merge_trace() - Aggregate histograms
**File**: `evidence.rs:23-70`
```rust
pub fn merge_trace(&mut self, mut trace: Trace)
```
**Does**: Aligns kernels, sums memory access counts

### 5. test_mem_impl() - Core KS test
**File**: `memory.rs:191-238`
```rust
fn test_mem_impl(l: &MemAccess, r: &MemAccess, m: usize, n: usize) -> f64
```
**Does**: Builds CDFs, finds max difference, returns p-value

### 6. ks_test_p_value() - Statistical formula
**File**: `hist.rs:3-7`
```rust
pub fn ks_test_p_value(x: f64, m: usize, n: usize) -> f64
```
**Formula**: `p = 2 * e^(-2 * x² * (n*m)/(n+m))`

---

## Data Structures

### MemAccess (Histogram)
```rust
pub type MemAccess = BTreeMap<TargetAddr, usize>;
```
**Example**: `{Te0[0x54]: 80, Te0[0x79]: 80}` (address → count)

### TargetAddr
```rust
pub struct TargetAddr {
    pub offset: u64,      // Offset within memory pool
    pub pool: Option<u64>, // Pool ID
    pub ty: MemType,      // Global/Shared/Local
}
```

### Evidence
```rust
pub struct Evidence {
    kernels: Vec<KernelCall>,  // Aggregated kernel traces
}
```

### Report
```rust
pub struct Report {
    pub kernel_leak: HashSet<KernelLeakage>,
    pub cf_leak: HashMap<TraceCtx, HashSet<CFleakage>>,
    pub df_leak: HashMap<TraceCtx, HashSet<DFLeakage>>,
}
```

---

## File Formats

### kernel.json
**Location**: `owl_results/0/{fix|rnd}/{0..9}/kernel.json`

```json
{
  "data": [
    {
      "bt": [...],           // Backtrace (call stack)
      "g": {
        "nodes": [
          {
            "id": 352,       // Basic block ID
            "control_flow": [...],
            "mem_access": [
              {
                "addr": 4621,  // Instruction PC
                "data": [      // Operand data
                  {
                    "access": [
                      {
                        "ty": "Global",
                        "memory": [
                          {"addr": 139907563569152, "count": 8}
                        ]
                      }
                    ]
                  }
                ]
              }
            ]
          }
        ]
      },
      "id": 1,
      "mp": [...],           // Memory pools
      "name": "_Z13aes_128_ecb_g...",
      "ty": 1
    }
  ]
}
```

### report.json
**Location**: `owl_results/0/report.json`

```json
{
  "kernel_leak": [],
  "cf_leak": {...},
  "df_leak": {
    "0x7f68.../...": [
      {
        "kernel": "_Z13aes_128_ecb_g...",
        "instr": 4621,    // Instruction address
        "bb": 352,        // Basic block ID
        "p": 0.00009      // p-value
      }
    ]
  }
}
```

---

## Critical Algorithms

### Two-Pointer CDF Construction
**File**: `memory.rs:191-238`

```rust
// Initialize
l_cdf = 0.0, r_cdf = 0.0, max_diff = 0.0

// Iterate sorted addresses
while not done:
    if l_addr < r_addr:
        l_cdf += l_count * l_weight
        advance l
    else if r_addr < l_addr:
        r_cdf += r_count * r_weight
        advance r
    else:  // equal
        l_cdf += l_count * l_weight
        r_cdf += r_count * r_weight
        advance both
    
    max_diff = max(max_diff, |l_cdf - r_cdf|)

return ks_test_p_value(max_diff, m, n)
```

**Time**: O(n + m), **Space**: O(1)

### Memory Pool Offset Conversion
**File**: `trace.rs:135-187`

```rust
for each memory access at absolute_addr:
    for each pool in mem_pools:
        if pool.start <= absolute_addr < pool.start + pool.size:
            offset = absolute_addr - pool.start
            addr = TargetAddr { offset, pool: Some(pool.id), ty }
            break
```

**Why**: GPU allocates at different addresses each run  
**Result**: Consistent offsets across runs enable comparison

---

## Command Reference

### Build
```bash
cd /home/alan/Documents/Owl/src/owl_analyzer
cargo build --release --features plot
```

### Run Analysis
```bash
cd /home/alan/Documents/Owl/example/crypt-examples/libgpucrypto

# Native
/home/alan/Documents/Owl/src/owl_analyzer/target/release/owl_analyzer \
  -c "/home/alan/Documents/Owl/src/owl-wrapper bin/aes_test -m ENC -l 1024 -f" \
  -r "/home/alan/Documents/Owl/src/owl-wrapper bin/aes_test -m ENC -l 1024" \
  -t 10

# Docker
docker run --rm --gpus all \
  -v /home/alan/Documents/Owl:/workspace \
  -w /workspace/example/crypt-examples/libgpucrypto \
  owl:devel \
  /workspace/src/owl_analyzer/target/release/owl_analyzer \
  -c "/workspace/src/owl-wrapper bin/aes_test -m ENC -l 1024 -f" \
  -r "/workspace/src/owl-wrapper bin/aes_test -m ENC -l 1024" \
  -t 10
```

### Check Results
```bash
# Count leaks
jq '[.df_leak | to_entries[] | .value[] | select(.p < 0.05)] | length' \
  owl_results/0/report.json

# Highly significant
jq '[.df_leak | to_entries[] | .value[] | select(.p < 0.001)] | length' \
  owl_results/0/report.json

# Very highly significant
jq '[.df_leak | to_entries[] | .value[] | select(.p < 0.0001)] | length' \
  owl_results/0/report.json
```

---

## Statistical Thresholds

| p-value | Significance | Meaning |
|---------|--------------|---------|
| p < 0.05 | Significant | 5% chance distributions are same |
| p < 0.01 | Highly significant | 1% chance |
| p < 0.001 | Very highly significant | 0.1% chance |
| p < 0.0001 | Extremely significant | 0.01% chance |

**Default threshold**: 0.05 (can be changed with `-s` flag)

---

## Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `OWL_TRACE` | Trace output directory | `owl_results/0/fix/0/` |
| `OWL_RES` | Results root directory | `./owl_results` |
| `RUST_LOG` | Logging level | `info`, `debug`, `trace` |

---

## Common Issues

### Permission Denied
**Cause**: Running from wrong directory  
**Solution**: Always run from `example/crypt-examples/libgpucrypto/`

### No kernel.json
**Cause**: Tracer didn't write file  
**Solution**: Check `OWL_TRACE` is set, wait 500ms after exec

### Different leak counts (native vs Docker)
**Cause**: Randomness in GPU scheduling  
**Solution**: Normal variation; check highly significant leaks match

### Plot dependencies missing (Docker)
**Cause**: pkg-config or libfontconfig1-dev not installed  
**Solution**: Rebuild image from updated Dockerfile

---

## Directory Structure

```
owl_results/0/
├── fix/
│   ├── 0/kernel.json  (64KB)
│   ├── 1/kernel.json
│   └── ...
├── rnd/
│   ├── 0/kernel.json
│   ├── 1/kernel.json
│   └── ...
└── report.json (118KB)
```

---

## AES Test Case Results

**Target**: AES-128-CBC T-table implementation  
**Iterations**: 10 fixed, 10 random  
**Total DF leaks**: 222  
**Highly significant**: 28  
**Very highly significant**: 17  
**Minimum p-value**: ~0.00009

**Root cause**: T-table lookups leak key bytes through memory access patterns

---

## Further Reading

- **Complete Flow**: `OWL_EXECUTION_FLOW_DETAILED.md`
- **Code Walkthrough**: `OWL_CODE_WALKTHROUGH.md`
- **Setup Guide**: `OWL_COMPLETE_GUIDE.md`
- **AES Tutorial**: `AES_ANALYSIS_TUTORIAL.md`
- **CDF Plotting**: `CDF_VISUALIZATION.md`
- **Docker Setup**: `DOCKER_PLOTTING_SETUP.md`

---

**Last Updated**: October 30, 2025  
**Version**: 1.0
