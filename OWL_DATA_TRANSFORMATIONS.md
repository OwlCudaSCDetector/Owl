# OWL Data Transformation Pipeline

**Purpose**: Visual guide showing how data transforms at each stage  
**Focus**: From raw GPU traces to statistical leak detection

---

## Pipeline Overview

```
┌──────────────┐
│  GPU Tracer  │ Intercepts CUDA calls
│ (owl-wrapper)│ Records memory accesses
└──────┬───────┘
       │ Writes
       ▼
┌──────────────────────────────────────────────────────────┐
│                     kernel.json                          │
│  Raw JSON file with execution trace                      │
│  Size: ~64KB per iteration                               │
└──────────────────────────────┬───────────────────────────┘
       │ DataAcceptor::raw_trace()
       │ Parses JSON → Rust structs
       ▼
┌──────────────────────────────────────────────────────────┐
│                      RawTrace                            │
│  Deserialized Rust structure                             │
│  - kernels: Vec<RawKernelTrace>                          │
│  - Contains absolute memory addresses                     │
└──────────────────────────────┬───────────────────────────┘
       │ From<RawTrace> for Trace
       │ Converts addresses to offsets
       ▼
┌──────────────────────────────────────────────────────────┐
│                       Trace                              │
│  Processed structure with pool offsets                   │
│  - kernels: Vec<KernelCall>                              │
│  - Addresses normalized to pool offsets                  │
└──────────────────────────────┬───────────────────────────┘
       │ Evidence::merge_trace() × 10
       │ Aggregates histograms
       ▼
┌──────────────────────────────────────────────────────────┐
│                     Evidence                             │
│  Aggregated memory access histograms                     │
│  - Fixed: 10 runs merged                                 │
│  - Random: 10 runs merged                                │
└──────────────────────────────┬───────────────────────────┘
       │ DeviceTest::test()
       │ Statistical comparison
       ▼
┌──────────────────────────────────────────────────────────┐
│                    TestResult                            │
│  p-values for each instruction                           │
│  - cf: Control flow leaks                                │
│  - df: Data flow leaks                                   │
└──────────────────────────────┬───────────────────────────┘
       │ Report::builder()
       │ Aggregates and filters
       ▼
┌──────────────────────────────────────────────────────────┐
│                      Report                              │
│  Structured leak information                             │
│  - kernel_leak, cf_leak, df_leak                         │
└──────────────────────────────┬───────────────────────────┘
       │ serde_json::to_string_pretty()
       │ Serializes to JSON
       ▼
┌──────────────────────────────────────────────────────────┐
│                   report.json                            │
│  Final output with all detected leaks                    │
│  Size: ~118KB                                            │
└──────────────────────────────────────────────────────────┘
```

---

## Stage 1: kernel.json (Raw Trace)

### Example Memory Access Entry

```json
{
  "addr": 4621,  // ← Instruction PC (program counter)
  "data": [      // ← Operand 0
    {
      "access": [
        {
          "ty": "Global",  // ← Memory type
          "memory": [
            {
              "addr": 139907563569152,  // ← Absolute GPU address
              "count": 8                // ← Access count
            }
          ]
        }
      ]
    }
  ]
}
```

**Interpretation**:
- Instruction at PC `4621` (e.g., `LD.E.64`)
- Operand 0 accessed global memory
- Address `139907563569152` accessed 8 times (1 per thread in warp)

### Memory Pool Entry

```json
{
  "addr": 139907563569152,  // Pool base address
  "size": 480               // Pool size in bytes
}
```

**Purpose**: Used to convert absolute addresses to offsets

---

## Stage 2: RawTrace → Trace Conversion

### Address Transformation

**Before** (RawTrace):
```rust
RawMemAccessInstr {
    addr: 4621,  // Instruction PC
    data: [
        RawMemAccessOperand {
            access: [
                RawMemAccessType {
                    ty: Global,
                    memory: [
                        { addr: 139907563569152, count: 8 }  // Absolute address
                    ]
                }
            ]
        }
    ]
}
```

**After** (Trace):
```rust
MemAccessInstr {
    instr: InstrId(4621),
    data: [
        BTreeMap {
            TargetAddr {
                offset: 0,      // ← Converted to offset!
                pool: Some(0),  // ← Pool ID
                ty: Global
            }: 8
        }
    ]
}
```

**Conversion Logic**:
```
pool.start = 139907563569152
absolute_addr = 139907563569152
offset = absolute_addr - pool.start = 0

Result: Te0[0] accessed 8 times
```

---

## Stage 3: Trace → Evidence (Aggregation)

### Single Run (Trace)

**Fixed run 0**:
```rust
MemAccessInstr {
    instr: 4621,
    data: [
        {
            TargetAddr { offset: 0x54, ... }: 8,  // Te0[0x54]
            TargetAddr { offset: 0x79, ... }: 8,  // Te0[0x79]
            TargetAddr { offset: 0x4b, ... }: 8,  // Te0[0x4b]
        }
    ]
}
```

**Interpretation**: 8 threads accessed 3 different offsets

### After 10 Runs (Evidence)

**Fixed evidence** (10 runs merged):
```rust
MemAccessInstr {
    instr: 4621,
    data: [
        {
            TargetAddr { offset: 0x54, ... }: 80,  // 8 × 10
            TargetAddr { offset: 0x79, ... }: 80,  // 8 × 10
            TargetAddr { offset: 0x4b, ... }: 80,  // 8 × 10
        }
    ]
}
```

**Random evidence** (10 runs merged):
```rust
MemAccessInstr {
    instr: 4621,
    data: [
        {
            TargetAddr { offset: 0xa3, ... }: 8,   // Run 1
            TargetAddr { offset: 0x61, ... }: 8,   // Run 2
            TargetAddr { offset: 0xd5, ... }: 8,   // Run 3
            // ... 7 more entries
        }
    ]
}
```

**Key Difference**:
- Fixed: 3 addresses, 80 counts each (concentrated)
- Random: 10 addresses, 8 counts each (spread out)

---

## Stage 4: Statistical Testing (CDF Construction)

### Input Histograms

**Fixed**:
```
Address  | Count | Probability | CDF
---------|-------|-------------|------
0x4b     |  80   | 80/240      | 0.333
0x54     |  80   | 80/240      | 0.667
0x79     |  80   | 80/240      | 1.000
```

**Random**:
```
Address  | Count | Probability | CDF
---------|-------|-------------|------
0xa3     |   8   | 8/240       | 0.033
0x61     |   8   | 8/240       | 0.067
0xd5     |   8   | 8/240       | 0.100
...      | ...   | ...         | ...
(10 rows total)
```

### CDF Comparison

```
CDF
1.0 ┤                  Fixed ████
    │                        █
    │                        █
0.67┤                  █     █
    │                  █     █
0.33┤            █     █     █
    │            █     █     █
    │   Random   █     █     █
0.0 ┤   █ █ █ █ █ █ █ █ █ █ █
    └────────────────────────────► Address
       a3 61 d5 ... 4b   54   79
    
    Maximum vertical distance ≈ 0.97
```

### Algorithm Execution

```rust
### Two-Pointer Algorithm

**File**: `memory.rs:191-238`

```rust
// Initialize
l_cdf = 0.0 + (80 / 240) = 0.333
r_cdf = 0.0
max_diff = |0.333 - 0.0| = 0.333

// Iteration 2: Process address 0x54
l_cdf = 0.333 + (80 / 240) = 0.667
r_cdf = 0.0
max_diff = |0.667 - 0.0| = 0.667

// Iteration 3: Process address 0x61 (random only)
l_cdf = 0.667
r_cdf = 0.0 + (8 / 240) = 0.033
max_diff = |0.667 - 0.033| = 0.634

// Iteration 4: Process address 0x79
l_cdf = 0.667 + (80 / 240) = 1.0
r_cdf = 0.033
max_diff = |1.0 - 0.033| = 0.967  ← MAXIMUM!

// ... continue for remaining random addresses ...

// Final result
max_diff = 0.967
p = 2 * e^(-2 * 0.967² * (10*10)/(10+10))
p ≈ 0.00017
```

---

## Stage 5: TestResult → Report

### TestResult Structure

```rust
TestResult {
    diff_kernel: [],  // No kernel count mismatches
    eq_kernel: [
        EqKernelResult {
            ty: KernelTy(1),
            ctx: TraceCtx { cs: [...] },
            cf: [],  // No control flow leaks
            df: [
                NodeDfResult {
                    id: BBId(352),
                    instr: InstrId(4621),
                    p_value: 0.00017,
                    ld: MemAccessRecord { ... },
                    rd: MemAccessRecord { ... }
                },
                // ... 221 more entries
            ]
        }
    ]
}
```

### Report Structure

```rust
Report {
    kernel_leak: {},  // Empty set
    cf_leak: {
        "0x7f68.../...": {}  // Empty set
    },
    df_leak: {
        "0x7f68.../...": {
            DFLeakage {
                kernel: Rc("_Z13aes_128_ecb_g..."),
                instr: InstrId(4621),
                bb: BBId(352),
                p: 0.00017,
                ld: MemAccessRecord { ... },
                rd: MemAccessRecord { ... }
            },
            // ... 221 more entries
        }
    }
}
```

---

## Stage 6: Final report.json

```json
{
  "kernel_leak": [],
  "cf_leak": {
    "0x7f68...35/0x7f71...06/.../0x650...e2": []
  },
  "df_leak": {
    "0x7f68...35/0x7f71...06/.../0x650...e2": [
      {
        "kernel": "_Z13aes_128_ecb_gPKhPhPKjS3_j",
        "instr": 4621,
        "bb": 352,
        "p": 0.00017
      },
      {
        "kernel": "_Z13aes_128_ecb_gPKhPhPKjS3_j",
        "instr": 4655,
        "bb": 368,
        "p": 0.00009
      }
      // ... 220 more leaks
    ]
  }
}
```

---

## Memory Access Pattern Evolution

### Run 0 (Fixed Key: 0x2b7e151628aed2a6abf7158809cf4f3c)

```
kernel.json:
  instruction 4621 → addr 139907563569236 (count: 8)

After pool conversion:
  instruction 4621 → offset 0x54 (count: 8)
  
Interpretation: Te0[plaintext[0] ⊕ 0x2b] accessed
```

### Run 1-9 (Same Fixed Key)

```
All runs:
  instruction 4621 → offset 0x54 (count: 8)

After merging:
  instruction 4621 → offset 0x54 (count: 80)
```

### Random Runs 0-9 (Different Keys)

```
Run 0 (key[0] = 0x##):
  instruction 4621 → offset 0xa3

Run 1 (key[0] = 0x##):
  instruction 4621 → offset 0x61

...

After merging:
  instruction 4621 → {
    0xa3: 8, 0x61: 8, 0xd5: 8, ...
  }
```

---

## Data Size Through Pipeline

| Stage | Data Type | Size (approx) | Count |
|-------|-----------|---------------|-------|
| kernel.json | JSON file | 64 KB | 20 files (10 fix + 10 rnd) |
| RawTrace | Rust struct | ~64 KB | 20 instances |
| Trace | Rust struct | ~32 KB | 20 instances |
| Evidence | Rust struct | ~160 KB | 2 instances (fix + rnd) |
| TestResult | Rust struct | ~50 KB | 1 instance |
| Report | Rust struct | ~40 KB | 1 instance |
| report.json | JSON file | 118 KB | 1 file |

**Total disk usage**: ~1.4 MB (20 kernel.json + 1 report.json)

---

## Type Conversions Summary

```
JSON String
  ↓ serde_json::from_reader
RawTrace (monitor crate types)
  ↓ From<RawTrace> for Trace
Trace (analyzer crate types)
  ↓ Evidence::merge_trace
Evidence (aggregated histograms)
  ↓ DeviceTest::test
TestResult (p-values)
  ↓ Report::builder
Report (structured leaks)
  ↓ serde_json::to_string_pretty
JSON String
```

---

## Critical Data Transformations

### 1. Absolute Address → Pool Offset
```
Input:  addr = 139907563569236
Pool:   base = 139907563569152, size = 480
Output: offset = 139907563569236 - 139907563569152 = 84 (0x54)
```

### 2. Single Run → Aggregated Histogram
```
Run 0:  {0x54: 8}
Run 1:  {0x54: 8}
...
Run 9:  {0x54: 8}
Result: {0x54: 80}
```

### 3. Histogram → CDF
```
Histogram: {0x4b: 80, 0x54: 80, 0x79: 80}
Total:     240
CDF:       {0x4b: 0.333, 0x54: 0.667, 0x79: 1.0}
```

### 4. CDF Difference → p-value
```
max_diff = 0.967
m = 10, n = 10
p = 2 * e^(-2 * 0.967² * 5) = 0.00017
```

---

## Alignment Algorithm

### Problem: Different Kernel Orderings

**Fixed trace**:
```
[KernelA (ctx1), KernelB (ctx2)]
```

**Random trace**:
```
[KernelB (ctx2), KernelA (ctx1)]
```

### Solution: Myers Diff Algorithm

```rust
let (la, ra) = la.align_own(ra);

Result:
  la = [Some(KernelA), Some(KernelB)]
  ra = [Some(KernelA), Some(KernelB)]
```

**Alignment ensures**:
- Same-context kernels compared
- Handles missing kernels (Some vs None)
- Preserves relative ordering

---

## Memory Layout Example

### GPU Memory Allocation

```
Address Range               | Content        | Size
----------------------------|----------------|------
139907563569152 - 569631    | Te0 table      | 480 bytes
139907563569632 - ...       | Te1 table      | 480 bytes
...                         | Te2, Te3, Td0  | ...
```

### After Pool Conversion

```
Pool ID | Offset Range | Interpretation
--------|--------------|---------------
0       | 0x00 - 0x1DF | Te0[0..479]
1       | 0x00 - 0x1DF | Te1[0..479]
2       | 0x00 - 0x1DF | Te2[0..479]
```

**Benefit**: Offset `0x54` is consistent across runs, regardless of absolute allocation address

---

## Statistical Interpretation

### p-value Meaning

| p-value | Interpretation | Action |
|---------|----------------|--------|
| 0.5 | Distributions look very similar | No leak |
| 0.1 | Some difference | Investigate |
| 0.05 | Statistically significant | **Leak detected** |
| 0.001 | Highly significant | **Strong leak** |
| 0.0001 | Very highly significant | **Critical leak** |

### AES Results Distribution

```
p-value range    | Count | Percentage
-----------------|-------|------------
0.00009 - 0.001  |  17   |   7.7%  ← Critical
0.001 - 0.01     |  11   |   5.0%  ← Strong
0.01 - 0.05      | 194   |  87.3%  ← Moderate
Total (p < 0.05) | 222   | 100.0%
```

---

## Execution Timeline

```
Time    | Stage
--------|-------------------------------------------------------
0:00    | main() starts
0:00    | stage3() begins
0:00    | run_fix() iteration 0
0:01    | run_fix() iteration 1
...     | ...
0:10    | run_fix() complete (Evidence with aggregated histograms)
0:10    | run_rnd() iteration 0
0:11    | run_rnd() iteration 1
...     | ...
0:20    | run_rnd() complete
0:20    | DeviceTest::test() begins statistical testing
0:21    | TDcfg::test() testing all basic blocks
0:22    | test_mem_impl() running KS tests
0:23    | Report::builder() generating report
0:24    | Writing report.json
0:24    | Complete!
```

**Total time**: ~4 minutes for 10 iterations (native)

---

## Error Handling

### Missing kernel.json
```rust
DataAcceptor::raw_trace()
  ↓ File not found
  ↓ panic!("kernel.json not found")
```

**Solution**: Ensure tracer writes file, wait 500ms after exec

### Alignment Mismatch
```rust
Evidence::merge_trace()
  ↓ align_own() returns different lengths
  ↓ assert_eq!(la.len(), ra.len())
  ↓ panic!
```

**Solution**: Check traces come from same program

### Empty Histogram
```rust
test_mem_impl()
  ↓ One histogram empty, other not
  ↓ return ks_test_p_value(1.0, m, n)  // Maximum difference
  ↓ p ≈ 0.0 (leak detected)
```

**Interpretation**: One variant accesses memory, other doesn't → leak!

---

**Document Version**: 1.0  
**Last Updated**: October 30, 2025  
**Verified Against**: OWL commit f5f4160
