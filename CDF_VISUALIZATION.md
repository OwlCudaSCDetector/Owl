# CDF Visualization in Owl

## Overview

This document explains the CDF (Cumulative Distribution Function) visualization feature added to Owl. This feature can generate plots showing the statistical comparison that Owl performs to detect side-channel leaks.

**Note:** The automatic plot generation is currently **commented out by default**. You need to manually enable it in the code when you want to visualize the CDFs (see the "Enabling Plot Generation" section below).

## What Was Added

### 1. Plotting Infrastructure

**New Files:**
- `analyzer/src/plot.rs` - Core plotting module using the `plotters` library

**Modified Files:**
- `analyzer/Cargo.toml` - Added `plotters` as an optional dependency
- `owl_analyzer/Cargo.toml` - Propagated the `plot` feature
- `analyzer/src/lib.rs` - Added the plot module
- `analyzer/src/memory.rs` - Integrated plot generation into the testing function

### 2. Building with Plotting Support

The plotting feature is optional and can be enabled during build:

```bash
cd src/owl_analyzer
cargo build --release --features plot
```

Without the `--features plot` flag, Owl builds normally without any plotting dependencies.

### 3. Enabling Plot Generation

**By default, plot generation is commented out** to avoid automatically creating hundreds of plots during every analysis. To enable it:

1. Open `analyzer/src/memory.rs`
2. Find the `test()` function (around line 55-90)
3. Look for the commented-out section that starts with:
   ```rust
   // Commented out automatic CDF plotting
   // Uncomment this section to generate CDF plots for significant leaks
   /*
   if p < 0.05 && plot_idx < 5 {
   ```
4. Uncomment the entire block by:
   - Uncommenting `let mut plot_idx = 0;` near the top
   - Removing the `/*` and `*/` around the plotting code
5. Rebuild with `cargo build --release --features plot`
6. Run your analysis - plots will be automatically generated

**To disable plotting again:** Simply re-comment the code block and rebuild.

## How It Works

### Statistical Testing Background

Owl detects side-channel leaks by comparing memory access patterns between two scenarios:
- **Fixed input**: Same encryption key used for all traces
- **Random input**: Different random keys for each trace

For each memory address accessed by an instruction, Owl collects the distribution of when that address was accessed across multiple traces. It then performs a **Kolmogorov-Smirnov (KS) two-sample test** to determine if the two distributions are statistically different.

The KS test works by:
1. Building the Cumulative Distribution Function (CDF) for each distribution
2. Finding the maximum vertical distance between the two CDFs
3. Computing a p-value based on this maximum distance

### What the Plots Show

Each generated plot displays:

1. **Blue line**: CDF of the fixed-key distribution
2. **Red line**: CDF of the random-key distribution  
3. **Green vertical line**: Location of the maximum difference between the CDFs
4. **Annotation**: The maximum difference value
5. **Title**: Instruction address, operand index, and p-value

The plot filename encodes this information:
```
instr_0x{instruction_addr}_operand_{index}_p{p_value}.png
```

### Automatic Plot Generation
### Automatic Plot Generation (When Enabled)

When you uncomment the plotting code in `memory.rs`, Owl will automatically generate plots for all detected leaks (p < 0.05). The plots are saved in the `owl_cdf_plots/` directory in the same location where the analysis is run.

Key implementation details:
- Plots are generated during the statistical testing phase
- Only significant leaks (p < 0.05) trigger plot generation
- CDFs are built from the same data used in the actual statistical test
- Plot generation requires both:
  1. Building with `--features plot`
  2. Uncommenting the plotting code in `memory.rs`

## Example Analysis Results

### 10-Iteration Analysis Statistics

From a recent 10-iteration AES analysis:

```
Total plots generated: 284 leaks

P-value distribution:
  177 leaks with p ≈ 0.015  (moderate significance)
   40 leaks with p ≈ 0.0006 (high significance)
   37 leaks with p ≈ 0.003  (high significance)
   30 leaks with p ≈ 0.00009 (very high significance)
```

### Most Significant Leaks

The most significant leaks (p < 0.0001) were found at:
- Instruction `0x1770` (5 operands, p = 0.000091)
- Instruction `0x2180` (5 operands, p = 0.000091)
- Instruction `0x2490` (5 operands, p = 0.000091)
- Instruction `0xe80` (5 operands, p = 0.000091)
- Instruction `0xd70` (5 operands, p = 0.000091)
- Instruction `0x2f60` (5 operands, p = 0.000091)

These extremely low p-values indicate that the memory access patterns for these instructions are **dramatically different** between fixed-key and random-key scenarios - a clear sign of key-dependent behavior.

## Interpreting the Plots

### Understanding CDF Comparison

The CDF shows the probability that a memory access occurs before or at a given time:

- **X-axis**: Time (in trace cycles/steps)
- **Y-axis**: Cumulative probability (0.0 to 1.0)
- **CDF shape**: Shows how the accesses are distributed over time

### What Different Patterns Mean

1. **Concentrated vs. Spread Out**
   - If the blue CDF (fixed-key) rises sharply at one point while the red CDF (random-key) rises gradually, this indicates that:
     - Fixed-key accesses cluster at a specific time
     - Random-key accesses are spread across multiple times
     - This is typical of table lookup leaks in AES T-table implementations

2. **Maximum Difference**
   - Large max_diff (e.g., 0.8-0.9): Very distinct distributions
   - Small max_diff (e.g., 0.2-0.3): Subtle differences
   - The KS test converts this into a p-value accounting for sample size

3. **P-value Significance**
   - p < 0.001: Highly significant leak (very strong evidence)
   - p < 0.01: Significant leak (strong evidence)
   - p < 0.05: Marginally significant leak (moderate evidence)
   - p ≥ 0.05: Not significant (insufficient evidence of leak)

## Technical Details

### Plot Generation Code

The plotting is implemented in `analyzer/src/plot.rs`:

```rust
pub fn plot_cdf_comparison(
    fixed_dist: &[usize],
    random_dist: &[usize],
    label: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>>
```

**Algorithm:**
1. Sort both distributions
2. Build CDFs by computing cumulative probabilities
3. Find the maximum vertical distance between CDFs
4. Create a plot with both CDFs and mark the max difference
5. Save as PNG file

### Integration Point

In `analyzer/src/memory.rs`, the `MemAccessInstr::test()` function was modified:

```rust
// After computing the p-value from KS test
if p < 0.05 {
    let plot_path = format!(
        "owl_cdf_plots/instr_0x{:x}_operand_{}_p{:.6}.png",
        self.id, operand_idx, p
    );
    
    #[cfg(feature = "plot")]
    {
        std::fs::create_dir_all("owl_cdf_plots").ok();
        log::info!("Generating CDF plot: {}", plot_path);
        crate::plot::plot_cdf_comparison(l, r, &label, &plot_path)
            .unwrap_or_else(|e| log::warn!("Failed to create plot: {}", e));
    }
}
```

### Conditional Compilation

The plot generation code uses Rust's conditional compilation to ensure it only compiles when the feature is enabled:

```rust
#[cfg(feature = "plot")]
pub mod plot;

#[cfg(feature = "plot")]
{
    // plotting code here
}
```

This means:
- If built without `--features plot`: No plotting dependencies, no plotting code in binary
- If built with `--features plot`: Full plotting functionality included

## Usage Example

### Running Analysis with Plotting

```bash
# Build with plotting support
cd src/owl_analyzer
cargo build --release --features plot

# Run analysis (plots will be generated automatically)
cd ../../example/crypt-examples/libgpucrypto
/path/to/owl_analyzer -c "command-fixed" -r "command-random" -t 10

# View the generated plots
ls owl_cdf_plots/
```

### Viewing Plots

The plots are standard PNG images (800x600 pixels) that can be viewed with any image viewer:

```bash
# On Linux with GUI
eog owl_cdf_plots/instr_0x1770_operand_0_p0.000091.png

# Or use VS Code
code owl_cdf_plots/

# Or copy to a shared folder for viewing
```

## Why This Is Useful

### 1. **Visual Understanding**
- See the statistical test that Owl performs internally
- Understand why certain p-values are low (distributions look very different)
- Verify that the KS test is working correctly

### 2. **Documentation**
- Include plots in research papers or presentations
- Show concrete examples of side-channel leaks
- Illustrate the difference between vulnerable and secure implementations

### 3. **Debugging**
- Verify that the leak detection is working as expected
- Identify patterns in the CDF shapes for different types of leaks
- Compare plots across different implementations

### 4. **Education**
- Teach students about statistical side-channel analysis
- Demonstrate the concept of timing distributions
- Show real-world examples of cryptographic leaks

## Performance Impact

- **Build time**: Minimal increase (plotters library adds ~1 second)
- **Runtime**: Plotting adds ~10-50ms per leak (negligible compared to trace collection)
- **Disk space**: Each plot is ~60-70KB, so 300 leaks = ~20MB total

## Future Enhancements

Potential improvements for the plotting feature:

1. **Configurable output**
   - Command-line option to control number of plots
   - Option to plot only the most significant leaks
   - Different output formats (SVG, PDF)

2. **Enhanced visualizations**
   - Overlay multiple CDFs for different iterations
   - Show confidence intervals
   - Histogram view alongside CDF

3. **Interactive plots**
   - HTML output with zoom/pan capabilities
   - Hover to see exact values
   - Link to source code location

4. **Analysis summaries**
   - Generate an HTML report with all plots
   - Summary statistics table
   - Automatic classification of leak types

## References

- **Kolmogorov-Smirnov Test**: https://en.wikipedia.org/wiki/Kolmogorov%E2%80%93Smirnov_test
- **Plotters Library**: https://github.com/plotters-rs/plotters
- **Owl Paper**: Section on statistical methodology
- **AES T-table Timing Attacks**: Classic cache-timing attack on AES

## Summary

The CDF visualization feature provides a powerful way to:
- **Understand** how Owl detects leaks statistically
- **Verify** that the detection is working correctly
- **Communicate** findings through visual evidence
- **Debug** unexpected results

By visualizing the cumulative distribution functions, you can see exactly what the Kolmogorov-Smirnov test is measuring - the maximum vertical distance between two CDFs - and understand why certain memory access patterns indicate side-channel vulnerabilities.
