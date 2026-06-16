use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{FOREST, GREEN_DARK, GREEN, LIGHT_GREEN, WHITE};

const MATRIX_CHARS: &[char] = &[
    'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ',
    'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ',
    'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ',
    'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    '+', '=', '-', '|', '*', '#', '@', '!', '?', '<', '>',
];

const SPEED_MIN: f32 = 4.0;
const SPEED_MAX: f32 = 14.0;
const TRAIL_LEN_MIN: usize = 6;
const TRAIL_LEN_MAX: usize = 14;
const RESTART_DELAY_MAX: f32 = 8.0;
const CHAR_INTERVAL_MIN: f32 = 0.04;
const CHAR_INTERVAL_MAX: f32 = 0.18;
const START_OFFSET_MAX: f32 = 10.0;

pub const TRAIL_COLORS: &[Color] = &[
    WHITE,
    LIGHT_GREEN,
    LIGHT_GREEN,
    GREEN,
    GREEN,
    GREEN,
    FOREST,
    FOREST,
    GREEN_DARK,
];

pub fn trail_color(distance: usize) -> Option<Color> {
    TRAIL_COLORS.get(distance).copied()
}

fn random_matrix_char(rng: &mut impl RngExt) -> char {
    MATRIX_CHARS[rng.random_range(0..MATRIX_CHARS.len())]
}

pub struct MatrixColumn {
    pub x: i32,
    pub head_y: f32,
    pub speed: f32,
    pub trail_length: usize,
    pub restart_delay: f32,
    pub chars: Vec<char>,
    char_timer: f32,
    char_interval: f32,
}

impl MatrixColumn {
    fn new(x: i32, rng: &mut impl RngExt) -> Self {
        let trail_length = rng.random_range(TRAIL_LEN_MIN..=TRAIL_LEN_MAX);
        let chars = (0..trail_length).map(|_| random_matrix_char(rng)).collect();
        Self {
            x,
            head_y: -(rng.random_range(0.0..START_OFFSET_MAX)),
            speed: rng.random_range(SPEED_MIN..SPEED_MAX),
            trail_length,
            restart_delay: 0.0,
            chars,
            char_timer: rng.random_range(CHAR_INTERVAL_MIN..CHAR_INTERVAL_MAX),
            char_interval: rng.random_range(CHAR_INTERVAL_MIN..CHAR_INTERVAL_MAX),
        }
    }

    pub fn tick(&mut self, dt: f32, height: i32, rng: &mut impl RngExt) {
        if self.restart_delay > 0.0 {
            self.restart_delay -= dt;
            return;
        }
        self.head_y += self.speed * dt;
        self.char_timer -= dt;
        if self.char_timer <= 0.0 {
            self.char_timer = self.char_interval;
            if !self.chars.is_empty() {
                self.chars[0] = random_matrix_char(rng);
            }
        }
        if self.head_y > height as f32 + self.trail_length as f32 {
            self.head_y = -(rng.random_range(0.0..START_OFFSET_MAX));
            self.speed = rng.random_range(SPEED_MIN..SPEED_MAX);
            self.trail_length = rng.random_range(TRAIL_LEN_MIN..=TRAIL_LEN_MAX);
            self.chars = (0..self.trail_length)
                .map(|_| random_matrix_char(rng))
                .collect();
            self.restart_delay = rng.random_range(0.0..RESTART_DELAY_MAX);
        }
    }
}

pub struct MatrixBackground {
    pub columns: Vec<MatrixColumn>,
}

impl MatrixBackground {
    pub fn new(width: i32, rng: &mut impl RngExt) -> Self {
        let columns = (0..width).map(|x| MatrixColumn::new(x, rng)).collect();
        Self { columns }
    }

    pub fn tick(&mut self, dt: f32, height: u16, rng: &mut impl RngExt) {
        let h = height as i32;
        for col in &mut self.columns {
            col.tick(dt, h, rng);
        }
    }
}

pub fn extend_matrix(columns: &mut Vec<MatrixColumn>, to_width: i32, rng: &mut impl RngExt) {
    let current = columns.len() as i32;
    for x in current..to_width {
        columns.push(MatrixColumn::new(x, rng));
    }
}
