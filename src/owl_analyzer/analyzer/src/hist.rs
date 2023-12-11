use std::f64::consts::E;

pub fn ks_test_p_value(x: f64, m: usize, n: usize) -> f64 {
    let m = m as f64;
    let n = n as f64;
    2f64 * E.powf(-2f64 * x.powi(2) * ((n * m) / (n + m)))
}
