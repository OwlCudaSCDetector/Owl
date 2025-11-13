# Owl: GPU Side-Channel Detection Framework

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Prerequisites](#prerequisites)
4. [Installation](#installation)
5. [Usage](#usage)
6. [Analysis Example: AES-128-CBC](#analysis-example-aes-128-cbc)
7. [Interpreting Results](#interpreting-results)
8. [Troubleshooting](#troubleshooting)

---

## Overview

Owl is a framework for detecting side-channel vulnerabilities in GPU applications through differential analysis of memory access patterns. It instruments GPU kernels and CPU-side CUDA API calls to identify key-dependent execution behavior that may leak sensitive information.

### Detection Methodology

Owl employs differential analysis:
1. Execute target program multiple times with fixed secret input
2. Execute target program multiple times with random secret inputs  
3. Compare execution traces to identify distinguishable patterns

If fixed-input traces exhibit consistent patterns that differ from random-input traces, a side-channel vulnerability is detected.


---

## System Architecture

### Components

**Pin Tool (cpu_trace.so)**  
Instruments x86 binaries to trace CUDA API calls. Based on Intel Pin 3.28.

**NVBit Tool (gpu_trace.so)**  
Instruments GPU kernels at SASS level to trace memory accesses. Based on NVBit v1.5.5.

**Analyzer (owl_analyzer)**  
Rust-based differential analysis engine that processes traces and identifies leaks.

**Wrapper Script (owl-wrapper)**  
Launcher that combines Pin and NVBit instrumentation via LD_PRELOAD injection.

### Analysis Pipeline

```
Target Binary → owl-wrapper → Pin + NVBit → Trace Files → owl_analyzer → Report
```

---

## Prerequisites

**Hardware:**
- NVIDIA GPU with compute capability 5.0+ (Maxwell or newer)
- 16GB+ RAM recommended

**Software:**
- Linux x86_64 (Ubuntu 20.04/22.04 recommended)
- CUDA Toolkit 11.6.x (NVBit incompatible with CUDA 12.0+)
- GCC 9.x or 10.x
- Rust 1.60+
- Make, Git

---

## Installation

### Install CUDA 11.6

```bash
wget https://developer.download.nvidia.com/compute/cuda/11.6.2/local_installers/cuda_11.6.2_510.47.03_linux.run
sudo sh cuda_11.6.2_510.47.03_linux.run

export PATH=/usr/local/cuda-11.6/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda-11.6/lib64:$LD_LIBRARY_PATH
export CUDA_HOME=/usr/local/cuda-11.6
```

### Install GCC 10 (Ubuntu 22.04)

```bash
sudo apt update
sudo apt install gcc-10 g++-10
```

### Build Owl

```bash
git clone https://github.com/alandebs/Owl.git
cd Owl
make
```

This builds:
- `src/owl_monitor/cpu_trace/obj/intel64/cpu_trace.so` (Pin tool)
- `src/owl_monitor/gpu_trace/gpu_trace.so` (NVBit tool)
- `src/owl_analyzer/target/release/owl_analyzer` (Analyzer)

---

## Usage

### Basic Workflow

1. Prepare target binary with fixed/random input modes
2. Create command file for fixed-input execution
3. Run `owl_analyzer` with fixed and random commands
4. Examine generated report

### Command Syntax

```bash
owl_analyzer \
  --cmds-file <fixed_commands_file> \
  --rand-cmd "<random_command>" \
  -t <iterations>
```

**Parameters:**
- `--cmds-file`: File containing command for fixed-input execution
- `--rand-cmd`: Command for random-input execution
- `-t`: Number of iterations per mode (default: 3)

---

## Analysis Example: AES-128-CBC

This example demonstrates detection of side-channel leakage in the libgpucrypto AES implementation.

### Background

The libgpucrypto library implements AES-128-CBC using GPU-accelerated T-box lookups in shared memory. T-box indices are derived from the encryption key, creating key-dependent memory access patterns.

### Build Target

```bash
cd example/crypt-examples/libgpucrypto
make
```

### Verify Binary

```bash
./bin/aes_test -m ENC -l 1024 -f
```

**Flags:**
- `-m ENC`: Encryption mode
- `-l 1024`: Process 1024 bytes
- `-f`: Use fixed key (deterministic)

### Create Command File

```bash
cat > aes_cmds << 'EOF'
/home/alan/Documents/Owl/src/owl-wrapper /home/alan/Documents/Owl/example/crypt-examples/libgpucrypto/bin/aes_test -m ENC -l 1024 -f
EOF
```

### Run Analysis

```bash
cd ~/Documents/Owl
src/owl_analyzer/target/release/owl_analyzer \
  --cmds-file example/crypt-examples/libgpucrypto/aes_cmds \
  --rand-cmd "src/owl-wrapper example/crypt-examples/libgpucrypto/bin/aes_test -m ENC -l 1024" \
  -t 3
```

### Output Directory Structure

```
owl_results/
└── 0/
    ├── report.json          # Analysis results
    ├── fix/                 # Fixed-key traces
    │   ├── 0/
    │   │   ├── cpu.json
    │   │   └── kernel.json
    │   ├── 1/
    │   └── 2/
    └── rnd/                 # Random-key traces
        ├── 0/
        ├── 1/
        └── 2/
```

---

## Interpreting Results

### Report Format

```bash
cat owl_results/0/report.json
```

**Example: Leak Detected**
```json
{
  "kernel_leak": [
    {
      "kernel": "AES_cbc_128_encrypt_kernel_SharedMem(...)",
      "fix_num": 3,
      "rnd_num": 0
    }
  ],
  "cf_leak": {},
  "df_leak": {}
}
```

**Fields:**
- `kernel`: Name of vulnerable kernel
- `fix_num`: Number of fixed-input runs exhibiting pattern
- `rnd_num`: Number of random-input runs exhibiting pattern

**Interpretation:**
- `fix_num = iterations, rnd_num = 0`: Side-channel detected
- `fix_num = 0, rnd_num = 0`: No consistent pattern found
- `fix_num ≈ rnd_num`: Inconclusive (increase iterations)

**Example: No Leak**
```json
{
  "kernel_leak": [],
  "cf_leak": {},
  "df_leak": {}
}
```

---

## Troubleshooting

### NVBit CUDA Version Error

**Error:** `ASSERT FAIL: nvbit_imp.cpp:1628`

**Cause:** NVBit requires CUDA ≤ 11.8

**Solution:** Install CUDA 11.6 as described in Installation section

### Pin Segmentation Fault

**Error:** `Pin: Segmentation fault (signal 11)`

**Solution:**
```bash
cd ~/Documents/Owl
git status
git checkout src/owl_monitor/cpu_trace/cpu_trace.cpp
make clean && make
```

### Missing CUDA Libraries

**Error:** `libcudart.so.11.0: cannot open shared object file`

**Solution:**
```bash
export LD_LIBRARY_PATH=/usr/local/cuda-11.6/lib64:$LD_LIBRARY_PATH
```

Add to `~/.bashrc` for persistence.

### GCC Version Incompatibility

**Error:** `nvcc fatal: Value 'g++-11' is not defined for option 'ccbin'`

**Solution:**
```bash
sudo apt install gcc-10 g++-10
# Modify compilation to use: nvcc -ccbin=g++-10
```

### Empty Trace Files

**Symptoms:** `kernel.json` files are 0 bytes

**Debug:**
```bash
export OWL_TRACE=/tmp/test
src/owl-wrapper ./bin/aes_test -m ENC -l 1024 -f
ls -lh /tmp/test/
```

Verify NVBit output messages appear. If not, check `gpu_trace.so` exists and is accessible.

---

## References

- **Owl Repository:** https://github.com/alandebs/Owl
- **NVBit Framework:** https://github.com/NVlabs/NVBit
- **Intel Pin:** https://www.intel.com/content/www/us/en/developer/articles/tool/pin-a-dynamic-binary-instrumentation-tool.html
- **CUDA Documentation:** https://docs.nvidia.com/cuda/

