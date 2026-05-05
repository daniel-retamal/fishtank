use std::f32::consts::TAU;

use rand::{RngExt, SeedableRng, rngs::SmallRng};
use ratatui::style::Color;
use unicode_width::UnicodeWidthChar;

use super::components::{Position, SwayState, Velocity, tick_sway};
use super::mutant::{EXTRA_BODY_FOR_DOUBLE, MutantState, derive_glistening_palette};
use super::species::{
    BodyChars, BodyTemplate, DEADFISH_BC_PLUS, DEADFISH_BC_SEMI, FishSpecies, PatternKind, TailKind,
};
use crate::settings::Settings;

const CHAR_SPREAD: f32 = 1.0;
pub const EATING_DURATION: f32 = 0.15;
const ZOOMIE_COOLDOWN_MIN: f32 = 120.0;
const ZOOMIE_COOLDOWN_MAX: f32 = 180.0;
const ZOOMIE_SPEED_MULTIPLIER: f32 = 11.0;
const ZOOMIE_SWAY_MULTIPLIER: f32 = 4.0;
const ZOOMIE_DURATION_MIN: f32 = 0.8;
const ZOOMIE_DURATION_MAX: f32 = 1.2;
const ZOOMIE_TURN_THRESHOLD: f32 = 0.35;
const DY_FRACTION: f32 = 0.4;
const WAVE_THRESHOLD: f32 = 0.8;
const GLISTEN_PEAK_THRESHOLD: f32 = 0.6;
const GLISTEN_MID_THRESHOLD: f32 = 0.1;
const VELOCITY_NORM_MIN: f32 = 0.01;
const DIRECTION_TIMER_MIN: u32 = 180;
const DIRECTION_TIMER_MAX: u32 = 480;
const DIRECTION_TIMER_POST_EVENT_MIN: u32 = 60;
const DIRECTION_TIMER_POST_EVENT_MAX: u32 = 180;
const ZOOMIE_INITIAL_TIMER_MIN: f32 = 30.0;
const ZOOMIE_INITIAL_TIMER_MAX: f32 = 90.0;

#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    fn flip(self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Clone, Copy)]
pub enum FishState {
    Idle,
    SeekingFood(usize, bool),
    Eating {
        time_remaining: f32,
    },
    Zoomie {
        time_remaining: f32,
        total_duration: f32,
        will_turn: bool,
        has_turned: bool,
    },
}

pub struct Fish {
    pub name: String,
    pub position: Position,
    pub velocity: Velocity,
    pub sway: SwayState,
    pub state: FishState,
    pub facing: Direction,
    pub body_size: usize,
    pub color: Color,
    pub speed: f32,
    pub seek_boost: f32,
    pub species: FishSpecies,
    pub pattern_seed: u64,
    pub display_width: usize,
    pub sway_speed: f32,
    pub mutant: Option<Box<MutantState>>,
    pub body_chars_override: Option<BodyChars>,
    pub food_eaten: usize,
    direction_timer: u32,
    zoomie_timer: f32,
}

impl Fish {
    pub fn new(species: FishSpecies, name: String, x: f32, y: f32, rng: &mut impl RngExt) -> Self {
        let cfg = species.config();

        let body_size = match cfg.body {
            BodyTemplate::Standard(_) => rng.random_range(cfg.size_range.0..=cfg.size_range.1),
            BodyTemplate::Fixed { .. } => 0,
        };

        let speed: f32 = rng.random_range(cfg.speed_range.0..cfg.speed_range.1);
        let pattern_seed: u64 = rng.random();
        let color = if species == FishSpecies::Mutantfish {
            FishSpecies::mutant_color_for_seed(pattern_seed)
        } else {
            cfg.palette[rng.random_range(0..cfg.palette.len())]
        };

        let mutant = if species == FishSpecies::Mutantfish {
            Some(Box::new(MutantState::new(body_size, pattern_seed, rng)))
        } else {
            None
        };

        let body_chars_override = if species == FishSpecies::Deadfish {
            Some(if pattern_seed % 2 == 0 {
                DEADFISH_BC_SEMI
            } else {
                DEADFISH_BC_PLUS
            })
        } else {
            None
        };

        let display_width = if let Some(ref m) = mutant {
            m.display_width(body_size)
        } else {
            compute_display_width(species, body_size)
        };

        let angle = rng.random::<f32>() * TAU;
        let dx = angle.cos() * speed;
        let dy = angle.sin() * speed * DY_FRACTION;

        Self {
            name,
            position: Position { x, y },
            velocity: Velocity { dx, dy },
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            state: FishState::Idle,
            facing: if dx < 0.0 {
                Direction::Left
            } else {
                Direction::Right
            },
            body_size,
            color,
            speed,
            seek_boost: 0.0,
            direction_timer: rng.random_range(DIRECTION_TIMER_MIN..DIRECTION_TIMER_MAX),
            zoomie_timer: rng.random_range(ZOOMIE_INITIAL_TIMER_MIN..ZOOMIE_INITIAL_TIMER_MAX),
            species,
            pattern_seed,
            display_width,
            sway_speed: cfg.sway_speed,
            mutant,
            body_chars_override,
            food_eaten: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.display_width
    }

    pub fn segments(&self) -> Vec<(char, Color)> {
        if self.species == FishSpecies::Mutantfish {
            return self.segments_mutant();
        }
        let cfg = self.species.config();
        let chars = self.build_chars(&cfg.body);
        let colors = if matches!(cfg.pattern, PatternKind::Glistening) {
            self.build_glistening_colors(
                chars.len(),
                cfg.palette[0],
                cfg.palette[1],
                cfg.palette[2],
            )
        } else {
            self.build_colors(chars.len(), cfg.palette, cfg.pattern)
        };
        let mut segs: Vec<(char, Color)> = chars.into_iter().zip(colors).collect();
        if matches!(cfg.body, BodyTemplate::Standard(_)) && matches!(self.facing, Direction::Right)
        {
            segs.reverse();
        }
        segs
    }

    fn build_chars(&self, body: &BodyTemplate) -> Vec<char> {
        match *body {
            BodyTemplate::Standard(bc) => {
                self.build_standard_chars(self.body_chars_override.unwrap_or(bc))
            }
            BodyTemplate::Fixed { left, right } => {
                let variants = match self.facing {
                    Direction::Left => left,
                    Direction::Right => right,
                };
                let idx = self.pattern_seed as usize % variants.len();
                variants[idx].chars().collect()
            }
        }
    }

    fn build_standard_chars(&self, bc: BodyChars) -> Vec<char> {
        let (body_char, wave_char, raw_mouth, eye) = match self.facing {
            Direction::Left => (bc.body_left, bc.wave_left, bc.mouth_left, bc.eye_left),
            Direction::Right => (bc.body_right, bc.wave_right, bc.mouth_right, bc.eye_right),
        };
        let mouth = if matches!(self.state, FishState::Eating { .. }) {
            invert_mouth(raw_mouth)
        } else {
            raw_mouth
        };

        let substituting = self.sway.phase.sin() > WAVE_THRESHOLD;
        let body: Vec<char> = (0..self.body_size)
            .map(|_| if substituting { wave_char } else { body_char })
            .collect();

        let tail = tail_chars(bc, self.facing, self.sway.phase);

        let mut chars = vec![mouth, eye];
        chars.extend(body);
        chars.extend(tail);
        chars
    }

    fn build_colors(
        &self,
        len: usize,
        palette: &'static [Color],
        pattern: PatternKind,
    ) -> Vec<Color> {
        match pattern {
            PatternKind::Solid => vec![self.color; len],
            PatternKind::Striped => (0..len).map(|i| palette[i % palette.len()]).collect(),
            PatternKind::Patchy => {
                let mut rng = SmallRng::seed_from_u64(self.pattern_seed);
                (0..len)
                    .map(|_| palette[rng.random_range(0..palette.len())])
                    .collect()
            }
            PatternKind::PatchyAll => {
                let mut rng = SmallRng::seed_from_u64(self.pattern_seed);
                let mut colors: Vec<Color> = (0..len)
                    .map(|_| palette[rng.random_range(0..palette.len())])
                    .collect();
                for (slot, &forced) in palette.iter().enumerate().take(len) {
                    if !colors.contains(&forced) {
                        colors[slot] = forced;
                    }
                }
                colors
            }
            PatternKind::Glistening => unreachable!(),
        }
    }

    fn build_glistening_colors(
        &self,
        len: usize,
        base: Color,
        mid: Color,
        peak: Color,
    ) -> Vec<Color> {
        (0..len)
            .map(|i| {
                let s = (self.sway.phase - i as f32 * CHAR_SPREAD).sin();
                if s > GLISTEN_PEAK_THRESHOLD {
                    peak
                } else if s > GLISTEN_MID_THRESHOLD {
                    mid
                } else {
                    base
                }
            })
            .collect()
    }

    fn segments_mutant(&self) -> Vec<(char, Color)> {
        let m = self.mutant.as_ref().unwrap();
        let facing_left = matches!(self.facing, Direction::Left);
        let (raw_mouth, body_ch, wave_ch) =
            body_chars_for_variant(m.body_variant, facing_left, m.mouth_inverted);
        let mouth = if matches!(self.state, FishState::Eating { .. }) {
            invert_mouth(raw_mouth)
        } else {
            raw_mouth
        };
        let sway_high = self.sway.phase.sin() > WAVE_THRESHOLD;

        let max_eyes = m.left_eyes.len().max(m.right_eyes.len());

        let mut chars: Vec<char> = Vec::new();
        chars.push(mouth);

        let eye_start = chars.len();
        let eye_count = if facing_left {
            for e in &m.left_eyes {
                chars.push(e.small_char());
            }
            m.left_eyes.len()
        } else {
            for e in &m.right_eyes {
                chars.push(e.big_char());
            }
            m.right_eyes.len()
        };
        let extra_body = max_eyes - eye_count;

        for _ in 0..(self.body_size + extra_body) {
            chars.push(if sway_high { wave_ch } else { body_ch });
        }

        let double_eye_start;
        let double_eye_count;

        if m.is_double {
            for _ in 0..EXTRA_BODY_FOR_DOUBLE {
                chars.push(if sway_high { wave_ch } else { body_ch });
            }
            double_eye_start = chars.len();
            for e in &m.double_head_eyes {
                chars.push(if facing_left {
                    e.big_char()
                } else {
                    e.small_char()
                });
            }
            double_eye_count = m.double_head_eyes.len();
            chars.push(if facing_left { '>' } else { '<' });
        } else {
            chars.extend(m.tail_variant.chars(facing_left, self.sway.phase));
            double_eye_start = 0;
            double_eye_count = 0;
        }

        let n = chars.len();
        let (base, mid, peak_default) = derive_glistening_palette(self.color);
        let peak = m.glistening_color.unwrap_or(peak_default);
        let mut colors: Vec<Color> = (0..n)
            .map(|i| color_for_glisten(m.glistening_mode, self.sway.phase, i, n, base, mid, peak))
            .collect();
        apply_patches_and_eyes(
            &mut colors,
            &m.color_patches,
            m.eye_color,
            eye_start,
            eye_count,
            double_eye_start,
            double_eye_count,
        );
        let mut segs: Vec<(char, Color)> = chars.into_iter().zip(colors).collect();
        if !facing_left {
            segs.reverse();
        }
        segs
    }

    pub fn head_x(&self) -> i32 {
        match self.facing {
            Direction::Left => self.position.x as i32,
            Direction::Right => self.position.x as i32 + self.display_width as i32 - 1,
        }
    }

    pub fn tick(&mut self, settings: &Settings, tank_width: u16, tank_height: u16) {
        let dt = 1.0 / settings.fps;

        match self.state {
            FishState::Idle => {
                self.zoomie_timer -= dt;
                if self.zoomie_timer <= 0.0 {
                    self.start_zoomie();
                } else {
                    self.direction_timer = self.direction_timer.saturating_sub(1);
                    if self.direction_timer == 0 {
                        self.randomize_direction();
                    }
                }
            }
            FishState::Zoomie {
                time_remaining,
                total_duration,
                will_turn,
                has_turned,
            } => {
                let new_time = time_remaining - dt;
                let should_turn =
                    will_turn && !has_turned && new_time < total_duration * ZOOMIE_TURN_THRESHOLD;

                if should_turn {
                    self.velocity.dx = -self.velocity.dx;
                    self.facing = self.facing.flip();
                }

                if new_time <= 0.0 {
                    self.end_zoomie();
                } else {
                    self.state = FishState::Zoomie {
                        time_remaining: new_time,
                        total_duration,
                        will_turn,
                        has_turned: has_turned || should_turn,
                    };
                }
            }
            FishState::SeekingFood(_, _) => {}
            FishState::Eating { time_remaining } => {
                let new_time = time_remaining - dt;
                if new_time <= 0.0 {
                    self.cancel_seek();
                } else {
                    self.state = FishState::Eating {
                        time_remaining: new_time,
                    };
                }
            }
        }

        if !matches!(self.state, FishState::Eating { .. }) {
            self.position.x += self.velocity.dx * dt;
            self.position.y += self.velocity.dy * dt;
            self.bounce_walls(tank_width, tank_height);
        }

        let effective_sway_speed = if matches!(self.state, FishState::Zoomie { .. }) {
            self.sway_speed * ZOOMIE_SWAY_MULTIPLIER
        } else {
            self.sway_speed
        };
        tick_sway(&mut self.sway, effective_sway_speed);

        if let Some(ref mut m) = self.mutant {
            m.tick_eyes(dt);
        }
    }

    pub fn cancel_seek(&mut self) {
        let norm = (self.velocity.dx * self.velocity.dx + self.velocity.dy * self.velocity.dy)
            .sqrt()
            .max(VELOCITY_NORM_MIN);
        self.velocity.dx = (self.velocity.dx / norm) * self.speed;
        self.velocity.dy = (self.velocity.dy / norm) * self.speed * DY_FRACTION;
        self.seek_boost = 0.0;
        let mut rng = rand::rng();
        self.direction_timer =
            rng.random_range(DIRECTION_TIMER_POST_EVENT_MIN..DIRECTION_TIMER_POST_EVENT_MAX);
        self.state = FishState::Idle;
    }

    pub fn randomize_direction(&mut self) {
        let mut rng = rand::rng();
        let angle = rng.random::<f32>() * TAU;
        self.velocity.dx = angle.cos() * self.speed;
        self.velocity.dy = angle.sin() * self.speed * DY_FRACTION;

        if self.velocity.dx != 0.0 {
            self.facing = if self.velocity.dx < 0.0 {
                Direction::Left
            } else {
                Direction::Right
            };
        }

        self.direction_timer = rng.random_range(DIRECTION_TIMER_MIN..DIRECTION_TIMER_MAX);
    }

    fn start_zoomie(&mut self) {
        let mut rng = rand::rng();
        let duration = rng.random_range(ZOOMIE_DURATION_MIN..ZOOMIE_DURATION_MAX);
        let zoomie_speed = self.speed * ZOOMIE_SPEED_MULTIPLIER;

        let will_turn = if self.species == FishSpecies::Jellyfish {
            self.velocity.dx = 0.0;
            self.velocity.dy = -zoomie_speed;
            false
        } else {
            self.velocity.dx = match self.facing {
                Direction::Left => -zoomie_speed,
                Direction::Right => zoomie_speed,
            };
            self.velocity.dy = 0.0;
            rng.random::<bool>()
        };

        self.zoomie_timer = rng.random_range(ZOOMIE_COOLDOWN_MIN..ZOOMIE_COOLDOWN_MAX);
        self.state = FishState::Zoomie {
            time_remaining: duration,
            total_duration: duration,
            will_turn,
            has_turned: false,
        };
    }

    fn end_zoomie(&mut self) {
        let mut rng = rand::rng();
        let sign = if self.velocity.dx >= 0.0 {
            1.0_f32
        } else {
            -1.0_f32
        };
        self.velocity.dx = sign * self.speed;
        self.velocity.dy = 0.0;
        self.direction_timer =
            rng.random_range(DIRECTION_TIMER_POST_EVENT_MIN..DIRECTION_TIMER_POST_EVENT_MAX);
        self.state = FishState::Idle;
    }

    pub fn static_left_segments(&self) -> Vec<(char, Color)> {
        if self.species == FishSpecies::Mutantfish {
            return self.static_left_segments_mutant();
        }
        let cfg = self.species.config();
        match cfg.body {
            BodyTemplate::Fixed { left, .. } => {
                let idx = self.pattern_seed as usize % left.len();
                let chars: Vec<char> = left[idx].chars().collect();
                let len = chars.len();
                let colors = self.build_colors(len, cfg.palette, cfg.pattern);
                chars.into_iter().zip(colors).collect()
            }
            BodyTemplate::Standard(bc) => {
                let mut chars = vec![bc.mouth_left, bc.eye_left];
                chars.extend(std::iter::repeat_n(bc.body_left, self.body_size));
                let tail: Vec<char> = match bc.tail {
                    TailKind::Wide => vec!['>', '<'],
                    TailKind::WideCurly => vec!['>', '<', '{'],
                    TailKind::Short => vec!['<'],
                    TailKind::Custom { left, .. } => vec![left],
                    TailKind::Swaying { left, .. } => vec![left],
                    TailKind::None => vec![],
                };
                chars.extend(tail);
                let len = chars.len();
                let colors = if matches!(cfg.pattern, PatternKind::Glistening) {
                    self.build_glistening_colors(
                        len,
                        cfg.palette[0],
                        cfg.palette[1],
                        cfg.palette[2],
                    )
                } else {
                    self.build_colors(len, cfg.palette, cfg.pattern)
                };
                chars.into_iter().zip(colors).collect()
            }
        }
    }

    fn static_left_segments_mutant(&self) -> Vec<(char, Color)> {
        let m = self.mutant.as_ref().unwrap();
        let (raw_mouth, body_ch, _) =
            body_chars_for_variant(m.body_variant, true, m.mouth_inverted);
        let max_eyes = m.left_eyes.len().max(m.right_eyes.len());
        let mut chars: Vec<char> = Vec::new();
        chars.push(raw_mouth);
        let eye_start = chars.len();
        for e in &m.left_eyes {
            chars.push(e.small_char());
        }
        let eye_count = m.left_eyes.len();
        let extra_body = max_eyes - eye_count;
        for _ in 0..(self.body_size + extra_body) {
            chars.push(body_ch);
        }
        let (double_eye_start, double_eye_count) = if m.is_double {
            for _ in 0..EXTRA_BODY_FOR_DOUBLE {
                chars.push(body_ch);
            }
            let start = chars.len();
            for e in &m.double_head_eyes {
                chars.push(e.big_char());
            }
            let count = m.double_head_eyes.len();
            chars.push('>');
            (start, count)
        } else {
            chars.extend(m.tail_variant.chars(true, 0.0));
            (0, 0)
        };
        let n = chars.len();
        let (base, mid, peak_default) = derive_glistening_palette(self.color);
        let peak = m.glistening_color.unwrap_or(peak_default);
        let mut colors: Vec<Color> = (0..n)
            .map(|i| color_for_glisten(m.glistening_mode, 0.0, i, n, base, mid, peak))
            .collect();
        apply_patches_and_eyes(
            &mut colors,
            &m.color_patches,
            m.eye_color,
            eye_start,
            eye_count,
            double_eye_start,
            double_eye_count,
        );
        chars.into_iter().zip(colors).collect()
    }

    fn bounce_walls(&mut self, tank_width: u16, tank_height: u16) {
        let max_x = tank_width as f32 - self.display_width as f32;
        let max_y = tank_height as f32 - 1.0;

        if self.position.x < 0.0 {
            self.position.x = 0.0;
            self.velocity.dx = self.velocity.dx.abs();
            if matches!(self.state, FishState::Idle | FishState::Zoomie { .. }) {
                self.facing = Direction::Right;
            }
        } else if self.position.x > max_x {
            self.position.x = max_x;
            self.velocity.dx = -self.velocity.dx.abs();
            if matches!(self.state, FishState::Idle | FishState::Zoomie { .. }) {
                self.facing = Direction::Left;
            }
        }

        if self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.dy = self.velocity.dy.abs();
        } else if self.position.y > max_y {
            self.position.y = max_y;
            self.velocity.dy = -self.velocity.dy.abs();
        }
    }
}

fn invert_mouth(ch: char) -> char {
    match ch {
        '<' => '>',
        '>' => '<',
        _ => ch,
    }
}

fn body_chars_for_variant(
    variant: u8,
    facing_left: bool,
    mouth_inverted: bool,
) -> (char, char, char) {
    let mouth = if facing_left ^ mouth_inverted {
        '<'
    } else {
        '>'
    };
    let (body, wave) = match (variant, facing_left) {
        (0, true) => ('(', '{'),
        (0, false) => (')', '}'),
        (1, true) => ('}', ')'),
        (1, false) => ('{', '('),
        (3, _) => ('+', '-'),
        _ => (';', ':'),
    };
    (mouth, body, wave)
}

fn color_for_glisten(
    mode: super::mutant::GlisteningMode,
    phase: f32,
    i: usize,
    total: usize,
    base: Color,
    mid: Color,
    peak: Color,
) -> Color {
    let v = mode.glistening_value(phase, i, total);
    if v > GLISTEN_PEAK_THRESHOLD {
        peak
    } else if v > GLISTEN_MID_THRESHOLD {
        mid
    } else {
        base
    }
}

fn apply_patches_and_eyes(
    colors: &mut [Color],
    patches: &[(usize, Color)],
    eye_color: Option<Color>,
    eye_a_start: usize,
    eye_a_count: usize,
    eye_b_start: usize,
    eye_b_count: usize,
) {
    for &(pos, c) in patches {
        if pos < colors.len() {
            colors[pos] = c;
        }
    }
    if let Some(ec) = eye_color {
        for i in eye_a_start..eye_a_start + eye_a_count {
            if i < colors.len() {
                colors[i] = ec;
            }
        }
        for i in eye_b_start..eye_b_start + eye_b_count {
            if i < colors.len() {
                colors[i] = ec;
            }
        }
    }
}

fn tail_chars(bc: BodyChars, facing: Direction, phase: f32) -> Vec<char> {
    match bc.tail {
        TailKind::Wide => vec!['>', '<'],
        TailKind::WideCurly => match facing {
            Direction::Left => vec!['>', '<', '{'],
            Direction::Right => vec!['<', '>', '}'],
        },
        TailKind::Short => match facing {
            Direction::Left => vec!['<'],
            Direction::Right => vec!['>'],
        },
        TailKind::Custom { left, right } => match facing {
            Direction::Left => vec![left],
            Direction::Right => vec![right],
        },
        TailKind::Swaying { left, right, wave } => {
            let base = match facing {
                Direction::Left => left,
                Direction::Right => right,
            };
            if phase.sin() > WAVE_THRESHOLD {
                let base_w = UnicodeWidthChar::width(base).unwrap_or(1);
                let wave_w = UnicodeWidthChar::width(wave).unwrap_or(1);
                let mut chars = vec![wave];
                chars.extend(std::iter::repeat_n(' ', base_w.saturating_sub(wave_w)));
                chars
            } else {
                vec![base]
            }
        }
        TailKind::None => vec![],
    }
}

pub fn compute_display_width(species: FishSpecies, body_size: usize) -> usize {
    let cfg = species.config();
    match cfg.body {
        BodyTemplate::Standard(bc) => {
            let tail_w = match bc.tail {
                TailKind::Wide => 2,
                TailKind::WideCurly => 3,
                TailKind::Short => 1,
                TailKind::None => 0,
                TailKind::Custom { left, right } => {
                    let lw = UnicodeWidthChar::width(left).unwrap_or(1);
                    let rw = UnicodeWidthChar::width(right).unwrap_or(1);
                    lw.max(rw)
                }
                TailKind::Swaying {
                    left,
                    right,
                    wave: _,
                } => {
                    let lw = UnicodeWidthChar::width(left).unwrap_or(1);
                    let rw = UnicodeWidthChar::width(right).unwrap_or(1);
                    lw.max(rw)
                }
            };
            2 + body_size + tail_w
        }
        BodyTemplate::Fixed { left, .. } => {
            let variant = left.first().copied().unwrap_or("");
            unicode_width::UnicodeWidthStr::width(variant)
        }
    }
}
