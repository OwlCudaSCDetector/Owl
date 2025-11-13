# Docker Plotting Dependencies Setup

## Overview

This document describes the improvements made to the Docker environment to support the CDF plotting feature in Owl.

## Changes Made

### 1. Updated Dockerfile

**File**: `/home/alan/Documents/Owl/Dockerfile`

**Added Dependencies**:
- `pkg-config` - Required by the Rust `plotters` crate for system library detection
- `libfontconfig1-dev` - Font rendering library required for text in plots

**Modified Section**:
```dockerfile
RUN apt update && \
        DEBIAN_FRONTEND=noninteractive apt install -y --no-install-recommends \
            build-essential ninja-build python3 python3-pip python3-setuptools \
            curl git g++-multilib wget bc ca-certificates clang \
            pkg-config libfontconfig1-dev && \
        rm -rf /var/lib/apt/lists/* && \
        python3 -m pip install --no-cache-dir pyyaml typing-extensions numpy scipy matplotlib
```

### 2. Rebuilt Docker Image

```bash
docker build -t owl:devel .
```

The image now includes all dependencies needed for building `owl_analyzer` with the `plot` feature.

### 3. Built owl_analyzer with Plot Feature

```bash
docker run --rm --gpus all -v /home/alan/Documents/Owl:/workspace \
  -w /workspace/src/owl_analyzer owl:devel \
  bash -c "cargo build --release --features plot"
```

This builds the analyzer inside Docker with:
- Correct glibc version (matches container)
- Full plotting support enabled
- All dependencies properly linked

## Benefits

### ✅ Consistent Environment
- Same build environment for both development and testing
- No more glibc version mismatches between host and container
- Reproducible builds

### ✅ Full Plotting Support
- Can now build with `--features plot` in Docker
- CDF visualization works in containerized environment
- Same plotting capabilities as native builds

### ✅ Simplified Workflow
- Single Docker image for all operations
- No need to build separately for native vs Docker
- Automated dependency management

## Usage

### Building in Docker (with plotting)

```bash
cd /home/alan/Documents/Owl
docker run --rm --gpus all -v $(pwd):/workspace \
  -w /workspace/src/owl_analyzer owl:devel \
  cargo build --release --features plot
```

### Building in Docker (without plotting)

```bash
cd /home/alan/Documents/Owl
docker run --rm --gpus all -v $(pwd):/workspace \
  -w /workspace/src/owl_analyzer owl:devel \
  cargo build --release
```

### Running Analysis in Docker

```bash
cd /home/alan/Documents/Owl/example/crypt-examples/libgpucrypto
docker run --rm --gpus all \
  -v /home/alan/Documents/Owl:/workspace \
  -w /workspace/example/crypt-examples/libgpucrypto \
  owl:devel \
  /workspace/src/owl_analyzer/target/release/owl_analyzer \
  -c "/workspace/src/owl-wrapper /workspace/example/crypt-examples/libgpucrypto/bin/aes_test -m ENC -l 1024 -f" \
  -r "/workspace/src/owl-wrapper /workspace/example/crypt-examples/libgpucrypto/bin/aes_test -m ENC -l 1024" \
  -t 10
```

## Enabling CDF Plotting

The CDF plotting code is currently **commented out by default** in `analyzer/src/memory.rs`.

To enable plotting in Docker:

1. **Uncomment the plotting code** in `analyzer/src/memory.rs` (lines ~56-88)
2. **Rebuild in Docker** with the plot feature:
   ```bash
   docker run --rm --gpus all -v $(pwd):/workspace \
     -w /workspace/src/owl_analyzer owl:devel \
     cargo build --release --features plot
   ```
3. **Run analysis** - plots will be generated in `owl_cdf_plots/`

## Verification

### Test Results

**Docker Environment** (with plotting dependencies):
- ✅ Builds successfully with `--features plot`
- ✅ No build errors related to fontconfig or pkg-config
- ✅ Binary runs correctly in container
- ✅ Analysis completes successfully
- ✅ No glibc version issues

**Comparison**:
- **Before**: Build failed with "pkg-config command could not be found"
- **After**: Build succeeds with all plotting dependencies

## Troubleshooting

### Issue: "pkg-config command could not be found"
**Solution**: This is now fixed. The Docker image includes `pkg-config`.

### Issue: "Could not find fontconfig"
**Solution**: This is now fixed. The Docker image includes `libfontconfig1-dev`.

### Issue: glibc version mismatch
**Solution**: Build inside Docker to match the container's glibc version.

### Issue: Plots not being generated
**Check**:
1. Is the plotting code uncommented in `memory.rs`?
2. Was the analyzer built with `--features plot`?
3. Are there significant leaks (p < 0.05) to plot?
4. Is the `owl_cdf_plots` directory writable?

## Related Documentation

- `CDF_VISUALIZATION.md` - Comprehensive guide to CDF plotting feature
- `HOW_TO_ENABLE_CDF_PLOTS.md` - Quick reference for enabling plots
- `Dockerfile` - Docker image configuration

## Summary

The Docker environment now fully supports:
- ✅ Building owl_analyzer with plotting feature
- ✅ Running analysis with plot generation capability
- ✅ Consistent build environment across native and Docker
- ✅ All required dependencies pre-installed

The plotting feature remains disabled by default (code commented out) to avoid generating hundreds of plots during routine analysis.
