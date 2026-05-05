use std::f32::consts::TAU;

use rand::RngExt;
use ratatui::style::Color;

use super::components::{SwayState, tick_sway};

const SWAY_SPEED: f32 = 0.04;
const WAVE_SPREAD: f32 = 0.5;
pub const SWAY_AMOUNT: f32 = 2.0;

pub struct Plant {
    pub x: i32,
    pub height: usize,
    pub sway: SwayState,
    pub color: Color,
}

impl Plant {
    pub fn new(x: i32, height: usize) -> Self {
        let mut rng = rand::rng();
        let colors = [Color::Green, Color::LightGreen, Color::Rgb(0, 100, 0)];
        let color = colors[rng.random_range(0..colors.len())];
        Self {
            x,
            height,
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            color,
        }
    }

    pub fn segment_at(&self, h: usize) -> (i32, char) {
        let ratio = if self.height <= 1 {
            1.0_f32
        } else {
            h as f32 / (self.height - 1) as f32
        };
        let phase = self.sway.phase + h as f32 * WAVE_SPREAD;
        let x_offset = (phase.sin() * SWAY_AMOUNT * ratio).round() as i32;
        let ch = if x_offset > 0 {
            ')'
        } else if x_offset < 0 {
            '('
        } else {
            '|'
        };
        (x_offset, ch)
    }

    pub fn tick(&mut self) {
        tick_sway(&mut self.sway, SWAY_SPEED);
    }
}
