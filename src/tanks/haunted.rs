use std::collections::HashSet;
use std::f32::consts::TAU;

use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{
    AMBER, LIGHT_CYAN, DARK_GRAY, GRAY, GREEN_BRIGHT, GREEN_DARK, LIGHT_YELLOW, ORANGE, ORANGE_DARK,
    ORANGE_LIGHT, WHITE,
};
use crate::entities::components::{BlinkTimer, SwayState, tick_sway};
use crate::util::sample_exponential;

pub const GATE_COLOR: Color = GRAY;
pub const GATE_FLOOR_COLOR: Color = GREEN_DARK;
pub const GATE_FLOOR_CHAR: char = '~';
pub const BARS_PER_BLOCK: usize = 16;
pub const GATE_BAR_W: i32 = 3;
pub const GATE_BLOCK_W: i32 = 7;
pub const GATE_BAR_PIPE_ROWS: usize = 7;
pub const GATE_BLOCK_ROWS: usize = 18;

const BAR_HOLE_MASK_LEN: usize = 512;
const BLOCK_COL2_MASK_LEN: usize = 32;
const BAR_PIPE_HOLE_DENOM: usize = 24;
const BLOCK_COL2_WEIGHT_EQ: usize = 6;
const BLOCK_COL2_WEIGHT_DASH: usize = 3;
const BLOCK_COL2_WEIGHT_SPACE: usize = 8;

pub const GATE_BAR_TILE: &[&str] = &[
    " ! ", "_I_", "-|-", "_|_", "-|-", " | ", " | ", " | ", " | ", " | ", " | ", " | ", "_|_",
    "-|-", "_|_", "-|-", "-|-",
];

pub const GATE_BLOCK_TILE: &[&str] = &[
    " ,___, ", " |=  | ", "_|=  |_", "-|-  |-", "_|   |_", "-|   |-", " |=  | ", " |   | ",
    " |-  | ", " |   | ", " |=  | ", " |   | ", " |   | ", "_|   |_", "-|=  |-", "_|   |_",
    "-|=  |-", "-|-  |-",
];

const GRAVE_MIN_WIDTH: i32 = 13;
const GRAVE_NAME_PAD: i32 = 1;
const GRAVE_BODY_BLANK_ROWS: usize = 4;
const GRAVE_COLOR: Color = DARK_GRAY;
pub const GRAVE_TRANSPARENT: char = '\0';

const PUMPKIN_EYE_OPEN: char = '0';
const PUMPKIN_EYE_CLOSED: char = '-';
const PUMPKIN_EYE_COLOR: Color = LIGHT_YELLOW;
const PUMPKIN_COLORS: &[Color] = &[ORANGE_DARK, ORANGE, ORANGE_LIGHT, AMBER];
const PUMPKIN_SIDE_CHANCE: f32 = 0.5;

const HAUNTED_LEFT_MIN: i32 = 1;
const HAUNTED_LEFT_MAX: i32 = 20;
const HAUNTED_GAP_MIN: i32 = 8;
const HAUNTED_GAP_MAX: i32 = 20;

const BAT_PACK_MEAN_SECS: f32 = 15.0;
const BAT_PACK_MIN: usize = 2;
const BAT_PACK_MAX: usize = 8;
const BAT_SPEED_MIN: f32 = 18.0;
const BAT_SPEED_MAX: f32 = 30.0;
const BAT_SWAY_SPEED: f32 = 0.15;
const BAT_SWAY_AMP: f32 = 1.0;
const BAT_PACK_STAGGER: f32 = 4.0;
const BAT_VERTICAL_FRACTION: f32 = 0.5;
pub const BAT_WIDTH: i32 = 3;
pub const BAT_SPRITE: [char; 3] = ['^', '\'', '^'];
pub const BAT_COLOR: Color = DARK_GRAY;

const GHOST_SPAWN_MIN: f32 = 2.5;
const GHOST_SPAWN_MAX: f32 = 4.0;
const GHOST_RISE_MIN: f32 = 2.0;
const GHOST_RISE_MAX: f32 = 5.0;
const GHOST_SWAY_SPEED: f32 = 0.03;
const GHOST_SWAY_AMP: f32 = 2.0;
const GHOST_COLORS: &[Color] = &[WHITE, GRAY, LIGHT_CYAN, GREEN_BRIGHT];

pub const GHOST_TOP: [&str; 3] = ["  _", r#" (")"#, "(' ')"];
pub const GHOST_BOTTOM_LEFT: [&str; 2] = [r#" \  \"#, "  ^^^"];
pub const GHOST_BOTTOM_RIGHT: [&str; 2] = ["/  /", "^^^"];
const GHOST_ROWS: i32 = 5;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PumpkinVariant {
    Lantern,
    Question,
    Slash,
    Stub,
}

const PUMPKIN_LANTERN: &[&str] = &[
    r#"         /\"#,
    r#"       ._)_)_."#,
    r#"    .-'._'-'_.'-."#,
    r#"  .'   (0)'(0)   '."#,
    r#" /   /_   A   _\   \"#,
    r#"|     \'=...='/     |"#,
    r#" \     '.___.'     /"#,
    r#"  '-:__:__:__:__:-'"#,
];

const PUMPKIN_QUESTION: &[&str] = &[
    "    _?_",
    r#" .'`"""`'."#,
    r#"/   0.0   \"#,
    r#"\  `===`  /"#,
    " `-------`",
];

const PUMPKIN_SLASH: &[&str] = &[
    r#"    )\"#,
    r#" .'`--`'."#,
    r#"/  0  0  \"#,
    r#"\ \/\/\/ /"#,
    " '------'",
];

const PUMPKIN_STUB: &[&str] = &[
    "    )",
    r#" .'`-`'."#,
    r#"/  0,0  \"#,
    r#"\   u   /"#,
    " '-----'",
];

impl PumpkinVariant {
    pub fn all() -> [PumpkinVariant; 4] {
        [
            PumpkinVariant::Lantern,
            PumpkinVariant::Question,
            PumpkinVariant::Slash,
            PumpkinVariant::Stub,
        ]
    }

    pub fn random(rng: &mut impl RngExt) -> Self {
        Self::all()[rng.random_range(0..4)]
    }

    fn template(self) -> &'static [&'static str] {
        match self {
            PumpkinVariant::Lantern => PUMPKIN_LANTERN,
            PumpkinVariant::Question => PUMPKIN_QUESTION,
            PumpkinVariant::Slash => PUMPKIN_SLASH,
            PumpkinVariant::Stub => PUMPKIN_STUB,
        }
    }

    fn eyes(self) -> &'static [(usize, usize)] {
        match self {
            PumpkinVariant::Lantern => &[(3, 8), (3, 12)],
            PumpkinVariant::Question => &[(2, 4), (2, 6)],
            PumpkinVariant::Slash => &[(2, 3), (2, 6)],
            PumpkinVariant::Stub => &[(2, 3), (2, 5)],
        }
    }

    pub fn width(self) -> i32 {
        self.template()
            .iter()
            .map(|l| l.chars().count() as i32)
            .max()
            .unwrap_or(0)
    }
}

fn opaque_line(s: &str, color: Color) -> Vec<(char, Color)> {
    let chars: Vec<char> = s.chars().collect();
    let first = chars.iter().position(|&c| c != ' ');
    let last = chars.iter().rposition(|&c| c != ' ');
    chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            if c == ' ' {
                match (first, last) {
                    (Some(f), Some(l)) if i > f && i < l => (' ', color),
                    _ => (GRAVE_TRANSPARENT, color),
                }
            } else {
                (c, color)
            }
        })
        .collect()
}

pub struct Grave {
    pub base_x: i32,
    pub width: i32,
    pub name: Option<String>,
}

impl Grave {
    pub fn rows(&self) -> Vec<Vec<(char, Color)>> {
        let tw = self.width as usize;
        let center = tw / 2;
        let cap_w = tw - 4;
        let mut rows = vec![
            point_row(tw, center, '.'),
            cross_arms_row(tw, center),
            point_row(tw, center, '|'),
            cap_row(tw, cap_w),
            shoulder_row(tw),
            body_row(tw, Some("RIP")),
            body_row(tw, self.name.as_deref()),
        ];
        for _ in 0..GRAVE_BODY_BLANK_ROWS {
            rows.push(body_row(tw, None));
        }
        rows
    }
}

fn point_row(tw: usize, center: usize, ch: char) -> Vec<(char, Color)> {
    (0..tw)
        .map(|i| {
            if i == center {
                (ch, GRAVE_COLOR)
            } else {
                (GRAVE_TRANSPARENT, GRAVE_COLOR)
            }
        })
        .collect()
}

fn cross_arms_row(tw: usize, center: usize) -> Vec<(char, Color)> {
    (0..tw)
        .map(|i| {
            if i == center {
                ('|', GRAVE_COLOR)
            } else if i + 1 == center || i == center + 1 {
                ('-', GRAVE_COLOR)
            } else {
                (GRAVE_TRANSPARENT, GRAVE_COLOR)
            }
        })
        .collect()
}

fn cap_row(tw: usize, cap_w: usize) -> Vec<(char, Color)> {
    let lead = (tw - cap_w) / 2;
    let cap: String = format!(".-'{}`-.", "~".repeat(cap_w - 6));
    let line: String = format!(
        "{}{}{}",
        " ".repeat(lead),
        cap,
        " ".repeat(tw - lead - cap_w)
    );
    opaque_line(&line, GRAVE_COLOR)
}

fn shoulder_row(tw: usize) -> Vec<(char, Color)> {
    let line = format!(".'{}`.", " ".repeat(tw - 4));
    opaque_line(&line, GRAVE_COLOR)
}

fn body_row(tw: usize, name: Option<&str>) -> Vec<(char, Color)> {
    let inner = tw - 2;
    let mut content: Vec<char> = vec![' '; inner];
    if let Some(n) = name {
        let nlen = n.chars().count();
        let start = (inner - nlen) / 2;
        for (i, ch) in n.chars().enumerate() {
            content[start + i] = ch;
        }
    }
    let line: String = std::iter::once('|')
        .chain(content)
        .chain(std::iter::once('|'))
        .collect();
    opaque_line(&line, GRAVE_COLOR)
}

fn grave_width_for(name: Option<&str>) -> i32 {
    let mut w = GRAVE_MIN_WIDTH;
    if let Some(n) = name {
        let needed = n.chars().count() as i32 + 2 * GRAVE_NAME_PAD + 2;
        w = w.max(needed);
    }
    if w % 2 == 0 {
        w += 1;
    }
    w
}

pub struct Pumpkin {
    pub base_x: i32,
    pub variant: PumpkinVariant,
    pub color: Color,
    eye: BlinkTimer,
}

impl Pumpkin {
    fn new(base_x: i32, variant: PumpkinVariant, rng: &mut impl RngExt) -> Self {
        Self {
            base_x,
            variant,
            color: PUMPKIN_COLORS[rng.random_range(0..PUMPKIN_COLORS.len())],
            eye: BlinkTimer::entity_eye(rng),
        }
    }

    pub fn rows(&self) -> Vec<Vec<(char, Color)>> {
        let eye_char = if self.eye.is_open {
            PUMPKIN_EYE_OPEN
        } else {
            PUMPKIN_EYE_CLOSED
        };
        let eyes = self.variant.eyes();
        self.variant
            .template()
            .iter()
            .enumerate()
            .map(|(row_idx, line)| {
                let mut cells = opaque_line(line, self.color);
                for &(er, ec) in eyes {
                    if er == row_idx && ec < cells.len() {
                        cells[ec] = (eye_char, PUMPKIN_EYE_COLOR);
                    }
                }
                cells
            })
            .collect()
    }
}

pub struct Bat {
    pub x: f32,
    pub y: f32,
    base_y: f32,
    sway: SwayState,
    speed: f32,
}

impl Bat {
    fn new(index: usize, height: u16, rng: &mut impl RngExt) -> Self {
        let ceiling = (height as f32 * BAT_VERTICAL_FRACTION).max(1.0);
        let base_y = rng.random_range(1.0..ceiling);
        Self {
            x: -(index as f32 * BAT_PACK_STAGGER) - BAT_WIDTH as f32,
            y: base_y,
            base_y,
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            speed: rng.random_range(BAT_SPEED_MIN..BAT_SPEED_MAX),
        }
    }

    fn tick(&mut self, dt: f32) {
        self.x += self.speed * dt;
        tick_sway(&mut self.sway, BAT_SWAY_SPEED);
        self.y = self.base_y + BAT_SWAY_AMP * self.sway.phase.sin();
    }

    fn is_gone(&self, width: u16) -> bool {
        self.x > width as f32 + BAT_WIDTH as f32
    }
}

pub struct Ghost {
    pub x: f32,
    pub y: f32,
    base_x: f32,
    sway: SwayState,
    rise_speed: f32,
    pub color: Color,
    pub facing_right: bool,
}

impl Ghost {
    fn new(width: u16, height: u16, rng: &mut impl RngExt) -> Self {
        let base_x = rng.random_range(0.0..width as f32);
        Self {
            x: base_x,
            y: (height as f32 - 1.0).max(0.0),
            base_x,
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            rise_speed: rng.random_range(GHOST_RISE_MIN..GHOST_RISE_MAX),
            color: GHOST_COLORS[rng.random_range(0..GHOST_COLORS.len())],
            facing_right: true,
        }
    }

    fn tick(&mut self, dt: f32) {
        self.y -= self.rise_speed * dt;
        tick_sway(&mut self.sway, GHOST_SWAY_SPEED);
        self.x = self.base_x + GHOST_SWAY_AMP * self.sway.phase.sin();
        self.facing_right = self.sway.phase.cos() >= 0.0;
    }

    fn is_gone(&self) -> bool {
        self.y < -(GHOST_ROWS as f32)
    }

    pub fn sprite_rows(&self) -> Vec<&'static str> {
        let bottom = if self.facing_right {
            &GHOST_BOTTOM_RIGHT[..]
        } else {
            &GHOST_BOTTOM_LEFT[..]
        };
        GHOST_TOP.iter().chain(bottom.iter()).copied().collect()
    }
}

pub struct HauntedBackground {
    pub graves: Vec<Grave>,
    pub pumpkins: Vec<Pumpkin>,
    pub bats: Vec<Bat>,
    pub ghosts: Vec<Ghost>,
    bat_timer: f32,
    ghost_timer: f32,
    pub bar_hole_mask: Vec<[bool; GATE_BAR_PIPE_ROWS]>,
    pub block_col2_mask: Vec<[char; GATE_BLOCK_ROWS]>,
}

impl HauntedBackground {
    pub fn new(rng: &mut impl RngExt) -> Self {
        let mut bar_hole_mask: Vec<[bool; GATE_BAR_PIPE_ROWS]> =
            Vec::with_capacity(BAR_HOLE_MASK_LEN);
        for _ in 0..BAR_HOLE_MASK_LEN {
            let mut entry = [false; GATE_BAR_PIPE_ROWS];
            for slot in &mut entry {
                *slot = rng.random_range(0..BAR_PIPE_HOLE_DENOM) == 0;
            }
            bar_hole_mask.push(entry);
        }

        let mut block_col2_mask: Vec<[char; GATE_BLOCK_ROWS]> =
            Vec::with_capacity(BLOCK_COL2_MASK_LEN);
        for _ in 0..BLOCK_COL2_MASK_LEN {
            let mut entry = [' '; GATE_BLOCK_ROWS];
            for (i, row_char) in entry.iter_mut().enumerate() {
                let col2 = GATE_BLOCK_TILE[i].chars().nth(2).unwrap_or(' ');
                *row_char = if matches!(col2, '=' | '-' | ' ') {
                    let total =
                        BLOCK_COL2_WEIGHT_EQ + BLOCK_COL2_WEIGHT_DASH + BLOCK_COL2_WEIGHT_SPACE;
                    let r = rng.random_range(0..total);
                    if r < BLOCK_COL2_WEIGHT_EQ {
                        '='
                    } else if r < BLOCK_COL2_WEIGHT_EQ + BLOCK_COL2_WEIGHT_DASH {
                        '-'
                    } else {
                        ' '
                    }
                } else {
                    col2
                };
            }
            block_col2_mask.push(entry);
        }

        Self {
            graves: Vec::new(),
            pumpkins: Vec::new(),
            bats: Vec::new(),
            ghosts: Vec::new(),
            bat_timer: sample_exponential(rng, BAT_PACK_MEAN_SECS),
            ghost_timer: rng.random_range(GHOST_SPAWN_MIN..GHOST_SPAWN_MAX),
            bar_hole_mask,
            block_col2_mask,
        }
    }

    pub fn clear_grave_name(&mut self, name: &str) {
        for grave in &mut self.graves {
            if grave.name.as_deref() == Some(name) {
                grave.name = None;
            }
        }
    }

    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt, width: u16, height: u16) {
        if width == 0 || height == 0 {
            return;
        }
        for pumpkin in &mut self.pumpkins {
            pumpkin.eye.tick(dt);
        }

        for bat in &mut self.bats {
            bat.tick(dt);
        }
        self.bats.retain(|b| !b.is_gone(width));
        self.bat_timer -= dt;
        if self.bat_timer <= 0.0 {
            self.bat_timer = sample_exponential(rng, BAT_PACK_MEAN_SECS);
            let count = rng.random_range(BAT_PACK_MIN..=BAT_PACK_MAX);
            for k in 0..count {
                self.bats.push(Bat::new(k, height, rng));
            }
        }

        for ghost in &mut self.ghosts {
            ghost.tick(dt);
        }
        self.ghosts.retain(|g| !g.is_gone());
        self.ghost_timer -= dt;
        if self.ghost_timer <= 0.0 {
            self.ghost_timer = rng.random_range(GHOST_SPAWN_MIN..GHOST_SPAWN_MAX);
            self.ghosts.push(Ghost::new(width, height, rng));
        }
    }
}

pub fn extend_haunted(
    graves: &mut Vec<Grave>,
    pumpkins: &mut Vec<Pumpkin>,
    to_width: i32,
    dead_names: &[String],
    rng: &mut impl RngExt,
) {
    let mut used: HashSet<String> = graves.iter().filter_map(|g| g.name.clone()).collect();
    let frontier = decoration_frontier(graves, pumpkins);
    let mut next_x = match frontier {
        Some(edge) => edge + rng.random_range(HAUNTED_GAP_MIN..=HAUNTED_GAP_MAX),
        None => rng.random_range(HAUNTED_LEFT_MIN..=HAUNTED_LEFT_MAX),
    };

    while next_x < to_width {
        let left = (rng.random::<f32>() < PUMPKIN_SIDE_CHANCE).then(|| PumpkinVariant::random(rng));
        let right = (rng.random::<f32>() < PUMPKIN_SIDE_CHANCE).then(|| PumpkinVariant::random(rng));
        let name = dead_names.iter().find(|n| !used.contains(*n)).cloned();
        if let Some(ref n) = name {
            used.insert(n.clone());
        }
        let grave_w = grave_width_for(name.as_deref());

        let mut x = next_x;
        if let Some(variant) = left {
            pumpkins.push(Pumpkin::new(x, variant, rng));
            x += variant.width();
        }
        graves.push(Grave {
            base_x: x,
            width: grave_w,
            name,
        });
        x += grave_w;
        if let Some(variant) = right {
            pumpkins.push(Pumpkin::new(x, variant, rng));
            x += variant.width();
        }
        next_x = x + rng.random_range(HAUNTED_GAP_MIN..=HAUNTED_GAP_MAX);
    }
}

fn decoration_frontier(graves: &[Grave], pumpkins: &[Pumpkin]) -> Option<i32> {
    let grave_edge = graves.iter().map(|g| g.base_x + g.width).max();
    let pumpkin_edge = pumpkins.iter().map(|p| p.base_x + p.variant.width()).max();
    match (grave_edge, pumpkin_edge) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}
