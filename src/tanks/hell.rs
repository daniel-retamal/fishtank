use std::f32::consts::TAU;

use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{LIGHT_RED, RED, RED_DARK};
use crate::entities::components::{EyeRow, SwayState, extend_spaced, sway_x_offset, tick_sway};
use crate::entities::plant::Seaweed;
use crate::fishes::species::EYE_CIRCLE;

pub const FACE_COLOR: Color = RED_DARK;
pub const RANDOM_FACE_COLOR: Color = RED;

pub const H_WAVE_AMPLITUDE: f32 = 3.0;
pub const H_WAVE_ROW_SPREAD: f32 = 0.3;

const DIRECTION_CHANGE_INTERVAL: f32 = 30.0;
const RANDOM_MODE_CHANCE: f32 = 0.05;
const FAST_SPEED_CHANCE: f32 = 0.05;
const FAST_SPEED_MULT: f32 = 1.5;
const BASE_MOVE_SPEED: f32 = 2.0;
const BASE_H_WAVE_SPEED: f32 = 0.8;
const RANDOM_REFRESH_RATE: f32 = 0.1;

const HELL_PLANT_SPACING_MIN: i32 = 3;
const HELL_PLANT_SPACING_MAX: i32 = 6;
const HELL_PLANT_HEIGHT_MIN: usize = 8;
const HELL_PLANT_HEIGHT_MAX: usize = 27;
const SWAY_SPEED: f32 = 0.04;
const WAVE_SPREAD: f32 = 0.5;
const SWAY_AMOUNT: f32 = 2.0;

const EYE_SPACING_MIN: usize = 2;
const EYE_SPACING_MAX: usize = 3;
const EYE_CLOSED_CHAR: char = '-';

const RANDOM_CHARS: &[char] = &[' ', ' ', ' ', ' ', ' ', '.', ':', '.', ':', '.'];

const HELL_PLANT_COLORS: &[Color] = &[RED, LIGHT_RED, RED, LIGHT_RED, RED];

pub const FACE_LINES: &[&str] = &[
    "                                        ..::::.",
    "                                 .  .....:::...::.",
    "                      ..:::.......   .  .     . .::::. .",
    "                    ::.   ::...          .     ..   ......   ...",
    "                 ..  .:::::....::::....  :.::.     .  .::::: .  :..",
    "                 . . ...:.  .         ... .:::....   ....:::: . .::.",
    "               ::.  .... .  ... ..       .::::: ....       .:::  ..::.",
    "            .::.  .. .   .:::...  .           ::.. ....  .  ..... . .:.",
    "          .::: . ::.   ...                      .:.. .  .:    .... .....",
    "          . .  ::. ...                                  .:     ...::.. :.",
    "        ..    ::. ..                ..........           ..      :::::...",
    "       .:.   :::..                 .:::. .:: ....           ..    :::::..",
    "      .:::  ::::..               .:::::. . ...::...          ..      :::   ..",
    "     ..  : .. . .              .:.::. ........  .:...     .    . .  . :::  ...",
    "    ..    . . .               .....  .   .:.      ....     .   ::. ....:.:..::.",
    "    .. . ..                .  :. ...:.   .::   ::..         .   :::.   ...:. ::",
    "         .:.             ..   .  :...  . .::.   ....        .   .::   . ..    ::",
    "    ..  ..:.            .            .... .  .. ..   .       .   ..   .. . .. .:",
    "  ... . ...  .        ..     ..  . ::     .   ..:.   ..      .    .     .. .   ::",
    "  :.. . .. .         ..     .:..::.: ...::..:... ..:::..     .    .    ::::: . .:",
    "  .. .   . . .      ..      .::.  : ....     .::. . .:.      .    .     :::::.. .",
    "  ..    .::.        .            ...            ...         ..        ...::: :. . .",
    "  ..  . .::        ..           .:.              ..:        .          . .:.  .",
    " . .  . ::. .     ...           ...               .:       ..           .::. .  ..",
    "      . ::  .     .:.            .                ..      ..          . ::: .. ..",
    "     ...:::      ..:             .                .      ..            .:::    ..",
    "   . ...::::     ..:.            .                .    ..            . .. .   ..:",
    "   :.   ::.. .   ..:.           ...              ...  .                .... ....:",
    "   .: ..  . .     .::            ..              ..              .....  ... .. .:",
    "    ::  . .      :..:.  .        ...            ..             .... :.. :..   :.",
    "     :.  ....   .::...   .        ...          ...            ::.  .:.:.:.   ..",
    "     .::. .:..  . .:. .   .          .        .              .:.  ... :::  . .",
    "      .:.. .::.   . .  .   .           ..:::. .             ::. .::.  ....  ..",
    "       .. . .::.            ..         ......            .::.  .::.  ..  . ....",
    "         .   .::::.     ..    ..                      .:::.   :::. ..   .::::.",
    "           .  :::::.    ..    .  .                  .:....::::::.      .:::.",
    "           :. :.::...    ..:. :.     ......:....:::.      .::::. ....:::.",
    "            :..... ...    ...:::. ..    .  ::::   .     .. :.:: ...::.",
    "             ::  . .. .   .    .:::. ..  .   ..::.  ...  .. .",
    "              .::.. ..::.         ..::.    ....  :::..     .",
    "                .::   .:::::..             ::..:::.  ......  . ..",
    "                  ..    .::....       ....::::::.     .. ..:..",
    "                           .  ...       .....     ......",
    "                               .:::.     ...  . .",
    "                                   .:::::::::...",
];

pub struct HellBackground {
    pub face_grid: Vec<Vec<char>>,
    pub offset_x: f32,
    pub offset_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub h_phase: f32,
    pub h_phase_speed: f32,
    pub is_random: bool,
    pub random_buffer: Vec<char>,
    pub is_frantic: bool,
    frantic_timer: f32,
    random_refresh_timer: f32,
    cycle_timer: f32,
}

impl HellBackground {
    pub fn new(rng: &mut impl RngExt) -> Self {
        let face_grid = build_face_grid();
        let (vel_x, vel_y) = pick_nonzero_velocity(rng, BASE_MOVE_SPEED);
        Self {
            face_grid,
            offset_x: rng.random::<f32>() * 100.0,
            offset_y: rng.random::<f32>() * 100.0,
            vel_x,
            vel_y,
            h_phase: rng.random::<f32>() * TAU,
            h_phase_speed: BASE_H_WAVE_SPEED,
            is_random: false,
            random_buffer: Vec::new(),
            is_frantic: false,
            frantic_timer: 0.0,
            random_refresh_timer: RANDOM_REFRESH_RATE,
            cycle_timer: DIRECTION_CHANGE_INTERVAL,
        }
    }

    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt, w: u16, h: u16) {
        if self.is_frantic {
            self.frantic_timer -= dt;
            if self.frantic_timer <= 0.0 {
                self.is_frantic = false;
            }
        }

        self.cycle_timer -= dt;
        if self.cycle_timer <= 0.0 {
            self.cycle_timer = DIRECTION_CHANGE_INTERVAL;
            self.is_random = rng.random::<f32>() < RANDOM_MODE_CHANCE;
            if self.is_random {
                self.random_buffer = generate_random_buffer(rng, w, h);
            }
            if rng.random::<f32>() < FAST_SPEED_CHANCE {
                self.is_frantic = true;
                self.frantic_timer = DIRECTION_CHANGE_INTERVAL;
            }
            let (vx, vy) = pick_nonzero_velocity(rng, BASE_MOVE_SPEED);
            self.vel_x = vx;
            self.vel_y = vy;
            self.h_phase_speed = BASE_H_WAVE_SPEED;
        }

        let face_h = self.face_grid.len();
        if face_h == 0 {
            return;
        }
        let face_w = self.face_grid[0].len();
        let period_x = face_w as f32;
        let period_y = 2.0 * face_h as f32;

        let speed_mult = if self.is_frantic {
            FAST_SPEED_MULT
        } else {
            1.0
        };
        self.offset_x = (self.offset_x + self.vel_x * dt * speed_mult).rem_euclid(period_x);
        self.offset_y = (self.offset_y + self.vel_y * dt * speed_mult).rem_euclid(period_y);
        self.h_phase = (self.h_phase + self.h_phase_speed * dt * speed_mult).rem_euclid(TAU);

        if self.is_random {
            self.random_refresh_timer -= dt;
            if self.random_refresh_timer <= 0.0 {
                self.random_refresh_timer = RANDOM_REFRESH_RATE;
                self.random_buffer = generate_random_buffer(rng, w, h);
            }
        }
    }
}

pub struct HellPlant {
    pub x: i32,
    pub height: usize,
    pub sway: SwayState,
    pub color: Color,
    eyes: Vec<EyeRow>,
}

impl HellPlant {
    pub fn new(x: i32, height: usize, rng: &mut impl RngExt) -> Self {
        let color = HELL_PLANT_COLORS[rng.random_range(0..HELL_PLANT_COLORS.len())];
        let mut eyes = Vec::new();
        let mut next_eye_height = rng.random_range(EYE_SPACING_MIN..=EYE_SPACING_MAX);
        while next_eye_height < height {
            eyes.push(EyeRow::new(next_eye_height, rng));
            next_eye_height += rng.random_range(EYE_SPACING_MIN..=EYE_SPACING_MAX);
        }
        Self {
            x,
            height,
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            color,
            eyes,
        }
    }
}

impl Seaweed for HellPlant {
    fn x(&self) -> i32 {
        self.x
    }

    fn height(&self) -> usize {
        self.height
    }

    fn segment_at(&self, row: usize) -> (i32, char) {
        let x_offset = sway_x_offset(self.sway.phase, row, self.height, WAVE_SPREAD, SWAY_AMOUNT);
        if let Some(eye) = self.eyes.iter().find(|e| e.height == row) {
            let ch = if eye.blink.is_open {
                EYE_CIRCLE
            } else {
                EYE_CLOSED_CHAR
            };
            return (x_offset, ch);
        }
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

    fn tick(&mut self, delta_time: f32) {
        tick_sway(&mut self.sway, SWAY_SPEED);
        for eye in &mut self.eyes {
            eye.blink.tick(delta_time);
        }
    }
}

pub fn extend_hell_plants(plants: &mut Vec<HellPlant>, to_width: i32, rng: &mut impl RngExt) {
    extend_spaced(
        plants,
        to_width,
        0,
        HELL_PLANT_SPACING_MIN..=HELL_PLANT_SPACING_MAX,
        rng,
        |p| p.x,
        |x, rng| {
            HellPlant::new(
                x,
                rng.random_range(HELL_PLANT_HEIGHT_MIN..=HELL_PLANT_HEIGHT_MAX),
                rng,
            )
        },
    );
}

fn build_face_grid() -> Vec<Vec<char>> {
    let face_w = FACE_LINES.iter().map(|l| l.len()).max().unwrap_or(0);
    FACE_LINES
        .iter()
        .map(|line| {
            let mut chars: Vec<char> = line.chars().collect();
            chars.resize(face_w, ' ');
            chars
        })
        .collect()
}

fn generate_random_buffer(rng: &mut impl RngExt, w: u16, h: u16) -> Vec<char> {
    let size = w as usize * h as usize;
    (0..size)
        .map(|_| RANDOM_CHARS[rng.random_range(0..RANDOM_CHARS.len())])
        .collect()
}

fn pick_nonzero_velocity(rng: &mut impl RngExt, speed: f32) -> (f32, f32) {
    loop {
        let vx = vel_component(rng, speed);
        let vy = vel_component(rng, speed);
        if vx != 0.0 || vy != 0.0 {
            return (vx, vy);
        }
    }
}

fn vel_component(rng: &mut impl RngExt, speed: f32) -> f32 {
    match rng.random_range(0..3u8) {
        0 => -speed,
        1 => speed,
        _ => 0.0,
    }
}
