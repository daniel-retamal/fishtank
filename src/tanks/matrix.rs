use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{FOREST, GREEN, GREEN_DARK, LIGHT_GREEN, WHITE};

const MATRIX_CHARS: &[char] = &[
    'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ',
    'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ',
    'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ', '0', '1',
    '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
    'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '+', '=', '-', '|',
    '*', '#', '@', '!', '?', '<', '>',
];

const FALL_SPEED: f32 = 9.0;
const TRAIL_LEN_MIN: usize = 16;
const TRAIL_LEN_MAX: usize = 40;
const RESTART_DELAY_MIN: f32 = 3.0;
const RESTART_DELAY_MAX: f32 = 16.0;
const INITIAL_DELAY_MAX: f32 = 16.0;
const SWAP_INTERVAL_MEAN: f32 = 2.0;

pub fn trail_color(distance: usize) -> Color {
    match distance {
        0 => WHITE,
        1 => LIGHT_GREEN,
        2..=4 => GREEN,
        5..=10 => FOREST,
        _ => GREEN_DARK,
    }
}

fn random_matrix_char(rng: &mut impl RngExt) -> char {
    MATRIX_CHARS[rng.random_range(0..MATRIX_CHARS.len())]
}

fn sample_swap_delay(rng: &mut impl RngExt) -> f32 {
    let u: f32 = rng.random();
    -(1.0 - u).ln() * SWAP_INTERVAL_MEAN
}

pub struct MatrixColumn {
    pub x: i32,
    pub head_row: i32,
    pub chars: Vec<char>,
    progress: f32,
    trail_length: usize,
    restart_delay: f32,
    swap_timer: f32,
}

impl MatrixColumn {
    fn new(x: i32, rng: &mut impl RngExt) -> Self {
        Self {
            x,
            head_row: 0,
            chars: Vec::new(),
            progress: 0.0,
            trail_length: rng.random_range(TRAIL_LEN_MIN..=TRAIL_LEN_MAX),
            restart_delay: rng.random_range(0.0..INITIAL_DELAY_MAX),
            swap_timer: 0.0,
        }
    }

    fn begin_drop(&mut self, rng: &mut impl RngExt) {
        self.head_row = 0;
        self.progress = 0.0;
        self.trail_length = rng.random_range(TRAIL_LEN_MIN..=TRAIL_LEN_MAX);
        self.swap_timer = sample_swap_delay(rng);
        self.chars = vec![random_matrix_char(rng)];
    }

    pub fn tick(&mut self, dt: f32, height: i32, rng: &mut impl RngExt) {
        if self.restart_delay > 0.0 {
            self.restart_delay -= dt;
            return;
        }
        if self.chars.is_empty() {
            self.begin_drop(rng);
        }

        self.progress += FALL_SPEED * dt;
        while self.progress >= 1.0 {
            self.progress -= 1.0;
            self.head_row += 1;
            self.chars.insert(0, random_matrix_char(rng));
            self.chars.truncate(self.trail_length);
        }

        self.swap_timer -= dt;
        if self.swap_timer <= 0.0 {
            let idx = rng.random_range(0..self.chars.len());
            self.chars[idx] = random_matrix_char(rng);
            self.swap_timer = sample_swap_delay(rng);
        }

        if self.head_row - self.chars.len() as i32 + 1 > height {
            self.chars.clear();
            self.restart_delay = rng.random_range(RESTART_DELAY_MIN..RESTART_DELAY_MAX);
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
