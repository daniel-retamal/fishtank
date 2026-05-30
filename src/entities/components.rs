use std::f32::consts::TAU;

use rand::RngExt;

#[derive(Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Clone)]
pub struct SwayState {
    pub phase: f32,
}

pub fn tick_sway(sway: &mut SwayState, sway_speed: f32) {
    sway.phase = (sway.phase + sway_speed).rem_euclid(TAU);
}

#[derive(Clone)]
pub struct BlinkTimer {
    pub is_open: bool,
    timer: f32,
    open_secs: f32,
    closed_secs: f32,
}

impl BlinkTimer {
    pub fn new(
        rng: &mut impl RngExt,
        open_min: f32,
        open_max: f32,
        closed_min: f32,
        closed_max: f32,
    ) -> Self {
        let open_secs = rng.random_range(open_min..open_max);
        let closed_secs = rng.random_range(closed_min..closed_max);
        let timer = rng.random_range(0.0_f32..open_secs);
        Self {
            is_open: true,
            timer,
            open_secs,
            closed_secs,
        }
    }

    pub fn fish_eye(rng: &mut impl RngExt) -> Self {
        Self::new(rng, 0.8, 1.8, 0.08, 0.35)
    }

    pub fn entity_eye(rng: &mut impl RngExt) -> Self {
        Self::new(rng, 0.5, 1.0, 0.1, 0.30)
    }

    pub fn wing(rng: &mut impl RngExt) -> Self {
        Self::new(rng, 1.5, 2.0, 0.15, 0.4)
    }

    pub fn tick(&mut self, dt: f32) {
        self.timer += dt;
        if self.is_open {
            if self.timer >= self.open_secs {
                self.timer -= self.open_secs;
                self.is_open = false;
            }
        } else if self.timer >= self.closed_secs {
            self.timer -= self.closed_secs;
            self.is_open = true;
        }
    }
}

#[derive(Clone)]
pub struct EyeRow {
    pub height: usize,
    pub blink: BlinkTimer,
}

impl EyeRow {
    pub fn new(height: usize, rng: &mut impl RngExt) -> Self {
        Self {
            height,
            blink: BlinkTimer::entity_eye(rng),
        }
    }
}

pub fn sway_x_offset(
    sway_phase: f32,
    row: usize,
    total_height: usize,
    wave_spread: f32,
    sway_amount: f32,
) -> i32 {
    let ratio = if total_height <= 1 {
        1.0_f32
    } else {
        row as f32 / (total_height - 1) as f32
    };
    let phase = sway_phase + row as f32 * wave_spread;
    (phase.sin() * sway_amount * ratio).round() as i32
}

pub fn extend_spaced<T, R: RngExt>(
    items: &mut Vec<T>,
    to_width: i32,
    extra_width: i32,
    spacing: std::ops::RangeInclusive<i32>,
    rng: &mut R,
    x_of: impl Fn(&T) -> i32,
    mut make: impl FnMut(i32, &mut R) -> T,
) {
    let mut next_x = if items.is_empty() {
        rng.random_range(spacing.clone())
    } else {
        x_of(items.last().unwrap()) + extra_width + rng.random_range(spacing.clone())
    };
    while next_x < to_width {
        items.push(make(next_x, rng));
        next_x += extra_width + rng.random_range(spacing.clone());
    }
}
