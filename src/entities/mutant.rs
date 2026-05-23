use std::f32::consts::PI;

use rand::RngExt;
use ratatui::style::Color;
use unicode_width::UnicodeWidthChar;

const EYE_OPEN_MIN: f32 = 0.8;
const EYE_OPEN_MAX: f32 = 1.8;
const EYE_CLOSED_MIN: f32 = 0.08;
const EYE_CLOSED_MAX: f32 = 0.35;
const CHAR_SPREAD: f32 = 1.0;
const WAVE_THRESHOLD: f32 = 0.8;
const GLISTEN_BASE_FACTOR: f32 = 0.70;
const GLISTEN_PEAK_FACTOR: f32 = 0.65;
pub const EXTRA_BODY_FOR_DOUBLE: usize = 2;
pub const MIN_BODY_CHARS: usize = 2;

#[derive(Clone, Copy, PartialEq)]
pub enum GlisteningMode {
    Wave,
    FullGlow,
    HalfHalf,
    CenterOut,
    OutsideIn,
}

impl GlisteningMode {
    pub fn random_other(self, rng: &mut impl RngExt) -> Self {
        const ALL: [GlisteningMode; 5] = [
            GlisteningMode::Wave,
            GlisteningMode::FullGlow,
            GlisteningMode::HalfHalf,
            GlisteningMode::CenterOut,
            GlisteningMode::OutsideIn,
        ];
        loop {
            let m = ALL[rng.random_range(0..ALL.len())];
            if m != self {
                return m;
            }
        }
    }

    pub fn glistening_value(self, phase: f32, i: usize, total: usize) -> f32 {
        let center = total as f32 / 2.0;
        let dist = (i as f32 - center).abs();
        match self {
            GlisteningMode::Wave => (phase - i as f32 * CHAR_SPREAD).sin(),
            GlisteningMode::FullGlow => phase.sin(),
            GlisteningMode::HalfHalf => {
                if i < total / 2 {
                    phase.sin()
                } else {
                    (phase + PI).sin()
                }
            }
            GlisteningMode::CenterOut => (phase - dist * CHAR_SPREAD).sin(),
            GlisteningMode::OutsideIn => (phase + dist * CHAR_SPREAD).sin(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MutantTail {
    Wide,
    Swaying,
    Curly,
}

impl MutantTail {
    pub fn display_width(self) -> usize {
        match self {
            MutantTail::Wide => 2,
            MutantTail::Swaying => 2,
            MutantTail::Curly => 3,
        }
    }

    pub fn chars(self, facing_left: bool, phase: f32) -> Vec<char> {
        match self {
            MutantTail::Wide => vec!['>', '<'],
            MutantTail::Swaying => {
                let base = if facing_left { '彡' } else { 'ミ' };
                if phase.sin() > WAVE_THRESHOLD {
                    let bw = UnicodeWidthChar::width(base).unwrap_or(1);
                    let ww = UnicodeWidthChar::width('≡').unwrap_or(1);
                    let mut v = vec!['≡'];
                    v.extend(std::iter::repeat_n(' ', bw.saturating_sub(ww)));
                    v
                } else {
                    vec![base]
                }
            }
            MutantTail::Curly => {
                if facing_left {
                    vec!['>', '<', '{']
                } else {
                    vec!['<', '>', '}']
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct EyeState {
    pub closed: bool,
    timer: f32,
}

impl EyeState {
    pub fn new(rng: &mut impl RngExt) -> Self {
        Self {
            closed: false,
            timer: rng.random_range(EYE_OPEN_MIN..EYE_OPEN_MAX),
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.closed = !self.closed;
            let mut rng = rand::rng();
            self.timer = if self.closed {
                rng.random_range(EYE_CLOSED_MIN..EYE_CLOSED_MAX)
            } else {
                rng.random_range(EYE_OPEN_MIN..EYE_OPEN_MAX)
            };
        }
    }

    pub fn small_char(&self) -> char {
        if self.closed { '¯' } else { 'º' }
    }

    pub fn big_char(&self) -> char {
        if self.closed { 'u' } else { 'ʘ' }
    }
}

#[derive(Clone)]
pub struct MutantState {
    pub left_eyes: Vec<EyeState>,
    pub right_eyes: Vec<EyeState>,
    pub eye_color: Option<Color>,
    pub glistening_mode: GlisteningMode,
    pub glistening_color: Option<Color>,
    pub body_variant: u8,
    pub tail_variant: MutantTail,
    pub mouth_inverted: bool,
    pub color_patches: Vec<(usize, Color)>,
    pub is_double: bool,
    pub double_head_eyes: Vec<EyeState>,
    pub mutation_count: u32,
    pub mutation_history: Vec<String>,
    pub mitosis_partners: Vec<String>,
}

impl MutantState {
    pub fn new_for_standard(tail_variant: MutantTail, rng: &mut impl RngExt) -> Self {
        Self {
            left_eyes: vec![EyeState::new(rng)],
            right_eyes: vec![EyeState::new(rng)],
            eye_color: None,
            glistening_mode: GlisteningMode::Wave,
            glistening_color: None,
            body_variant: 0,
            tail_variant,
            mouth_inverted: false,
            color_patches: Vec::new(),
            is_double: false,
            double_head_eyes: Vec::new(),
            mutation_count: 0,
            mutation_history: Vec::new(),
            mitosis_partners: Vec::new(),
        }
    }

    pub fn new(body_size: usize, seed: u64, rng: &mut impl RngExt) -> Self {
        let (head_count, tail_count) = pick_eye_counts(body_size, rng);
        let tail_variant = match (seed >> 2) % 3 {
            0 => MutantTail::Wide,
            1 => MutantTail::Swaying,
            _ => MutantTail::Curly,
        };
        Self {
            left_eyes: (0..head_count).map(|_| EyeState::new(rng)).collect(),
            right_eyes: (0..tail_count).map(|_| EyeState::new(rng)).collect(),
            eye_color: None,
            glistening_mode: GlisteningMode::Wave,
            glistening_color: None,
            body_variant: (seed % 4) as u8,
            tail_variant,
            mouth_inverted: false,
            color_patches: Vec::new(),
            is_double: false,
            double_head_eyes: Vec::new(),
            mutation_count: 0,
            mutation_history: Vec::new(),
            mitosis_partners: Vec::new(),
        }
    }

    pub fn display_width(&self, body_size: usize) -> usize {
        let max_eyes = self.left_eyes.len().max(self.right_eyes.len());
        if self.is_double {
            1 + max_eyes + body_size + EXTRA_BODY_FOR_DOUBLE + self.double_head_eyes.len() + 1
        } else {
            1 + max_eyes + body_size + self.tail_variant.display_width()
        }
    }

    pub fn tick_eyes(&mut self, dt: f32) {
        for e in &mut self.left_eyes {
            e.tick(dt);
        }
        for e in &mut self.right_eyes {
            e.tick(dt);
        }
        for e in &mut self.double_head_eyes {
            e.tick(dt);
        }
    }
}

pub fn pick_eye_counts(body_size: usize, rng: &mut impl RngExt) -> (usize, usize) {
    let max_eyes = body_size.saturating_sub(MIN_BODY_CHARS).clamp(1, 4);
    let left = rng.random_range(1..=max_eyes);
    let right = rng.random_range(1..=max_eyes);
    (left, right)
}

pub fn derive_glistening_palette(color: Color) -> (Color, Color, Color) {
    match color {
        Color::Rgb(r, g, b) => {
            let base = Color::Rgb(
                (r as f32 * GLISTEN_BASE_FACTOR) as u8,
                (g as f32 * GLISTEN_BASE_FACTOR) as u8,
                (b as f32 * GLISTEN_BASE_FACTOR) as u8,
            );
            let peak = Color::Rgb(
                r.saturating_add(((255u16 - r as u16) as f32 * GLISTEN_PEAK_FACTOR) as u8),
                g.saturating_add(((255u16 - g as u16) as f32 * GLISTEN_PEAK_FACTOR) as u8),
                b.saturating_add(((255u16 - b as u16) as f32 * GLISTEN_PEAK_FACTOR) as u8),
            );
            (base, color, peak)
        }
        _ => (Color::DarkGray, color, Color::White),
    }
}

pub fn random_rgb(rng: &mut impl RngExt) -> Color {
    const HUES: [(u8, u8, u8); 12] = [
        (255, 0, 0),
        (255, 100, 0),
        (255, 220, 0),
        (150, 255, 0),
        (0, 255, 0),
        (0, 255, 140),
        (0, 220, 255),
        (0, 100, 255),
        (0, 0, 255),
        (120, 0, 255),
        (220, 0, 255),
        (255, 0, 160),
    ];
    let (r, g, b) = HUES[rng.random_range(0..HUES.len())];
    Color::Rgb(r, g, b)
}
