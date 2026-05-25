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

pub fn extend_spaced<T, R: RngExt>(
    items: &mut Vec<T>,
    to_width: i32,
    extra_width: i32,
    spacing_min: i32,
    spacing_max: i32,
    rng: &mut R,
    x_of: impl Fn(&T) -> i32,
    mut make: impl FnMut(i32, &mut R) -> T,
) {
    let mut next_x = if items.is_empty() {
        rng.random_range(spacing_min..=spacing_max)
    } else {
        x_of(items.last().unwrap()) + extra_width + rng.random_range(spacing_min..=spacing_max)
    };
    while next_x < to_width {
        items.push(make(next_x, rng));
        next_x += extra_width + rng.random_range(spacing_min..=spacing_max);
    }
}
