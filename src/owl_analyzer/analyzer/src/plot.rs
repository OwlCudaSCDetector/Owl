use std::collections::BTreeMap;

#[cfg(feature = "plot")]
use plotters::prelude::*;

use crate::memory::TargetAddr;

pub type MemAccess = BTreeMap<TargetAddr, usize>;

#[cfg(feature = "plot")]
pub fn plot_cdf_comparison(
    fixed: &MemAccess,
    random: &MemAccess,
    instruction_id: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Calculate weights for normalization
    let l_total: usize = fixed.iter()
        .filter(|(v, _)| v.is_valid())
        .map(|(_, c)| *c)
        .sum();
    
    let r_total: usize = random.iter()
        .filter(|(v, _)| v.is_valid())
        .map(|(_, c)| *c)
        .sum();
    
    if l_total == 0 || r_total == 0 {
        log::warn!("Empty distribution, skipping plot");
        return Ok(());
    }
    
    let l_weight = 1.0 / l_total as f64;
    let r_weight = 1.0 / r_total as f64;
    
    // Build CDF data points
    let mut fixed_points = Vec::new();
    let mut random_points = Vec::new();
    let mut max_diff_point = None;
    
    let mut l_cdf = 0.0;
    let mut r_cdf = 0.0;
    let mut max_diff = 0.0;
    
    let mut l_iter = fixed.iter().filter(|(v, _)| v.is_valid());
    let mut r_iter = random.iter().filter(|(v, _)| v.is_valid());
    
    let mut l_cur = l_iter.next();
    let mut r_cur = r_iter.next();
    
    let mut x_pos = 0usize;
    
    loop {
        match (l_cur, r_cur) {
            (None, None) => break,
            
            (None, Some((r_addr, r_num))) => {
                r_cur = r_iter.next();
                r_cdf += *r_num as f64 * r_weight;
                
                fixed_points.push((x_pos, l_cdf));
                random_points.push((x_pos, r_cdf));
                x_pos += 1;
            }
            
            (Some((l_addr, l_num)), None) => {
                l_cur = l_iter.next();
                l_cdf += *l_num as f64 * l_weight;
                
                fixed_points.push((x_pos, l_cdf));
                random_points.push((x_pos, r_cdf));
                x_pos += 1;
            }
            
            (Some((l_addr, l_num)), Some((r_addr, r_num))) => {
                if l_addr < r_addr {
                    l_cur = l_iter.next();
                    l_cdf += *l_num as f64 * l_weight;
                } else if l_addr > r_addr {
                    r_cur = r_iter.next();
                    r_cdf += *r_num as f64 * r_weight;
                } else {
                    l_cur = l_iter.next();
                    l_cdf += *l_num as f64 * l_weight;
                    r_cur = r_iter.next();
                    r_cdf += *r_num as f64 * r_weight;
                }
                
                fixed_points.push((x_pos, l_cdf));
                random_points.push((x_pos, r_cdf));
                x_pos += 1;
            }
        }
        
        // Track maximum difference
        let diff = (l_cdf - r_cdf).abs();
        if diff > max_diff {
            max_diff = diff;
            max_diff_point = Some((x_pos - 1, l_cdf, r_cdf));
        }
    }
    
    // Create the plot
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("CDF Comparison - Instruction {}", instruction_id),
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0usize..x_pos.max(1), 0f64..1.1f64)?;
    
    chart
        .configure_mesh()
        .x_desc("Memory Address (sorted index)")
        .y_desc("Cumulative Probability")
        .draw()?;
    
    // Draw fixed-key CDF
    chart
        .draw_series(LineSeries::new(
            fixed_points.iter().map(|(x, y)| (*x, *y)),
            &BLUE,
        ))?
        .label("Fixed Key")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
    // Draw random-key CDF
    chart
        .draw_series(LineSeries::new(
            random_points.iter().map(|(x, y)| (*x, *y)),
            &RED,
        ))?
        .label("Random Keys")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    // Draw maximum difference line
    if let Some((x, y1, y2)) = max_diff_point {
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(x, y1), (x, y2)],
            &GREEN.mix(0.5),
        )))?;
        
        // Add annotation for max difference
        chart.draw_series(std::iter::once(Text::new(
            format!("Max diff: {:.4}", max_diff),
            (x, (y1 + y2) / 2.0),
            ("sans-serif", 15).into_font().color(&GREEN),
        )))?;
    }
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    
    root.present()?;
    
    log::info!("CDF plot saved to: {}", output_path);
    log::info!("  Fixed total accesses: {}", l_total);
    log::info!("  Random total accesses: {}", r_total);
    log::info!("  Maximum CDF difference: {:.4}", max_diff);
    
    Ok(())
}

#[cfg(not(feature = "plot"))]
pub fn plot_cdf_comparison(
    _fixed: &MemAccess,
    _random: &MemAccess,
    _instruction_id: &str,
    _output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    log::warn!("Plotting feature not enabled. Rebuild with --features plot");
    Ok(())
}
