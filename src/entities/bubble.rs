use std::f32::consts::TAU;

use rand::RngExt;
use ratatui::style::Color;

use super::components::{Position, SwayState, tick_sway};
use crate::settings::Settings;

const RISE_SPEED_MIN: f32 = 2.0;
const RISE_SPEED_MAX: f32 = 5.0;
const SURFACE_RISE_SPEED_MIN: f32 = 0.5;
const SURFACE_RISE_SPEED_MAX: f32 = 1.5;
const SWAY_SPEED: f32 = 0.02;
const SWAY_AMOUNT: f32 = 0.4;
const HOVER_TIME_MIN: f32 = 0.5;
const HOVER_TIME_MAX: f32 = 1.5;
const SURFACE_LIFETIME_MIN: f32 = 3.0;
const SURFACE_LIFETIME_MAX: f32 = 8.0;

const BUBBLE_CHARS: [char; 3] = ['°', '◦', '○'];

#[derive(Copy, Clone)]
enum BubblePhase {
    Rising { hover_time: f32 },
    Hovering { remaining: f32 },
    Surface { remaining: f32 },
}

pub struct Bubble {
    pub position: Position,
    pub bubble_char: char,
    pub color: Color,
    pub dead: bool,
    pub cash_value: Option<u32>,
    sway: SwayState,
    base_x: f32,
    rise_speed: f32,
    phase: BubblePhase,
}

impl Bubble {
    pub fn new_rising(x: f32, y: f32) -> Self {
        let mut rng = rand::rng();
        Self {
            position: Position { x, y },
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            base_x: x,
            rise_speed: rng.random_range(RISE_SPEED_MIN..RISE_SPEED_MAX),
            bubble_char: BUBBLE_CHARS[rng.random_range(0..BUBBLE_CHARS.len())],
            color: Color::Cyan,
            dead: false,
            cash_value: None,
            phase: BubblePhase::Rising {
                hover_time: rng.random_range(HOVER_TIME_MIN..HOVER_TIME_MAX),
            },
        }
    }

    pub fn new_rising_custom(x: f32, y: f32, ch: char, color: Color) -> Self {
        let mut rng = rand::rng();
        Self {
            position: Position { x, y },
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            base_x: x,
            rise_speed: rng.random_range(RISE_SPEED_MIN..RISE_SPEED_MAX),
            bubble_char: ch,
            color,
            dead: false,
            cash_value: None,
            phase: BubblePhase::Rising {
                hover_time: rng.random_range(HOVER_TIME_MIN..HOVER_TIME_MAX),
            },
        }
    }

    pub fn new_rising_colored(x: f32, y: f32, color: Color) -> Self {
        let mut rng = rand::rng();
        Self {
            position: Position { x, y },
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            base_x: x,
            rise_speed: rng.random_range(RISE_SPEED_MIN..RISE_SPEED_MAX),
            bubble_char: BUBBLE_CHARS[rng.random_range(0..BUBBLE_CHARS.len())],
            color,
            dead: false,
            cash_value: None,
            phase: BubblePhase::Rising {
                hover_time: rng.random_range(HOVER_TIME_MIN..HOVER_TIME_MAX),
            },
        }
    }

    pub fn new_surface_colored(x: f32, y: f32, color: Color) -> Self {
        let mut rng = rand::rng();
        Self {
            position: Position { x, y },
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            base_x: x,
            rise_speed: rng.random_range(SURFACE_RISE_SPEED_MIN..SURFACE_RISE_SPEED_MAX),
            bubble_char: BUBBLE_CHARS[rng.random_range(0..BUBBLE_CHARS.len())],
            color,
            dead: false,
            cash_value: None,
            phase: BubblePhase::Surface {
                remaining: rng.random_range(SURFACE_LIFETIME_MIN..SURFACE_LIFETIME_MAX),
            },
        }
    }

    pub fn new_surface(x: f32, y: f32) -> Self {
        let mut rng = rand::rng();
        Self {
            position: Position { x, y },
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            base_x: x,
            rise_speed: rng.random_range(SURFACE_RISE_SPEED_MIN..SURFACE_RISE_SPEED_MAX),
            bubble_char: BUBBLE_CHARS[rng.random_range(0..BUBBLE_CHARS.len())],
            color: Color::Cyan,
            dead: false,
            cash_value: None,
            phase: BubblePhase::Surface {
                remaining: rng.random_range(SURFACE_LIFETIME_MIN..SURFACE_LIFETIME_MAX),
            },
        }
    }

    pub fn tick(&mut self, settings: &Settings, tank_width: u16) -> u32 {
        if self.dead {
            return 0;
        }
        let dt = 1.0 / settings.fps;

        tick_sway(&mut self.sway, SWAY_SPEED);
        self.position.x = (self.base_x + SWAY_AMOUNT * self.sway.phase.sin())
            .clamp(0.0, (tank_width as f32 - 1.0).max(0.0));

        match self.phase {
            BubblePhase::Rising { hover_time } => {
                self.position.y -= self.rise_speed * dt;
                if self.position.y <= 0.0 {
                    self.position.y = 0.0;
                    self.phase = BubblePhase::Hovering {
                        remaining: hover_time,
                    };
                    return self.cash_value.unwrap_or(0);
                }
            }
            BubblePhase::Hovering { remaining } => {
                let new_rem = remaining - dt;
                if new_rem <= 0.0 {
                    self.dead = true;
                } else {
                    self.phase = BubblePhase::Hovering { remaining: new_rem };
                }
            }
            BubblePhase::Surface { remaining } => {
                self.position.y -= self.rise_speed * dt;
                let new_rem = remaining - dt;
                if new_rem <= 0.0 || self.position.y <= 0.0 {
                    self.dead = true;
                } else {
                    self.phase = BubblePhase::Surface { remaining: new_rem };
                }
            }
        }
        0
    }
}
