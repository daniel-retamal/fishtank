use std::f32::consts::TAU;

use rand::{RngExt, SeedableRng, rngs::SmallRng};
use ratatui::style::Color;
use unicode_width::UnicodeWidthChar;

use super::mutant::{EXTRA_BODY_FOR_DOUBLE, MutantState, MutationRecord};
use super::species::{BodyChars, BodyTemplate, FishSpecies, PatternKind, SizeCategory, TailKind};
use super::unfish::{
    BALL_WIDTH, BLINKER_BASE_COLOR, BLINKER_GLISTEN_MID, BLINKER_GLISTEN_PEAK, BLINKER_MID_COLOR,
    BLINKER_PEAK_COLOR, SKULL_WIDTH, UNFISH_BODY_COLOR, UNFISH_EYE_COLOR, UnfishKind, UnfishState,
    WORM_DEFAULT_SEGMENTS, build_worm, is_multi_row, worm_display_width, worm_eye_cols,
};
use crate::colors::{DARK_GRAY, WHITE};
use crate::consumable::{COFFEE_SPEED_MULT, COFFEE_SWAY_MULT, COFFEE_ZOOMIE_DT_MULT};
use crate::entities::components::{Position, SwayState, Velocity, tick_sway};
use crate::entities::glistening::{GlisteningMode, color_for_glisten, derive_glistening_palette};
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
const VELOCITY_NORM_MIN: f32 = 0.01;
const DIRECTION_TIMER_MIN: u32 = 180;
const DIRECTION_TIMER_MAX: u32 = 480;
const DIRECTION_TIMER_POST_EVENT_MIN: u32 = 60;
const DIRECTION_TIMER_POST_EVENT_MAX: u32 = 180;
const ZOOMIE_INITIAL_TIMER_MIN: f32 = 30.0;
const ZOOMIE_INITIAL_TIMER_MAX: f32 = 90.0;
const DIRECTION_TIMER_DISPLAY_DEFAULT: u32 = 300;
const ZOOMIE_TIMER_DISPLAY_DEFAULT: f32 = 60.0;

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

#[derive(Clone)]
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
    pub mutations: Option<Box<MutationRecord>>,
    pub weight_g: u32,
    pub size_category: SizeCategory,
    pub devil_marked: bool,
    pub unfish_state: Option<Box<UnfishState>>,
    pub sell_price_bonus_pct: u8,
    pub abduction_lock: bool,
    direction_timer: u32,
    zoomie_timer: f32,
}

fn roll_size_category(rng: &mut impl RngExt) -> SizeCategory {
    const WEIGHTS: [(u32, SizeCategory); 4] = [
        (55, SizeCategory::S),
        (30, SizeCategory::M),
        (12, SizeCategory::L),
        (3, SizeCategory::XL),
    ];
    let mut v = rng.random_range(0u32..100);
    for (w, cat) in WEIGHTS {
        if v < w {
            return cat;
        }
        v -= w;
    }
    SizeCategory::XL
}

struct BodyFields {
    body_size: usize,
    weight_g: u32,
    speed: f32,
    pattern_seed: u64,
    color: Color,
    mutant: Option<Box<MutantState>>,
    display_width: usize,
    sway_speed: f32,
}

fn init_body_fields(
    species: FishSpecies,
    size_cat: SizeCategory,
    rng: &mut impl RngExt,
) -> BodyFields {
    let config = species.config();
    let body_size = match config.body {
        BodyTemplate::Fixed { .. } => 0,
        _ => config.sizes[size_cat as usize],
    };
    let weight_g = config.weight_base[size_cat as usize];
    let speed = rng.random_range(config.speed_range.0..config.speed_range.1);
    let pattern_seed: u64 = rng.random();
    let color = if species == FishSpecies::Mutantfish {
        FishSpecies::mutant_color_for_seed(pattern_seed)
    } else {
        config.palette[rng.random_range(0..config.palette.len())]
    };
    let mutant = if species == FishSpecies::Mutantfish {
        Some(Box::new(MutantState::new(body_size, pattern_seed, rng)))
    } else {
        None
    };
    let display_width = mutant
        .as_ref()
        .map(|mutant| mutant.display_width(body_size))
        .unwrap_or_else(|| compute_display_width(species, body_size));
    BodyFields {
        body_size,
        weight_g,
        speed,
        pattern_seed,
        color,
        mutant,
        display_width,
        sway_speed: config.sway_speed,
    }
}

impl Fish {
    pub fn new(species: FishSpecies, name: String, x: f32, y: f32, rng: &mut impl RngExt) -> Self {
        let size_cat = if species == FishSpecies::Mutantfish {
            SizeCategory::M
        } else {
            roll_size_category(rng)
        };
        let fields = init_body_fields(species, size_cat, rng);
        let angle = rng.random::<f32>() * TAU;
        let dx = angle.cos() * fields.speed;
        let dy = angle.sin() * fields.speed * DY_FRACTION;
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
            body_size: fields.body_size,
            color: fields.color,
            speed: fields.speed,
            seek_boost: 0.0,
            direction_timer: rng.random_range(DIRECTION_TIMER_MIN..DIRECTION_TIMER_MAX),
            zoomie_timer: rng.random_range(ZOOMIE_INITIAL_TIMER_MIN..ZOOMIE_INITIAL_TIMER_MAX),
            species,
            pattern_seed: fields.pattern_seed,
            display_width: fields.display_width,
            sway_speed: fields.sway_speed,
            mutant: fields.mutant,
            mutations: None,
            weight_g: fields.weight_g,
            size_category: size_cat,
            devil_marked: false,
            unfish_state: None,
            sell_price_bonus_pct: 0,
            abduction_lock: false,
        }
    }

    pub fn new_for_display(species: FishSpecies, rng: &mut impl RngExt) -> Self {
        let fields = init_body_fields(species, SizeCategory::M, rng);
        Self {
            name: String::new(),
            position: Position { x: 0.0, y: 0.0 },
            velocity: Velocity {
                dx: fields.speed,
                dy: 0.0,
            },
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            state: FishState::Idle,
            facing: Direction::Right,
            body_size: fields.body_size,
            color: fields.color,
            speed: fields.speed,
            seek_boost: 0.0,
            direction_timer: DIRECTION_TIMER_DISPLAY_DEFAULT,
            zoomie_timer: ZOOMIE_TIMER_DISPLAY_DEFAULT,
            species,
            pattern_seed: fields.pattern_seed,
            display_width: fields.display_width,
            sway_speed: fields.sway_speed,
            mutant: fields.mutant,
            mutations: None,
            weight_g: fields.weight_g,
            size_category: SizeCategory::M,
            devil_marked: false,
            unfish_state: None,
            sell_price_bonus_pct: 0,
            abduction_lock: false,
        }
    }

    pub fn new_unfish(
        kind: UnfishKind,
        name: String,
        x: f32,
        y: f32,
        rng: &mut impl RngExt,
    ) -> Self {
        let (speed_lo, speed_hi) = kind.speed_range();
        let speed = rng.random_range(speed_lo..speed_hi);
        let display_width = match kind {
            UnfishKind::Skull => SKULL_WIDTH as usize,
            UnfishKind::Ball => BALL_WIDTH as usize,
            UnfishKind::Worm => worm_display_width(WORM_DEFAULT_SEGMENTS, 0, false),
            _ => compute_display_width(FishSpecies::Unfish, 5),
        };
        let angle = rng.random::<f32>() * TAU;
        let dx = angle.cos() * speed;
        let dy = angle.sin() * speed * kind.dy_fraction();
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
            body_size: 5,
            color: WHITE,
            speed,
            seek_boost: 0.0,
            direction_timer: rng.random_range(DIRECTION_TIMER_MIN..DIRECTION_TIMER_MAX),
            zoomie_timer: rng.random_range(ZOOMIE_INITIAL_TIMER_MIN..ZOOMIE_INITIAL_TIMER_MAX),
            species: FishSpecies::Unfish,
            pattern_seed: rng.random(),
            display_width,
            sway_speed: FishSpecies::Unfish.config().sway_speed,
            mutant: None,
            mutations: None,
            weight_g: 1,
            size_category: SizeCategory::M,
            devil_marked: false,
            unfish_state: Some(Box::new(UnfishState::new(kind, rng))),
            sell_price_bonus_pct: 0,
            abduction_lock: false,
        }
    }

    pub fn is_invisible(&self) -> bool {
        self.unfish_state
            .as_ref()
            .is_some_and(|us| us.is_invisible())
    }

    pub fn is_weight_uncapped(&self) -> bool {
        self.unfish_state.is_some()
    }

    pub fn segments(&self) -> Vec<(char, Color)> {
        if self.unfish_state.is_some() {
            return self.segments_unfish();
        }
        if self.mutant.is_some() {
            return self.segments_mutant();
        }
        let config = self.species.config();
        let chars = self.build_chars(&config.body);
        let colors = if matches!(config.pattern, PatternKind::Glistening) {
            self.build_glistening_colors(
                chars.len(),
                config.palette[0],
                config.palette[1],
                config.palette[2],
            )
        } else {
            self.build_colors(chars.len(), config.palette, config.pattern)
        };
        let mut segs: Vec<(char, Color)> = chars.into_iter().zip(colors).collect();
        if !matches!(config.body, BodyTemplate::Fixed { .. })
            && matches!(self.facing, Direction::Right)
        {
            segs.reverse();
        }
        if !matches!(config.body, BodyTemplate::Fixed { .. })
            && segs.len() >= 2
            && self.unfish_state.is_some()
        {
            let eye_idx = if matches!(self.facing, Direction::Right) {
                segs.len() - 2
            } else {
                1
            };
            segs[eye_idx].1 = DARK_GRAY;
        }
        segs
    }

    fn build_chars(&self, body: &BodyTemplate) -> Vec<char> {
        match *body {
            BodyTemplate::Standard(body_chars) => self.build_standard_chars(body_chars),
            BodyTemplate::Alternating(even_body_chars, odd_body_chars) => {
                let body_chars = if self.pattern_seed.is_multiple_of(2) {
                    even_body_chars
                } else {
                    odd_body_chars
                };
                self.build_standard_chars(body_chars)
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

    fn build_standard_chars(&self, body_chars: BodyChars) -> Vec<char> {
        self.build_standard_chars_for(body_chars, self.facing)
    }

    fn build_standard_chars_for(&self, body_chars: BodyChars, facing: Direction) -> Vec<char> {
        let (body_char, wave_char, raw_mouth, eye) = match facing {
            Direction::Left => (
                body_chars.body_left,
                body_chars.wave_left,
                body_chars.mouth_left,
                body_chars.eye_left,
            ),
            Direction::Right => (
                body_chars.body_right,
                body_chars.wave_right,
                body_chars.mouth_right,
                body_chars.eye_right,
            ),
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
        let tail = tail_chars(body_chars, facing, self.sway.phase);
        let mut chars = vec![mouth, eye];
        chars.extend(body);
        chars.extend(tail);
        chars
    }

    fn segments_unfish(&self) -> Vec<(char, Color)> {
        let unfish_state = self.unfish_state.as_ref().unwrap();
        if is_multi_row(unfish_state.kind) {
            return vec![];
        }
        let config = FishSpecies::Unfish.config();
        let body_chars = match config.body {
            BodyTemplate::Standard(b) => b,
            _ => unreachable!(),
        };
        match unfish_state.kind {
            UnfishKind::Worm => {
                let facing_left = matches!(self.facing, Direction::Left);
                let sprite = build_worm(
                    facing_left,
                    unfish_state.worm_forward,
                    unfish_state.worm_segments,
                    unfish_state.worm_extra_eyes,
                    unfish_state.worm_is_double,
                );
                let eye_cols = worm_eye_cols(
                    facing_left,
                    unfish_state.worm_segments,
                    unfish_state.worm_extra_eyes,
                    unfish_state.worm_is_double,
                );
                let body_color = unfish_state.slime_body_color.unwrap_or(UNFISH_BODY_COLOR);
                let n = sprite.chars().count();
                let glisten_colors: Vec<Color> = if unfish_state.slime_glisten_enabled {
                    let (base, mid, peak_default) = derive_glistening_palette(body_color);
                    let peak = unfish_state.slime_glisten_color.unwrap_or(peak_default);
                    (0..n)
                        .map(|i| {
                            color_for_glisten(
                                unfish_state.slime_glisten_mode,
                                unfish_state.slime_glisten_phase,
                                i,
                                n,
                                base,
                                mid,
                                peak,
                            )
                        })
                        .collect()
                } else {
                    let mut colors = vec![body_color; n];
                    for &(pos, color) in &unfish_state.slime_color_patches {
                        if pos < n && !eye_cols.contains(&pos) {
                            colors[pos] = color;
                        }
                    }
                    colors
                };
                sprite
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        if eye_cols.contains(&i) {
                            let ch = if unfish_state.eye.is_open { '0' } else { '-' };
                            (ch, unfish_state.slime_eye_color.unwrap_or(UNFISH_EYE_COLOR))
                        } else {
                            (c, glisten_colors[i])
                        }
                    })
                    .collect()
            }
            UnfishKind::Reversed => {
                let opposite = match self.facing {
                    Direction::Left => Direction::Right,
                    Direction::Right => Direction::Left,
                };
                let chars = self.build_standard_chars_for(body_chars, opposite);
                let colors = vec![UNFISH_BODY_COLOR; chars.len()];
                let mut segs: Vec<(char, Color)> = chars.into_iter().zip(colors).collect();
                if matches!(self.facing, Direction::Left) {
                    segs.reverse();
                }
                let n = segs.len();
                for &(pos, color) in &unfish_state.slime_color_patches {
                    if pos < n {
                        segs[pos].1 = color;
                    }
                }
                if n >= 2 {
                    let eye_idx = if matches!(self.facing, Direction::Left) {
                        n - 2
                    } else {
                        1
                    };
                    segs[eye_idx].1 = UNFISH_EYE_COLOR;
                }
                segs
            }
            UnfishKind::Blinker => {
                let chars = self.build_standard_chars(body_chars);
                let n = chars.len();
                let colors: Vec<Color> = (0..n)
                    .map(|i| {
                        let s = (unfish_state.glistening_phase - i as f32 * CHAR_SPREAD).sin();
                        if s > BLINKER_GLISTEN_PEAK {
                            BLINKER_PEAK_COLOR
                        } else if s > BLINKER_GLISTEN_MID {
                            BLINKER_MID_COLOR
                        } else {
                            BLINKER_BASE_COLOR
                        }
                    })
                    .collect();
                let mut segs: Vec<(char, Color)> = chars.into_iter().zip(colors).collect();
                if matches!(self.facing, Direction::Right) {
                    segs.reverse();
                }
                for &(pos, color) in &unfish_state.slime_color_patches {
                    if pos < n {
                        segs[pos].1 = color;
                    }
                }
                segs
            }
            _ => {
                let chars = self.build_standard_chars(body_chars);
                let colors = vec![UNFISH_BODY_COLOR; chars.len()];
                let mut segs: Vec<(char, Color)> = chars.into_iter().zip(colors).collect();
                if matches!(self.facing, Direction::Right) {
                    segs.reverse();
                }
                let n = segs.len();
                for &(pos, color) in &unfish_state.slime_color_patches {
                    if pos < n {
                        segs[pos].1 = color;
                    }
                }
                if n >= 2 {
                    let eye_idx = if matches!(self.facing, Direction::Right) {
                        n - 2
                    } else {
                        1
                    };
                    segs[eye_idx].1 = UNFISH_EYE_COLOR;
                }
                segs
            }
        }
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
                color_for_glisten(
                    GlisteningMode::Wave,
                    self.sway.phase,
                    i,
                    len,
                    base,
                    mid,
                    peak,
                )
            })
            .collect()
    }

    fn segments_mutant(&self) -> Vec<(char, Color)> {
        let mutant = self.mutant.as_ref().unwrap();

        if let BodyTemplate::Fixed { left, right } = self.species.config().body {
            let facing_left_fixed = matches!(self.facing, Direction::Left);
            let idx = self.pattern_seed as usize % left.len();
            let chars_left: Vec<char> = left[idx].chars().collect();

            if chars_left.len() <= 1 {
                let jelly_char = chars_left.first().copied().unwrap_or(' ');
                let n = if mutant.is_double { 2 } else { 1 };
                if mutant.glistening_color.is_some() {
                    let (base, mid, peak_default) = derive_glistening_palette(self.color);
                    let peak = mutant.glistening_color.unwrap_or(peak_default);
                    return (0..n)
                        .map(|i| {
                            let c = color_for_glisten(
                                mutant.glistening_mode,
                                self.sway.phase,
                                i,
                                n,
                                base,
                                mid,
                                peak,
                            );
                            (jelly_char, c)
                        })
                        .collect();
                } else {
                    return (0..n).map(|_| (jelly_char, self.color)).collect();
                }
            }

            let _ = right;
            let mouth_ch = if facing_left_fixed {
                chars_left[0]
            } else {
                invert_mouth(chars_left[0])
            };
            let body_ch = if facing_left_fixed {
                chars_left[1]
            } else {
                invert_mouth(chars_left[1])
            };
            let tail_ch = if facing_left_fixed {
                *chars_left.last().unwrap()
            } else {
                invert_mouth(*chars_left.last().unwrap())
            };

            let eye_chars: Vec<char> = if facing_left_fixed {
                mutant.left_eyes.iter().map(|e| e.small_char()).collect()
            } else {
                mutant.right_eyes.iter().map(|e| e.big_char()).collect()
            };
            let eye_start = 1usize;
            let eye_count = eye_chars.len();

            let mut out_chars: Vec<char> = vec![mouth_ch];
            out_chars.extend(eye_chars.iter().copied());
            for _ in 0..self.body_size {
                out_chars.push(body_ch);
            }

            let (double_eye_start, double_eye_count) = if mutant.is_double {
                let mirror_body = invert_mouth(body_ch);
                let mirror_mouth = invert_mouth(mouth_ch);
                for _ in 0..self.body_size {
                    out_chars.push(mirror_body);
                }
                let d_start = out_chars.len();
                let double_eyes: Vec<char> = mutant
                    .double_head_eyes
                    .iter()
                    .map(|e| {
                        if facing_left_fixed {
                            e.big_char()
                        } else {
                            e.small_char()
                        }
                    })
                    .collect();
                let d_count = double_eyes.len();
                out_chars.extend(double_eyes);
                out_chars.push(mirror_mouth);
                (d_start, d_count)
            } else {
                out_chars.push(tail_ch);
                (0usize, 0usize)
            };

            let n = out_chars.len();
            let use_glisten = mutant.glistening_color.is_some();
            let mut colors: Vec<Color> = if use_glisten {
                let (base, mid, peak_default) = derive_glistening_palette(self.color);
                let peak = mutant.glistening_color.unwrap_or(peak_default);
                (0..n)
                    .map(|i| {
                        color_for_glisten(
                            mutant.glistening_mode,
                            self.sway.phase,
                            i,
                            n,
                            base,
                            mid,
                            peak,
                        )
                    })
                    .collect()
            } else {
                vec![self.color; n]
            };
            apply_patches_and_eyes(
                &mut colors,
                &mutant.color_patches,
                mutant.eye_color,
                eye_start,
                eye_count,
                double_eye_start,
                double_eye_count,
            );
            let mut segs: Vec<(char, Color)> = out_chars.into_iter().zip(colors).collect();
            if !facing_left_fixed {
                segs.reverse();
            }
            return segs;
        }

        let facing_left = matches!(self.facing, Direction::Left);
        let eating = matches!(self.state, FishState::Eating { .. });

        let (mouth, body_ch, wave_ch, non_double_tail): (char, char, char, Vec<char>) =
            if self.species == FishSpecies::Mutantfish {
                let (raw_mouth, bc, wc) =
                    body_chars_for_variant(mutant.body_variant, facing_left, mutant.mouth_inverted);
                let mo = if eating {
                    invert_mouth(raw_mouth)
                } else {
                    raw_mouth
                };
                (
                    mo,
                    bc,
                    wc,
                    mutant.tail_variant.chars(facing_left, self.sway.phase),
                )
            } else {
                let config = self.species.config();
                let body_chars = match config.body {
                    BodyTemplate::Standard(b) => b,
                    _ => unreachable!(),
                };
                let (body_char, wave_char, raw_mouth) = if facing_left {
                    (
                        body_chars.body_left,
                        body_chars.wave_left,
                        body_chars.mouth_left,
                    )
                } else {
                    (
                        body_chars.body_right,
                        body_chars.wave_right,
                        body_chars.mouth_right,
                    )
                };
                let pre_eat = if mutant.mouth_inverted {
                    invert_mouth(raw_mouth)
                } else {
                    raw_mouth
                };
                let mo = if eating {
                    invert_mouth(pre_eat)
                } else {
                    pre_eat
                };
                (
                    mo,
                    body_char,
                    wave_char,
                    tail_chars(body_chars, self.facing, self.sway.phase),
                )
            };

        let sway_high = self.sway.phase.sin() > WAVE_THRESHOLD;
        let max_eyes = mutant.left_eyes.len().max(mutant.right_eyes.len());

        let mut chars: Vec<char> = Vec::new();
        chars.push(mouth);

        let eye_start = chars.len();
        let eye_count = if facing_left {
            for e in &mutant.left_eyes {
                chars.push(e.small_char());
            }
            mutant.left_eyes.len()
        } else {
            for e in &mutant.right_eyes {
                chars.push(e.big_char());
            }
            mutant.right_eyes.len()
        };
        let extra_body = max_eyes - eye_count;

        for _ in 0..(self.body_size + extra_body) {
            chars.push(if sway_high { wave_ch } else { body_ch });
        }

        let double_eye_start;
        let double_eye_count;

        if mutant.is_double {
            for _ in 0..EXTRA_BODY_FOR_DOUBLE {
                chars.push(if sway_high { wave_ch } else { body_ch });
            }
            double_eye_start = chars.len();
            for e in &mutant.double_head_eyes {
                chars.push(if facing_left {
                    e.big_char()
                } else {
                    e.small_char()
                });
            }
            double_eye_count = mutant.double_head_eyes.len();
            chars.push(if facing_left { '>' } else { '<' });
        } else {
            chars.extend(non_double_tail);
            double_eye_start = 0;
            double_eye_count = 0;
        }

        let n = chars.len();
        let use_glisten = mutant.glistening_color.is_some() || self.species.config().auto_glisten;
        let mut colors: Vec<Color> = if use_glisten {
            let (base, mid, peak_default) = derive_glistening_palette(self.color);
            let peak = mutant.glistening_color.unwrap_or(peak_default);
            (0..n)
                .map(|i| {
                    color_for_glisten(
                        mutant.glistening_mode,
                        self.sway.phase,
                        i,
                        n,
                        base,
                        mid,
                        peak,
                    )
                })
                .collect()
        } else {
            vec![self.color; n]
        };
        apply_patches_and_eyes(
            &mut colors,
            &mutant.color_patches,
            mutant.eye_color,
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

    pub fn tick(
        &mut self,
        settings: &Settings,
        tank_width: u16,
        tank_height: u16,
        coffee_stacks: u32,
    ) {
        let dt = 1.0 / settings.fps;

        if self.abduction_lock {
            tick_sway(&mut self.sway, self.sway_speed);
            if let Some(ref mut mutant) = self.mutant {
                mutant.tick_eyes(dt);
            }
            return;
        }

        match self.state {
            FishState::Idle => {
                if self.species.config().can_zoomie {
                    let zoomie_dt = dt * (1.0 + COFFEE_ZOOMIE_DT_MULT * coffee_stacks as f32);
                    self.zoomie_timer -= zoomie_dt;
                    if self.zoomie_timer <= 0.0 {
                        self.start_zoomie();
                    } else {
                        self.direction_timer = self.direction_timer.saturating_sub(1);
                        if self.direction_timer == 0 {
                            self.randomize_direction();
                        }
                    }
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
            let speed_mult = 1.0 + COFFEE_SPEED_MULT * coffee_stacks as f32;
            self.position.x += self.velocity.dx * dt * speed_mult;
            self.position.y += self.velocity.dy * dt * speed_mult;
            self.bounce_walls(tank_width, tank_height);
        }

        let sway_mult = 1.0 + COFFEE_SWAY_MULT * coffee_stacks as f32;
        let effective_sway_speed = if matches!(self.state, FishState::Zoomie { .. }) {
            self.sway_speed * ZOOMIE_SWAY_MULTIPLIER * sway_mult
        } else {
            self.sway_speed * sway_mult
        };
        tick_sway(&mut self.sway, effective_sway_speed);

        if let Some(ref mut mutant) = self.mutant {
            mutant.tick_eyes(dt);
        }
        if let Some(ref mut us) = self.unfish_state {
            let mut rng = rand::rng();
            us.tick(dt, &mut rng);
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
        let dy_frac = self
            .unfish_state
            .as_ref()
            .map_or(DY_FRACTION, |us| us.kind.dy_fraction());
        self.velocity.dx = angle.cos() * self.speed;
        self.velocity.dy = angle.sin() * self.speed * dy_frac;

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

        let will_turn = if self.species.config().zoomie_vertical {
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

    pub fn tick_animation(&mut self, dt: f32) {
        tick_sway(&mut self.sway, self.sway_speed);
        if let Some(ref mut mutant) = self.mutant {
            mutant.tick_eyes(dt);
        }
        if let Some(ref mut us) = self.unfish_state {
            let mut rng = rand::rng();
            us.tick(dt, &mut rng);
        }
    }

    pub fn static_left_segments(&self) -> Vec<(char, Color)> {
        if self.unfish_state.is_some() {
            return self.static_left_segments_unfish();
        }
        if self.mutant.is_some() {
            return self.static_left_segments_mutant();
        }
        let config = self.species.config();
        match config.body {
            BodyTemplate::Fixed { left, .. } => {
                let idx = self.pattern_seed as usize % left.len();
                let chars: Vec<char> = left[idx].chars().collect();
                let len = chars.len();
                let colors = self.build_colors(len, config.palette, config.pattern);
                chars.into_iter().zip(colors).collect()
            }
            BodyTemplate::Standard(bc) | BodyTemplate::Alternating(bc, _) => {
                let body_chars = match config.body {
                    BodyTemplate::Alternating(even_body_chars, odd_body_chars) => {
                        if self.pattern_seed.is_multiple_of(2) {
                            even_body_chars
                        } else {
                            odd_body_chars
                        }
                    }
                    _ => bc,
                };
                let mut chars = vec![body_chars.mouth_left, body_chars.eye_left];
                chars.extend(std::iter::repeat_n(body_chars.body_left, self.body_size));
                let tail: Vec<char> = match body_chars.tail {
                    TailKind::Wide => vec!['>', '<'],
                    TailKind::WideCurly => vec!['>', '<', '{'],
                    TailKind::Short => vec!['<'],
                    TailKind::Custom { left, .. } => vec![left],
                    TailKind::Swaying { left, .. } => vec![left],
                    TailKind::None => vec![],
                };
                chars.extend(tail);
                let len = chars.len();
                let colors = if matches!(config.pattern, PatternKind::Glistening) {
                    self.build_glistening_colors(
                        len,
                        config.palette[0],
                        config.palette[1],
                        config.palette[2],
                    )
                } else {
                    self.build_colors(len, config.palette, config.pattern)
                };
                chars.into_iter().zip(colors).collect()
            }
        }
    }

    fn static_left_segments_unfish(&self) -> Vec<(char, Color)> {
        let unfish_state = self.unfish_state.as_ref().unwrap();
        if is_multi_row(unfish_state.kind) {
            return vec![];
        }
        let config = FishSpecies::Unfish.config();
        let body_chars = match config.body {
            BodyTemplate::Standard(b) => b,
            _ => unreachable!(),
        };
        match unfish_state.kind {
            UnfishKind::Worm => {
                let sprite = build_worm(
                    true,
                    true,
                    unfish_state.worm_segments,
                    unfish_state.worm_extra_eyes,
                    unfish_state.worm_is_double,
                );
                let eye_cols = worm_eye_cols(
                    true,
                    unfish_state.worm_segments,
                    unfish_state.worm_extra_eyes,
                    unfish_state.worm_is_double,
                );
                let body_color = unfish_state.slime_body_color.unwrap_or(UNFISH_BODY_COLOR);
                let mut segs: Vec<(char, Color)> = sprite
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        if eye_cols.contains(&i) {
                            (
                                '0',
                                unfish_state.slime_eye_color.unwrap_or(UNFISH_EYE_COLOR),
                            )
                        } else {
                            (c, body_color)
                        }
                    })
                    .collect();
                for &(pos, color) in &unfish_state.slime_color_patches {
                    if pos < segs.len() && !eye_cols.contains(&pos) {
                        segs[pos].1 = color;
                    }
                }
                segs
            }
            UnfishKind::Reversed => {
                let chars = self.build_standard_chars_for(body_chars, Direction::Right);
                let mut segs: Vec<(char, Color)> =
                    chars.into_iter().map(|c| (c, UNFISH_BODY_COLOR)).collect();
                segs.reverse();
                let n = segs.len();
                for &(pos, color) in &unfish_state.slime_color_patches {
                    if pos < n {
                        segs[pos].1 = color;
                    }
                }
                if n >= 2 {
                    segs[n - 2].1 = UNFISH_EYE_COLOR;
                }
                segs
            }
            UnfishKind::Blinker => {
                let chars = self.build_standard_chars_for(body_chars, Direction::Left);
                let mut segs: Vec<(char, Color)> =
                    chars.into_iter().map(|c| (c, BLINKER_BASE_COLOR)).collect();
                let n = segs.len();
                for &(pos, color) in &unfish_state.slime_color_patches {
                    if pos < n {
                        segs[pos].1 = color;
                    }
                }
                if n >= 2 {
                    segs[1].1 = UNFISH_EYE_COLOR;
                }
                segs
            }
            _ => {
                let chars = self.build_standard_chars_for(body_chars, Direction::Left);
                let mut segs: Vec<(char, Color)> =
                    chars.into_iter().map(|c| (c, UNFISH_BODY_COLOR)).collect();
                let n = segs.len();
                for &(pos, color) in &unfish_state.slime_color_patches {
                    if pos < n {
                        segs[pos].1 = color;
                    }
                }
                if n >= 2 {
                    segs[1].1 = UNFISH_EYE_COLOR;
                }
                segs
            }
        }
    }

    fn static_left_segments_mutant(&self) -> Vec<(char, Color)> {
        let mutant = self.mutant.as_ref().unwrap();

        if let BodyTemplate::Fixed { left, .. } = self.species.config().body {
            let idx = self.pattern_seed as usize % left.len();
            let chars_left: Vec<char> = left[idx].chars().collect();

            if chars_left.len() <= 1 {
                let jelly_char = chars_left.first().copied().unwrap_or(' ');
                let n = if mutant.is_double { 2 } else { 1 };
                if mutant.glistening_color.is_some() {
                    let (base, mid, peak_default) = derive_glistening_palette(self.color);
                    let peak = mutant.glistening_color.unwrap_or(peak_default);
                    return (0..n)
                        .map(|i| {
                            let c = color_for_glisten(
                                mutant.glistening_mode,
                                0.0,
                                i,
                                n,
                                base,
                                mid,
                                peak,
                            );
                            (jelly_char, c)
                        })
                        .collect();
                } else {
                    return (0..n).map(|_| (jelly_char, self.color)).collect();
                }
            }

            let mouth_ch = chars_left[0];
            let body_ch = chars_left[1];
            let tail_ch = *chars_left.last().unwrap();

            let eye_start = 1usize;
            let mut out_chars: Vec<char> = vec![mouth_ch];
            for e in &mutant.left_eyes {
                out_chars.push(e.small_char());
            }
            let eye_count = mutant.left_eyes.len();
            for _ in 0..self.body_size {
                out_chars.push(body_ch);
            }

            let (double_eye_start, double_eye_count) = if mutant.is_double {
                let mirror_body = invert_mouth(body_ch);
                let mirror_mouth = invert_mouth(mouth_ch);
                for _ in 0..self.body_size {
                    out_chars.push(mirror_body);
                }
                let d_start = out_chars.len();
                for e in &mutant.double_head_eyes {
                    out_chars.push(e.big_char());
                }
                let d_count = mutant.double_head_eyes.len();
                out_chars.push(mirror_mouth);
                (d_start, d_count)
            } else {
                out_chars.push(tail_ch);
                (0usize, 0usize)
            };

            let n = out_chars.len();
            let use_glisten = mutant.glistening_color.is_some();
            let mut colors: Vec<Color> = if use_glisten {
                let (base, mid, peak_default) = derive_glistening_palette(self.color);
                let peak = mutant.glistening_color.unwrap_or(peak_default);
                (0..n)
                    .map(|i| color_for_glisten(mutant.glistening_mode, 0.0, i, n, base, mid, peak))
                    .collect()
            } else {
                vec![self.color; n]
            };
            apply_patches_and_eyes(
                &mut colors,
                &mutant.color_patches,
                mutant.eye_color,
                eye_start,
                eye_count,
                double_eye_start,
                double_eye_count,
            );
            return out_chars.into_iter().zip(colors).collect();
        }

        let (raw_mouth, body_ch, non_double_tail): (char, char, Vec<char>) =
            if self.species == FishSpecies::Mutantfish {
                let (rm, bc, _) =
                    body_chars_for_variant(mutant.body_variant, true, mutant.mouth_inverted);
                (rm, bc, mutant.tail_variant.chars(true, 0.0))
            } else {
                let config = self.species.config();
                let body_chars = match config.body {
                    BodyTemplate::Standard(b) => b,
                    _ => unreachable!(),
                };
                let rm = if mutant.mouth_inverted {
                    invert_mouth(body_chars.mouth_left)
                } else {
                    body_chars.mouth_left
                };
                (
                    rm,
                    body_chars.body_left,
                    tail_chars(body_chars, Direction::Left, 0.0),
                )
            };
        let max_eyes = mutant.left_eyes.len().max(mutant.right_eyes.len());
        let mut chars: Vec<char> = Vec::new();
        chars.push(raw_mouth);
        let eye_start = chars.len();
        for e in &mutant.left_eyes {
            chars.push(e.small_char());
        }
        let eye_count = mutant.left_eyes.len();
        let extra_body = max_eyes - eye_count;
        for _ in 0..(self.body_size + extra_body) {
            chars.push(body_ch);
        }
        let (double_eye_start, double_eye_count) = if mutant.is_double {
            for _ in 0..EXTRA_BODY_FOR_DOUBLE {
                chars.push(body_ch);
            }
            let start = chars.len();
            for e in &mutant.double_head_eyes {
                chars.push(e.big_char());
            }
            let count = mutant.double_head_eyes.len();
            chars.push('>');
            (start, count)
        } else {
            chars.extend(non_double_tail);
            (0, 0)
        };
        let n = chars.len();
        let use_glisten = mutant.glistening_color.is_some() || self.species.config().auto_glisten;
        let mut colors: Vec<Color> = if use_glisten {
            let (base, mid, peak_default) = derive_glistening_palette(self.color);
            let peak = mutant.glistening_color.unwrap_or(peak_default);
            (0..n)
                .map(|i| color_for_glisten(mutant.glistening_mode, 0.0, i, n, base, mid, peak))
                .collect()
        } else {
            vec![self.color; n]
        };
        apply_patches_and_eyes(
            &mut colors,
            &mutant.color_patches,
            mutant.eye_color,
            eye_start,
            eye_count,
            double_eye_start,
            double_eye_count,
        );
        chars.into_iter().zip(colors).collect()
    }

    fn sprite_y_margins(&self) -> (f32, f32) {
        if self.species != FishSpecies::Unfish {
            return (0.0, 0.0);
        }
        match self.unfish_state.as_ref().map(|us| us.kind) {
            Some(UnfishKind::Skull) => (3.0, 2.0),
            Some(UnfishKind::Ball) => (4.0, 4.0),
            _ => (0.0, 0.0),
        }
    }

    fn bounce_walls(&mut self, tank_width: u16, tank_height: u16) {
        if let Some(ref us) = self.unfish_state
            && us.kind == UnfishKind::Worm
        {
            self.position.x = self.position.x.rem_euclid(tank_width as f32);
            self.position.y = self.position.y.rem_euclid(tank_height as f32);
            return;
        }
        let max_x = tank_width as f32 - self.display_width as f32;
        let max_y = tank_height as f32 - 1.0;
        let (top_margin, bottom_margin) = self.sprite_y_margins();

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

        if self.position.y < top_margin {
            self.position.y = top_margin;
            self.velocity.dy = self.velocity.dy.abs();
        } else if self.position.y > max_y - bottom_margin {
            self.position.y = max_y - bottom_margin;
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

fn tail_chars(body_chars: BodyChars, facing: Direction, phase: f32) -> Vec<char> {
    match body_chars.tail {
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
    let config = species.config();
    match config.body {
        BodyTemplate::Standard(body_chars) | BodyTemplate::Alternating(body_chars, _) => {
            let tail_w = match body_chars.tail {
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
