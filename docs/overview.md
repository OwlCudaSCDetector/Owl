# Owl: CUDA Side-Channel Leakage Detection – Overview and Guide

This guide explains what Owl is, how it’s organized, how the pieces work together, and exactly how to build, run, and interpret results.

## What is Owl?

Owl detects side‑channel leakage in CUDA applications by comparing execution traces between fixed inputs and randomized inputs. It instruments both the CPU side (CUDA API calls) and GPU kernels, collects traces, and then performs differential analysis to flag potential leaks.

High level:
- Trace the program under fixed inputs
- Trace it again under randomized inputs
- Compare distributions of events (e.g., memory access patterns)
- Report where statistically significant differences appear

## Architecture (data flow)

1. You invoke your program through the wrapper script `src/owl-wrapper`.
2. Two tracers capture runtime data:
   - CPU tracer (Intel Pin) records CUDA API calls and memory activity
   - GPU tracer (NVBit) instruments kernels to capture device‑side events
3. Traces are written to files/pipes.
4. The Rust analyzer reads the traces for both fixed and randomized runs and computes leakage statistics.
5. A JSON report summarizes findings per kernel/instruction/basic block.

## Repository layout

Top-level control
- `Makefile` – Orchestrates the build:
  - `make analyzer` builds the Rust analyzer
  - `make monitor` builds tracers and copies shared objects to `build/lib`
  - `ARCH` variable sets the GPU SM architecture (e.g., `86` for RTX 3090 Ti)
- `Dockerfile` – Container build environment (optional)
- `README.md` – Basic quick start

Examples
- `example/cuda-examples/` – Minimal CUDA example
  - `randaccess.cu` – Test kernel
  - `Makefile` – Builds `randaccess`
  - `cmds` – Example commands for Docker default path `/root/owl`
  - `cmds.local` – Example commands using native absolute paths (you can create this)
- `example/crypt-examples/libgpucrypto/` – Larger crypto workloads (AES/RSA/SHA), with its own Makefile

Instrumentation (tracers)
- `src/owl_monitor/`
  - `gpu_trace/` – NVBit-based GPU tracer
    - `gpu_trace.cu`, `device_trace.cu` – Instrument kernels
    - `cfg.*`, `bt.*`, `dump.*` – Helpers for CFG, backtraces, output
    - `helper/pipe/pipe.cu` – Pipe utilities for streaming data
    - `Makefile` – Builds `gpu_trace.so`
  - `cpu_trace/` – Intel Pin tracer (host/CPU side)
    - `cpu_trace.cpp` – Hooks CUDA API calls: `cudaMalloc*`, `cudaHostAlloc`, `cudaHostGetDevicePointer`, `cudaFree`, etc.
    - `makefile` – Uses Pin’s build system; `make` auto‑downloads Pin into `src/owl_monitor/pin_root`
  - `core/` – NVBit headers and static lib (e.g., `libnvbit.a`)

Analyzer
- `src/owl_analyzer/` – Rust analyzer
  - `Cargo.toml`, `src/` – Implementation of differential analysis
  - Output binary: `src/owl_analyzer/target/release/owl_analyzer`

Outputs
- `build/lib/gpu_trace.so`, `build/lib/cpu_trace.so` – Tracer shared objects
- `owl_results/<i>/report.json` – Analyzer results per run index

## Prerequisites (native build)

- NVIDIA GPU + drivers
- CUDA Toolkit (tested with CUDA 11.5)
- Rust toolchain (Cargo)
- Clang/clang++ (for some GPU tracer C++ sources)
- GCC 10 available (as nvcc host compiler) recommended for CUDA 11.x

Note on host compiler: CUDA 11.x + GCC 11 can cause nvcc/libstdc++ template errors. Prefer GCC 10. The GPU tracer Makefile can auto‑select `g++-10` when available.

## Build (native)

From the repository root:

```bash
make ARCH=86
```

Artifacts:
- `build/lib/gpu_trace.so`
- `build/lib/cpu_trace.so`
- `src/owl_analyzer/target/release/owl_analyzer`

Tip: `ARCH=86` targets NVIDIA Ampere (e.g., RTX 3090 Ti). Adjust as needed for your GPU.

## Run the CUDA example (native)

### Try it quickly (native paths)

Tip: Prefer a one-liner? See [Convenience: `make test-native`](#convenience-make-test-native).

```bash
# Set once per shell
REPO=/home/alan/Documents/Owl

# Build the example
cd $REPO/example/cuda-examples
make

# Create cmds.local if it doesn't exist yet
if [ ! -f cmds.local ]; then
  printf "%s\n" "$REPO/src/owl-wrapper $REPO/example/cuda-examples/randaccess" > cmds.local
fi

# Run the analyzer (2 runs per phase)
$REPO/src/owl_analyzer/target/release/owl_analyzer \
  --cmds-file $REPO/example/cuda-examples/cmds.local \
  --rand-cmd "$REPO/src/owl-wrapper $REPO/example/cuda-examples/randaccess" \
  -t 2
```

### Step-by-step

1) Build the example:

```bash
cd example/cuda-examples
make
```

2) Provide commands for fixed inputs (native paths). For convenience, create `cmds.local` with a single line:

```text
/absolute/path/to/repo/src/owl-wrapper /absolute/path/to/repo/example/cuda-examples/randaccess
```

Example for `/home/alan/Documents/Owl`:

```text
/home/alan/Documents/Owl/src/owl-wrapper /home/alan/Documents/Owl/example/cuda-examples/randaccess
```

3) Run the analyzer:

```bash
/absolute/path/to/repo/src/owl_analyzer/target/release/owl_analyzer \
  --cmds-file /absolute/path/to/repo/example/cuda-examples/cmds.local \
  --rand-cmd "/absolute/path/to/repo/src/owl-wrapper /absolute/path/to/repo/example/cuda-examples/randaccess" \
  -t 2
```

### Convenience: `make test-native`

If your checkout includes the `test-native` helper target, you can run the full native flow with one command:

```bash
make ARCH=86 test-native         # default: RUNS=2
make ARCH=86 RUNS=3 test-native  # customize number of runs per phase
```

What it does:
- Builds the analyzer and monitor
- Builds `example/cuda-examples`
- Creates `example/cuda-examples/cmds.local` with native paths if missing
- Runs the analyzer end-to-end and writes `owl_results/<i>/report.json`


- `--cmds-file` lists commands for the fixed‑input phase
- `--rand-cmd` is the command to run for randomized inputs
- `-t 2` runs each phase twice (increase for stronger statistics)

## Run the CUDA example (Docker)

Prerequisites:
- Docker installed and the NVIDIA Container Toolkit configured for GPU access.
- An NVIDIA GPU on the host.

Build the image (from the repo root):

```bash
docker build -t owl:devel .
```

Recommended: run the container with your current checkout mounted at `/root/owl` and GPU access enabled:

```bash
# From your repo root on the host
docker run --gpus all --rm -it \
  -v "$PWD":/root/owl \
  -w /root/owl \
  owl:devel bash
```

Inside the container, build and run just like native (paths are already `/root/owl` and the example’s `cmds` uses those paths):

```bash
make ARCH=86
cd example/cuda-examples && make
src/owl_analyzer/target/release/owl_analyzer \
  --cmds-file example/cuda-examples/cmds \
  --rand-cmd "src/owl-wrapper example/cuda-examples/randaccess" \
  -t 2
```

Optional one-liner (no interactive shell):

```bash
docker run --gpus all --rm -it \
  -v "$PWD":/root/owl -w /root/owl owl:devel \
  bash -lc 'make ARCH=86 && cd example/cuda-examples && make && \
    src/owl_analyzer/target/release/owl_analyzer \
      --cmds-file example/cuda-examples/cmds \
      --rand-cmd "src/owl-wrapper example/cuda-examples/randaccess" -t 2'
```

Notes:
- The provided Dockerfile sets up the toolchain. When you mount your repo, the container uses your current sources rather than cloning.
- If you choose not to mount, the Dockerfile’s cloning step targets the upstream repository; you’ll need network and credentials if using SSH URLs. Mounting is preferred.

## Interpreting results

The analyzer writes a JSON report to `owl_results/<i>/report.json`.

Example snippet:

```json
{
  "kernel_leak": [],
  "cf_leak": { "<stack>": [] },
  "df_leak": {
    "<stack>": [
      { "kernel": "randomAccessKernel(int**, int, int, int, int)", "instr": 784, "bb": 736, "p": 1.9999 },
      { "kernel": "randomAccessKernel(int**, int, int, int, int)", "instr": 816, "bb": 736, "p": 1.9999 }
    ]
  }
}
```

- `df_leak` lists differential leakage findings
  - `kernel`: kernel name
  - `instr`: instruction index
  - `bb`: basic block id
  - `p`: a statistic indicating strength of leakage (higher = stronger)
- `cf_leak` would contain control‑flow leakage findings if any
- `kernel_leak` aggregates at kernel granularity (empty if none)

## Using Docker (optional)

Prefer the end-to-end steps in “Run the CUDA example (Docker)” above. The short version is: build the image, run with `--gpus all` and a bind mount to `/root/owl`, then follow the native build/run steps inside the container.

### Convenience: `make test-docker`

If you prefer a one-shot run from the host, use the helper target:

```bash
make ARCH=86 test-docker         # default: RUNS=2
make ARCH=86 RUNS=3 test-docker  # customize number of runs per phase
```

What it does:
- Builds the `owl:devel` Docker image (or `DOCKER_IMAGE` if you override it)
- Runs the container with `--gpus all`, bind-mounts the repo to `/root/owl`
- Builds analyzer and monitor, builds the CUDA example, and runs the analyzer
- Writes results under `owl_results/<i>/report.json` on your host checkout

## Configuration knobs

- `ARCH=<sm>` – GPU SM arch (e.g., `86`)
- Host compiler for nvcc:
  - `g++-10` preferred for CUDA 11.x (Makefile auto‑selects when available)
- Analyzer arguments:
  - `--cmds-file <file>` – fixed input commands
  - `--rand-cmd "<cmd>"` – random input command
  - `-t <N>` – number of runs per phase

## Troubleshooting

- nvcc template errors referencing libstdc++ (GCC 11):
  - Ensure `g++-10` is installed; the Makefile will use it if present
- PTX version mismatch (e.g., “Unsupported .version 7.6; current is 7.5”):
  - Perform a clean build; ensure helper objects like `helper/pipe/pipe.o` are rebuilt
- Intel Pin download issues:
  - `make` auto‑downloads Pin; verify network access and rerun
- Docker GPU access:
  - Ensure NVIDIA Container Toolkit is installed and `--gpus all` is used

## Repository map (quick reference)

- `Makefile` – top‑level build
- `src/owl_monitor/cpu_trace/` – Intel Pin CPU tracer
- `src/owl_monitor/gpu_trace/` – NVBit GPU tracer and helpers
- `src/owl_monitor/core/` – NVBit headers/libs
- `src/owl_analyzer/` – Rust analyzer (binary output)
- `src/owl-wrapper` – wrapper to run program under both tracers
- `example/cuda-examples/` – minimal CUDA example
- `example/crypt-examples/libgpucrypto/` – crypto workloads
- `build/lib/` – compiled tracer shared objects
- `owl_results/` – analyzer outputs

## Next steps

- Try larger workloads (e.g., AES/RSA in `example/crypt-examples/libgpucrypto`)
- Increase `-t` to improve statistical confidence
- Automate native testing via a small `make test-native` helper target (optional)

If you’d like, we can add a short link from the top-level `README.md` to this guide.
