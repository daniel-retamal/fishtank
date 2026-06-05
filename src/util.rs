use rand::RngExt;

pub fn sample_exponential(rng: &mut impl RngExt, mean: f32) -> f32 {
    let u: f32 = rng.random_range(f32::MIN_POSITIVE..1.0);
    -mean * u.ln()
}

pub fn hyperbolic_scale(base: f32, count: u32, alpha: f32) -> f32 {
    base / (1.0 + alpha * count as f32)
}

pub fn exponential_event(rng: &mut impl RngExt, mean: f32, dt: f32) -> bool {
    if mean <= 0.0 {
        return true;
    }
    rng.random::<f32>() < 1.0 - (-dt / mean).exp()
}

pub fn even_indices(avail: usize, count: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(count);
    let mut last: Option<usize> = None;
    for j in 0..count {
        let mut idx = (((2 * j + 1) * avail) / (2 * count)).min(avail - 1);
        if let Some(prev) = last
            && idx <= prev
        {
            idx = prev + 1;
        }
        out.push(idx);
        last = Some(idx);
    }
    out
}
