# How to Enable CDF Plot Generation

By default, Owl does **NOT** automatically generate CDF plots to avoid creating hundreds of plot files during every analysis. Follow these steps when you want to visualize the CDFs.

## Quick Steps

### 1. Edit the Code

Open `src/owl_analyzer/analyzer/src/memory.rs` and find the `test()` function (around lines 55-90).

Look for this section:

```rust
let mut minimum = 100.0;
// Commented out automatic CDF plotting - uncomment to enable visualization
// let mut plot_idx = 0;
```

And further down:

```rust
// Commented out automatic CDF plotting
// Uncomment this section to generate CDF plots for significant leaks
/*
if p < 0.05 && plot_idx < 5 {
    let plot_dir = "owl_cdf_plots";
    // ... plotting code ...
}
*/
```

### 2. Uncomment the Code

Change it to:

```rust
let mut minimum = 100.0;
// Automatic CDF plotting enabled
let mut plot_idx = 0;
```

And remove the `/*` and `*/` around the plotting block:

```rust
// Generate CDF plots for significant leaks
if p < 0.05 && plot_idx < 5 {
    let plot_dir = "owl_cdf_plots";
    std::fs::create_dir_all(plot_dir).ok();
    
    let plot_path = format!(
        "{}/instr_0x{:x}_operand_{}_p{:.6}.png",
        plot_dir, self.instr, idx, p
    );
    
    log::info!("Generating CDF plot for instruction 0x{:x} operand {} (p={:.6})", 
              self.instr, idx, p);
    
    if let Err(e) = crate::plot::plot_cdf_comparison(
        l,
        r,
        &format!("0x{:x} operand {} (p={:.6})", self.instr, idx, p),
        &plot_path,
    ) {
        log::warn!("Failed to plot CDF: {}", e);
    } else {
        plot_idx += 1;
    }
}
```

### 3. Rebuild

```bash
cd src/owl_analyzer
cargo build --release --features plot
```

### 4. Run Analysis

```bash
cd example/crypt-examples/libgpucrypto
/path/to/owl_analyzer -c "command-fixed" -r "command-random" -t 10
```

### 5. View Plots

Plots will be in `owl_cdf_plots/`:

```bash
ls owl_cdf_plots/
```

Each plot is named: `instr_0x{addr}_operand_{n}_p{pvalue}.png`

## Customization Options

### Change Number of Plots Generated

By default, the code generates all plots for leaks with `p < 0.05`. You can modify this:

**Limit to first 5 plots:**
```rust
if p < 0.05 && plot_idx < 5 {  // Only first 5 leaks
```

**Generate all significant leaks:**
```rust
if p < 0.05 {  // Remove plot_idx check
```

**Only highly significant leaks:**
```rust
if p < 0.001 {  // Only p-value < 0.001
```

### Change Output Directory

Modify the `plot_dir` variable:

```rust
let plot_dir = "my_custom_plots";  // Instead of "owl_cdf_plots"
```

### Change Plot Filename Format

Modify the `plot_path` format string:

```rust
let plot_path = format!(
    "{}/leak_{:04}_p{:.6}.png",  // Sequential numbering
    plot_dir, plot_idx, p
);
```

## Disabling Plots Again

When you're done visualizing, simply:

1. Re-comment the plotting code (add back the `/*` and `*/`)
2. Comment out `let mut plot_idx = 0;`
3. Rebuild: `cargo build --release --features plot`

Or just remove the `--features plot` flag when building to disable all plotting infrastructure:

```bash
cargo build --release  # No plotting support
```

## Clean Up Generated Plots

To remove all generated plots:

```bash
rm -rf owl_cdf_plots/
```

## More Information

See `CDF_VISUALIZATION.md` for:
- Detailed explanation of how CDF plots work
- How to interpret the plots
- Statistical background on the KS test
- Example analysis results
