use rand::RngExt;
use ratatui::style::Color;
use std::f32::consts::TAU;

use crate::entities::mutant::GlisteningMode;
use crate::util::sample_exponential;

pub const VOID_SPAWN_MEAN_SECS: f32 = 3600.0;

#[derive(Clone)]
pub struct BlinkTimer {
    pub is_open: bool,
    timer: f32,
    open_secs: f32,
    closed_secs: f32,
}

const EYE_OPEN_MIN: f32 = 0.5;
const EYE_OPEN_MAX: f32 = 1.0;
const EYE_CLOSED_MIN: f32 = 0.1;
const EYE_CLOSED_MAX: f32 = 0.35;
const WING_OPEN_MIN: f32 = 1.5;
const WING_OPEN_MAX: f32 = 2.0;
const WING_CLOSED_MIN: f32 = 0.15;
const WING_CLOSED_MAX: f32 = 0.4;

impl BlinkTimer {
    pub fn new_eye(rng: &mut impl RngExt) -> Self {
        Self::new(
            rng,
            EYE_OPEN_MIN,
            EYE_OPEN_MAX,
            EYE_CLOSED_MIN,
            EYE_CLOSED_MAX,
        )
    }

    pub fn new_wing(rng: &mut impl RngExt) -> Self {
        Self::new(
            rng,
            WING_OPEN_MIN,
            WING_OPEN_MAX,
            WING_CLOSED_MIN,
            WING_CLOSED_MAX,
        )
    }

    fn new(
        rng: &mut impl RngExt,
        open_min: f32,
        open_max: f32,
        closed_min: f32,
        closed_max: f32,
    ) -> Self {
        let open_secs = rng.random_range(open_min..open_max);
        let closed_secs = rng.random_range(closed_min..closed_max);
        let timer = rng.random_range(0.0_f32..open_secs);
        Self {
            is_open: true,
            timer,
            open_secs,
            closed_secs,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.timer += dt;
        if self.is_open {
            if self.timer >= self.open_secs {
                self.timer -= self.open_secs;
                self.is_open = false;
            }
        } else if self.timer >= self.closed_secs {
            self.timer -= self.closed_secs;
            self.is_open = true;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnfishKind {
    Reversed,
    Doppleganger,
    Phantom,
    Blinker,
    Ball,
    Skull,
    Worm,
}

pub const SPAWNABLE_UNFISH: &[UnfishKind] = &[
    UnfishKind::Reversed,
    UnfishKind::Doppleganger,
    UnfishKind::Phantom,
    UnfishKind::Blinker,
    UnfishKind::Ball,
    UnfishKind::Skull,
    UnfishKind::Worm,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlinkerPhase {
    Glistening,
    Invisible,
}

const BLINKER_GLISTENING_MIN: f32 = 4.0;
const BLINKER_GLISTENING_MAX: f32 = 6.4;
const BLINKER_INVISIBLE_MIN: f32 = 4.0;
const BLINKER_INVISIBLE_MAX: f32 = 6.4;

pub const PHANTOM_TELEPORT_MEAN: f32 = 5.0;
pub const PHANTOM_CROSS_TANK_CHANCE: f32 = 0.05;

const WORM_SWAY_INTERVAL: f32 = 0.25;

const EYE_SPEED_MIN: f32 = 0.425;
const EYE_SPEED_MAX: f32 = 2.125;
const EYE_DY_FRACTION: f32 = 0.4;
const EYE_DIR_TIMER_MIN: f32 = 1.5;
const EYE_DIR_TIMER_MAX: f32 = 5.0;
const BALL_CENTER_EYE_CHANCE: f32 = 0.99;
const BALL_MAX_EXTRA_EYES: usize = 4;
const BALL_FILLED_CHANCE: f32 = 0.05;
const SKULL_MAX_EYES: usize = 4;
const SKULL_FILLED_CHANCE: f32 = 0.05;
const SKULL_NO_EYE_CHANCE: f32 = 0.1;

pub const BALL_WIDTH: u16 = 15;
pub const BALL_CENTER_ROW: i32 = 4;
pub const BALL_EYE_ROW: usize = 4;
pub const BALL_EYE_COL: usize = 7;

pub const BALL_BASE: [&str; 9] = [
    "    ,-----.    ",
    "  ,` `    ,`.  ",
    " /  `     ,  \\ ",
    "; `  `      , :",
    "|   `     `   |",
    ":  ,        \\ ;",
    " \\      ` \\  / ",
    "  `. ,   ` ,`  ",
    "    `-----`    ",
];

pub const BALL_INTERIOR: &[(usize, i32, i32)] = &[
    (1, 3, 11),
    (2, 2, 12),
    (3, 2, 12),
    (4, 2, 12),
    (5, 2, 12),
    (6, 2, 12),
    (7, 3, 11),
];

pub const SKULL_WIDTH: u16 = 13;
pub const SKULL_CENTER_ROW: i32 = 3;

pub const SKULL_OPEN: [&str; 6] = [
    "    .---.    ",
    r"   /     \   ",
    r"}\|       |/{",
    "}(|       |){",
    r"}/ \  ^  / \{",
    r"    \___/    ",
];

pub const SKULL_CLOSED: [&str; 6] = [
    "    .---.    ",
    r"   /     \   ",
    " ||       || ",
    " ||       || ",
    r" | \  ^  / | ",
    r"    \___/    ",
];

pub const SKULL_INTERIOR: &[(usize, i32, i32)] = &[(2, 4, 8), (3, 4, 8), (4, 4, 8)];

pub const WORM_DEFAULT_SEGMENTS: usize = 6;

pub fn build_worm(
    facing_left: bool,
    forward: bool,
    segments: usize,
    extra_eyes: usize,
    is_double: bool,
) -> String {
    let head = "(0)".repeat(1 + extra_eyes);
    if is_double {
        let body = ",/\\".repeat(segments);
        format!("{}{},{}", head, body, head)
    } else if facing_left {
        let body = if forward {
            ",/\\".repeat(segments)
        } else {
            "\\,/".repeat(segments)
        };
        if forward {
            format!("{}{},", head, body)
        } else {
            format!("{},{}", head, body)
        }
    } else {
        let body = if forward {
            ",/\\".repeat(segments)
        } else {
            "\\,/".repeat(segments)
        };
        format!("{},{}", body, head)
    }
}

pub fn worm_eye_cols(
    facing_left: bool,
    segments: usize,
    extra_eyes: usize,
    is_double: bool,
) -> Vec<usize> {
    let head_count = 1 + extra_eyes;
    let mut cols = Vec::new();
    if is_double {
        for k in 0..head_count {
            cols.push(1 + 3 * k);
        }
        let right_start = 3 * head_count + 3 * segments + 1;
        for k in 0..head_count {
            cols.push(right_start + 1 + 3 * k);
        }
    } else if facing_left {
        for k in 0..head_count {
            cols.push(1 + 3 * k);
        }
    } else {
        let head_start = 3 * segments + 1;
        for k in 0..head_count {
            cols.push(head_start + 1 + 3 * k);
        }
    }
    cols
}

pub fn worm_display_width(segments: usize, extra_eyes: usize, is_double: bool) -> usize {
    let head_chars = 3 * (1 + extra_eyes);
    if is_double {
        head_chars + 3 * segments + 1 + head_chars
    } else {
        3 * segments + 1 + head_chars
    }
}

pub const BLINKER_GLISTEN_SPEED: f32 = 24.0;
pub const BLINKER_GLISTEN_PEAK: f32 = 0.6;
pub const BLINKER_GLISTEN_MID: f32 = 0.1;
pub const BLINKER_BASE_COLOR: Color = Color::DarkGray;
pub const BLINKER_MID_COLOR: Color = Color::Gray;
pub const BLINKER_PEAK_COLOR: Color = Color::White;

pub const UNFISH_BODY_COLOR: Color = Color::White;
pub const UNFISH_EYE_COLOR: Color = Color::DarkGray;
pub const BALL_HEIGHT: u16 = 9;
pub const SKULL_HEIGHT: u16 = 6;

pub const SLIME_GLISTEN_SPEED_DEFAULT: f32 = 10.0;
pub const SLIME_GLISTEN_SPEED_FAST: f32 = 22.0;
pub const SLIME_GLISTEN_SPEED_SLOW: f32 = 4.0;

pub fn is_multi_row(kind: UnfishKind) -> bool {
    matches!(kind, UnfishKind::Ball | UnfishKind::Skull)
}

#[derive(Clone)]
pub struct FloatingEye {
    pub row: usize,
    pub row_f: f32,
    pub col: f32,
    pub vel: f32,
    pub vel_y: f32,
    pub blink: BlinkTimer,
    dir_timer: f32,
}

impl FloatingEye {
    fn new(row: usize, col: f32, rng: &mut impl RngExt) -> Self {
        let speed = rng.random_range(EYE_SPEED_MIN..EYE_SPEED_MAX);
        let angle = rng.random::<f32>() * TAU;
        let vel = angle.cos() * speed;
        let vel_y = angle.sin() * speed * EYE_DY_FRACTION;
        Self {
            row,
            row_f: row as f32,
            col,
            vel,
            vel_y,
            blink: BlinkTimer::new_eye(rng),
            dir_timer: rng.random_range(EYE_DIR_TIMER_MIN..EYE_DIR_TIMER_MAX),
        }
    }
}

fn try_place_eye(
    eyes: &mut Vec<FloatingEye>,
    interior: &[(usize, i32, i32)],
    exclude: Option<(usize, f32)>,
    rng: &mut impl RngExt,
) {
    const MAX_ATTEMPTS: usize = 30;
    for _ in 0..MAX_ATTEMPTS {
        let idx = rng.random_range(0..interior.len());
        let (row, min_col, max_col) = interior[idx];
        if min_col >= max_col {
            continue;
        }
        let col = rng.random_range(min_col as f32..=max_col as f32);
        let blocked_ex = exclude.is_some_and(|(er, ec)| row == er && (col - ec).abs() < 3.0);
        let blocked = eyes
            .iter()
            .filter(|e| e.row == row)
            .any(|e| (e.col - col).abs() < 3.0);
        if !blocked_ex && !blocked {
            eyes.push(FloatingEye::new(row, col, rng));
            return;
        }
    }
}

fn fill_eyes(
    eyes: &mut Vec<FloatingEye>,
    interior: &[(usize, i32, i32)],
    exclude: Option<(usize, f32)>,
    rng: &mut impl RngExt,
) {
    for &(row, min_col, max_col) in interior {
        let mut col = min_col as f32;
        while col <= max_col as f32 {
            let blocked_ex = exclude.is_some_and(|(er, ec)| row == er && (col - ec).abs() < 3.0);
            let blocked = eyes
                .iter()
                .filter(|e| e.row == row)
                .any(|e| (e.col - col).abs() < 3.0);
            if !blocked_ex && !blocked {
                eyes.push(FloatingEye::new(row, col, rng));
            }
            col += 3.0;
        }
    }
}

#[derive(Clone)]
pub struct UnfishState {
    pub kind: UnfishKind,
    pub eye: BlinkTimer,
    pub wings: BlinkTimer,
    pub floating_eyes: Vec<FloatingEye>,
    pub ball_has_center_eye: bool,
    pub blinker_phase: BlinkerPhase,
    pub blinker_timer: f32,
    pub blinker_phase_secs: f32,
    pub phantom_timer: f32,
    pub worm_forward: bool,
    worm_timer: f32,
    pub doppleganger_cloned: bool,
    pub glistening_phase: f32,
    pub slime_body_color: Option<Color>,
    pub slime_glisten_enabled: bool,
    pub slime_glisten_color: Option<Color>,
    pub slime_glisten_mode: GlisteningMode,
    pub slime_glisten_phase: f32,
    pub slime_glisten_speed: f32,
    pub worm_segments: usize,
    pub worm_extra_eyes: usize,
    pub worm_is_double: bool,
    pub slime_eye_color: Option<Color>,
    pub slime_color_patches: Vec<(usize, Color)>,
    pub mutation_count: u32,
    pub mutation_history: Vec<String>,
    pub mitosis_partners: Vec<String>,
}

impl UnfishState {
    pub fn new(kind: UnfishKind, rng: &mut impl RngExt) -> Self {
        let mut floating_eyes: Vec<FloatingEye> = Vec::new();
        let mut ball_has_center_eye = false;

        match kind {
            UnfishKind::Ball => {
                ball_has_center_eye = rng.random::<f32>() < BALL_CENTER_EYE_CHANCE;
                let center = if ball_has_center_eye {
                    Some((BALL_EYE_ROW, BALL_EYE_COL as f32))
                } else {
                    None
                };
                if rng.random::<f32>() < BALL_FILLED_CHANCE {
                    fill_eyes(&mut floating_eyes, BALL_INTERIOR, center, rng);
                } else {
                    let n = rng.random_range(0..=BALL_MAX_EXTRA_EYES);
                    for _ in 0..n {
                        try_place_eye(&mut floating_eyes, BALL_INTERIOR, center, rng);
                    }
                }
            }
            UnfishKind::Skull => {
                if rng.random::<f32>() < SKULL_FILLED_CHANCE {
                    fill_eyes(&mut floating_eyes, SKULL_INTERIOR, None, rng);
                } else if rng.random::<f32>() > SKULL_NO_EYE_CHANCE {
                    let n = rng.random_range(1..=SKULL_MAX_EYES);
                    for _ in 0..n {
                        try_place_eye(&mut floating_eyes, SKULL_INTERIOR, None, rng);
                    }
                }
            }
            _ => {}
        }

        let blinker_phase_secs = rng.random_range(BLINKER_GLISTENING_MIN..BLINKER_GLISTENING_MAX);
        let phantom_timer = if kind == UnfishKind::Phantom {
            sample_exponential(rng, PHANTOM_TELEPORT_MEAN)
        } else {
            0.0
        };
        Self {
            kind,
            eye: BlinkTimer::new_eye(rng),
            wings: BlinkTimer::new_wing(rng),
            floating_eyes,
            ball_has_center_eye,
            blinker_phase: BlinkerPhase::Glistening,
            blinker_timer: 0.0,
            blinker_phase_secs,
            phantom_timer,
            worm_forward: true,
            worm_timer: 0.0,
            doppleganger_cloned: false,
            glistening_phase: 0.0,
            slime_body_color: None,
            slime_glisten_enabled: false,
            slime_glisten_color: None,
            slime_glisten_mode: GlisteningMode::Wave,
            slime_glisten_phase: 0.0,
            slime_glisten_speed: SLIME_GLISTEN_SPEED_DEFAULT,
            worm_segments: WORM_DEFAULT_SEGMENTS,
            worm_extra_eyes: 0,
            worm_is_double: false,
            slime_eye_color: None,
            slime_color_patches: Vec::new(),
            mutation_count: 0,
            mutation_history: Vec::new(),
            mitosis_partners: Vec::new(),
        }
    }

    pub fn is_invisible(&self) -> bool {
        self.kind == UnfishKind::Blinker && self.blinker_phase == BlinkerPhase::Invisible
    }

    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt) {
        self.eye.tick(dt);
        self.wings.tick(dt);

        let interior: &[(usize, i32, i32)] = match self.kind {
            UnfishKind::Ball => BALL_INTERIOR,
            UnfishKind::Skull => SKULL_INTERIOR,
            _ => &[],
        };
        for eye in &mut self.floating_eyes {
            eye.blink.tick(dt);
            eye.dir_timer -= dt;
            if eye.dir_timer <= 0.0 {
                let speed = rng.random_range(EYE_SPEED_MIN..EYE_SPEED_MAX);
                let angle = rng.random::<f32>() * TAU;
                eye.vel = angle.cos() * speed;
                eye.vel_y = angle.sin() * speed * EYE_DY_FRACTION;
                eye.dir_timer = rng.random_range(EYE_DIR_TIMER_MIN..EYE_DIR_TIMER_MAX);
            }
        }
        let min_row = interior.iter().map(|&(r, _, _)| r).min().unwrap_or(0);
        let max_row = interior.iter().map(|&(r, _, _)| r).max().unwrap_or(0);
        let proposed: Vec<(usize, f32, f32, f32, f32)> = self
            .floating_eyes
            .iter()
            .map(|eye| {
                let raw_row_f = eye.row_f + eye.vel_y * dt;
                let (new_row_f, new_vel_y) = if raw_row_f < min_row as f32 {
                    (min_row as f32, eye.vel_y.abs())
                } else if raw_row_f > max_row as f32 {
                    (max_row as f32, -eye.vel_y.abs())
                } else {
                    (raw_row_f, eye.vel_y)
                };
                let new_row = new_row_f.round() as usize;
                let (min_col, max_col) = interior
                    .iter()
                    .find(|&&(r, _, _)| r == new_row)
                    .map(|&(_, mn, mx)| (mn, mx))
                    .unwrap_or_else(|| {
                        let mn = interior.iter().map(|&(_, mn, _)| mn).min().unwrap_or(0);
                        let mx = interior.iter().map(|&(_, _, mx)| mx).max().unwrap_or(0);
                        (mn, mx)
                    });
                let raw_col = eye.col + eye.vel * dt;
                let (new_col, new_vel) = if raw_col < min_col as f32 {
                    (min_col as f32, eye.vel.abs())
                } else if raw_col > max_col as f32 {
                    (max_col as f32, -eye.vel.abs())
                } else {
                    (raw_col, eye.vel)
                };
                (new_row, new_row_f, new_col, new_vel, new_vel_y)
            })
            .collect();
        let current: Vec<(usize, f32)> =
            self.floating_eyes.iter().map(|e| (e.row, e.col)).collect();
        let has_center = self.kind == UnfishKind::Ball && self.ball_has_center_eye;
        for (i, &(p_row, p_row_f, p_col, p_vel, p_vel_y)) in proposed.iter().enumerate() {
            let blocked = current
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .any(|(_, &(cr, cc))| cr == p_row && (cc - p_col).abs() < 3.0)
                || (has_center
                    && p_row == BALL_EYE_ROW
                    && (BALL_EYE_COL as f32 - p_col).abs() < 3.0);
            if !blocked {
                self.floating_eyes[i].row = p_row;
                self.floating_eyes[i].row_f = p_row_f;
                self.floating_eyes[i].col = p_col;
                self.floating_eyes[i].vel = p_vel;
                self.floating_eyes[i].vel_y = p_vel_y;
            }
        }

        if self.kind == UnfishKind::Blinker {
            self.glistening_phase = (self.glistening_phase + dt * BLINKER_GLISTEN_SPEED)
                .rem_euclid(std::f32::consts::TAU);
            self.blinker_timer += dt;
            if self.blinker_timer >= self.blinker_phase_secs {
                self.blinker_timer = 0.0;
                match self.blinker_phase {
                    BlinkerPhase::Glistening => {
                        self.blinker_phase = BlinkerPhase::Invisible;
                        self.blinker_phase_secs =
                            rng.random_range(BLINKER_INVISIBLE_MIN..BLINKER_INVISIBLE_MAX);
                    }
                    BlinkerPhase::Invisible => {
                        self.blinker_phase = BlinkerPhase::Glistening;
                        self.blinker_phase_secs =
                            rng.random_range(BLINKER_GLISTENING_MIN..BLINKER_GLISTENING_MAX);
                    }
                }
            }
        }
        if self.kind == UnfishKind::Worm {
            self.worm_timer += dt;
            if self.worm_timer >= WORM_SWAY_INTERVAL {
                self.worm_timer -= WORM_SWAY_INTERVAL;
                self.worm_forward = !self.worm_forward;
            }
        }
        if self.slime_glisten_enabled {
            self.slime_glisten_phase =
                (self.slime_glisten_phase + dt * self.slime_glisten_speed).rem_euclid(TAU);
        }
    }

    pub fn add_floating_eye(&mut self, rng: &mut impl RngExt) {
        let interior: &[(usize, i32, i32)] = match self.kind {
            UnfishKind::Ball => BALL_INTERIOR,
            UnfishKind::Skull => SKULL_INTERIOR,
            _ => return,
        };
        let center = if self.kind == UnfishKind::Ball && self.ball_has_center_eye {
            Some((BALL_EYE_ROW, BALL_EYE_COL as f32))
        } else {
            None
        };
        try_place_eye(&mut self.floating_eyes, interior, center, rng);
    }

    pub fn remove_floating_eye(&mut self, rng: &mut impl RngExt) {
        if self.floating_eyes.is_empty() {
            return;
        }
        let idx = rng.random_range(0..self.floating_eyes.len());
        self.floating_eyes.remove(idx);
    }
}
