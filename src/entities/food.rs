use std::f32::consts::TAU;

use rand::RngExt;

use super::components::{Position, SwayState, tick_sway};
use crate::settings::Settings;

const FALL_SPEED_MIN: f32 = 2.5;
const FALL_SPEED_MAX: f32 = 5.5;
const SWAY_SPEED: f32 = 0.06;
const SWAY_AMOUNT: f32 = 3.0;
pub const DEFAULT_COUNT: usize = 16;

const FOOD_CHARS: [char; 4] = ['·', '•', '◦', '.'];

pub struct Food {
    pub position: Position,
    pub fall_speed: f32,
    pub sway: SwayState,
    pub food_char: char,
    base_x: f32,
    pub settled: bool,
    pub eaten: bool,
}

impl Food {
    pub fn new(x: f32) -> Self {
        let mut rng = rand::rng();
        Self {
            position: Position { x, y: 0.0 },
            fall_speed: rng.random_range(FALL_SPEED_MIN..FALL_SPEED_MAX),
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            food_char: FOOD_CHARS[rng.random_range(0..FOOD_CHARS.len())],
            base_x: x,
            settled: false,
            eaten: false,
        }
    }

    pub fn tick(&mut self, settings: &Settings, tank_width: u16, tank_height: u16) {
        if self.settled {
            return;
        }
        let dt = 1.0 / settings.fps;
        self.position.y += self.fall_speed * dt;

        tick_sway(&mut self.sway, SWAY_SPEED);
        self.position.x = (self.base_x + SWAY_AMOUNT * self.sway.phase.sin())
            .clamp(0.0, (tank_width as f32 - 1.0).max(0.0));

        let max_y = (tank_height as f32 - 1.0).max(0.0);
        if self.position.y >= max_y {
            self.position.y = max_y;
            self.settled = true;
        }
    }
}
