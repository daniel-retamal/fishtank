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

pub fn events_in(rng: &mut impl RngExt, per_sec: f32, dt: f32) -> u32 {
    let expected = (per_sec * dt).max(0.0);
    let whole = expected.floor();
    whole as u32 + u32::from(rng.random::<f32>() < expected - whole)
}

const BEAT_TOLERANCE: f32 = 1e-4;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Metronome {
    elapsed: f32,
}

impl Metronome {
    pub fn beats(&mut self, dt: f32, interval: f32) -> u32 {
        self.elapsed += dt;
        let beats = (self.elapsed / interval + BEAT_TOLERANCE).floor();
        if beats < 1.0 {
            return 0;
        }
        self.elapsed = (self.elapsed - beats * interval).max(0.0);
        beats as u32
    }
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

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::SmallRng};

    use super::*;

    const DEFAULT_FRAME: f32 = 1.0 / 30.0;
    const FAST_FRAME: f32 = 1.0 / 120.0;

    #[test]
    fn a_metronome_counts_whole_intervals_and_keeps_the_rest() {
        let mut clock = Metronome::default();
        assert_eq!(clock.beats(0.25, 1.0), 0);
        assert_eq!(clock.beats(0.25, 1.0), 0);
        assert_eq!(clock.beats(0.75, 1.0), 1);
        assert_eq!(clock.beats(2.0, 1.0), 2);
        assert_eq!(clock.beats(0.25, 1.0), 0);
        assert_eq!(clock.beats(0.5, 1.0), 1);
    }

    #[test]
    fn a_metronome_never_loses_a_beat_to_rounding() {
        let mut clock = Metronome::default();
        assert_eq!(clock.beats(1.0, DEFAULT_FRAME), 30);
    }

    #[test]
    fn a_metronome_at_the_default_frame_rate_beats_once_a_frame() {
        let mut clock = Metronome::default();
        for _ in 0..90 {
            assert_eq!(clock.beats(DEFAULT_FRAME, DEFAULT_FRAME), 1);
        }
    }

    #[test]
    fn a_metronome_beats_as_often_per_second_whatever_the_frame() {
        let mut slow = Metronome::default();
        let mut fast = Metronome::default();
        let slow_beats: u32 = (0..30).map(|_| slow.beats(DEFAULT_FRAME, 0.1)).sum();
        let fast_beats: u32 = (0..120).map(|_| fast.beats(FAST_FRAME, 0.1)).sum();
        assert!(slow_beats.abs_diff(10) <= 1, "{slow_beats}");
        assert!(fast_beats.abs_diff(10) <= 1, "{fast_beats}");
    }

    #[test]
    fn events_in_a_second_average_their_rate_at_any_frame() {
        let mut rng = SmallRng::seed_from_u64(7);
        let rate = 13.5;
        let seconds = 400;
        for frame in [1.0, DEFAULT_FRAME, FAST_FRAME] {
            let frames = (seconds as f32 / frame).round() as usize;
            let total: u32 = (0..frames).map(|_| events_in(&mut rng, rate, frame)).sum();
            let mean = total as f32 / seconds as f32;
            assert!(
                (mean - rate).abs() < 0.5,
                "{mean} events a second at {frame}"
            );
        }
    }

    #[test]
    fn no_time_is_no_event() {
        let mut rng = SmallRng::seed_from_u64(1);
        assert_eq!(events_in(&mut rng, 13.5, 0.0), 0);
    }
}
