use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{GRAY, WHITE};

pub const STAR_FIELD_RATIO: f32 = 0.06;
pub const STAR_ZERO_FRACTION: f32 = 0.6;

const TWINKLE_LIT_MIN: f32 = 0.05;
const TWINKLE_LIT_MAX: f32 = 0.25;
const TWINKLE_DARK_MIN: f32 = 3.0;
const TWINKLE_DARK_MAX: f32 = 10.0;
const STAR_CHARS: &[char] = &['.', '*', '\'', ':', '⋆', '⟡', '✶', '˚'];
const STAR_LIT_COLOR: Color = WHITE;
const STAR_DARK_COLOR: Color = GRAY;

#[derive(Clone)]
pub struct StarTwinkle {
    pub ch: char,
    pub lit: bool,
    timer: f32,
}

impl StarTwinkle {
    pub fn new(rng: &mut impl RngExt) -> Self {
        Self {
            ch: STAR_CHARS[rng.random_range(0..STAR_CHARS.len())],
            lit: false,
            timer: rng.random_range(TWINKLE_DARK_MIN..TWINKLE_DARK_MAX),
        }
    }

    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt) {
        self.timer -= dt;
        if self.timer > 0.0 {
            return;
        }
        self.lit = !self.lit;
        self.timer = if self.lit {
            rng.random_range(TWINKLE_LIT_MIN..TWINKLE_LIT_MAX)
        } else {
            rng.random_range(TWINKLE_DARK_MIN..TWINKLE_DARK_MAX)
        };
    }

    pub fn color(&self) -> Color {
        if self.lit {
            STAR_LIT_COLOR
        } else {
            STAR_DARK_COLOR
        }
    }
}

pub fn star_field_target(w: u16, h: u16) -> usize {
    if w == 0 || h == 0 {
        return 0;
    }
    let zero_row = (h as f32 * STAR_ZERO_FRACTION) as usize;
    if zero_row == 0 {
        return 0;
    }
    ((w as f32 * zero_row as f32 * STAR_FIELD_RATIO / 2.0) as usize).max(1)
}
