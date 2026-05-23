use rand::RngExt;

pub fn sample_exponential(rng: &mut impl RngExt, mean: f32) -> f32 {
    let u: f32 = rng.random_range(f32::MIN_POSITIVE..1.0);
    -mean * u.ln()
}

pub fn hyperbolic_scale(base: f32, count: u32, alpha: f32) -> f32 {
    base / (1.0 + alpha * count as f32)
}
