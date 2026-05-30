use std::f32::consts::TAU;

use rand::RngExt;
use ratatui::style::Color;

use super::components::{SwayState, sway_x_offset, tick_sway};
use crate::colors::{GREEN, GREEN_DARK, LIGHT_GREEN};

const SWAY_SPEED: f32 = 0.04;
const WAVE_SPREAD: f32 = 0.5;
pub const SWAY_AMOUNT: f32 = 2.0;

pub trait Seaweed {
    fn x(&self) -> i32;
    fn height(&self) -> usize;
    fn segment_at(&self, row: usize) -> (i32, char);
    fn color_at(&self, row: usize) -> Color;
    fn tick(&mut self, delta_time: f32);
}

pub struct Plant {
    pub x: i32,
    pub height: usize,
    pub sway: SwayState,
    pub color: Color,
}

impl Plant {
    pub fn new(x: i32, height: usize) -> Self {
        let mut rng = rand::rng();
        let colors = [GREEN, LIGHT_GREEN, GREEN_DARK];
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
}

impl Seaweed for Plant {
    fn x(&self) -> i32 {
        self.x
    }

    fn height(&self) -> usize {
        self.height
    }

    fn segment_at(&self, row: usize) -> (i32, char) {
        let x_offset = sway_x_offset(self.sway.phase, row, self.height, WAVE_SPREAD, SWAY_AMOUNT);
        let ch = if x_offset > 0 {
            ')'
        } else if x_offset < 0 {
            '('
        } else {
            '|'
        };
        (x_offset, ch)
    }

    fn color_at(&self, _row: usize) -> Color {
        self.color
    }

    fn tick(&mut self, _delta_time: f32) {
        tick_sway(&mut self.sway, SWAY_SPEED);
    }
}
