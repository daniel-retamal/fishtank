use std::f32::consts::TAU;

use rand::RngExt;
use ratatui::style::Color;
use unicode_width::UnicodeWidthChar;

use crate::colors::{CORAL_PINK, CORAL_PURPLE, ORANGE};

pub const CORAL_A_ROWS: usize = 31;
pub const ALGAE_COUNT: usize = 11;
pub const CORAL_COLOR: Color = CORAL_PURPLE;
pub const FLOOR_ALGAE_COLOR: Color = CORAL_PINK;

const ANIM_TIMER_MIN: f32 = 0.5;
const ANIM_TIMER_MAX: f32 = 2.5;
const B_CHARS: [char; 3] = ['º', '@', '^'];

const J_WAVE_SPEED: f32 = 0.4;
const J_WAVE_SPREAD: f32 = 1.0;
const J_WAVE_THRESHOLD: f32 = 0.5;

const VERT_WAVE_SPEED: f32 = 0.8;
const VERT_WAVE_SPREAD: f32 = 0.7;
const VERT_WAVE_THRESHOLD: f32 = 0.4;

const FLOOR_WAVE_SPEED: f32 = 0.6;
const FLOOR_WAVE_SPREAD: f32 = 0.5;
const FLOOR_WAVE_THRESHOLD: f32 = 0.4;

const CORAL_LEFT_MIN: i32 = 1;
const CORAL_LEFT_MAX: i32 = 24;
const CORAL_GAP_MIN: i32 = 70;
const CORAL_GAP_MAX: i32 = 80;
const FLOOR_ALGAE_CORAL_CLEARANCE: i32 = 10;
const FLOOR_ALGAE_ORIGIN_COL: i32 = 14;
const FLOOR_ALGAE_EXTRA_SPACING_MIN: i32 = 5;
const FLOOR_ALGAE_EXTRA_SPACING_MAX: i32 = 20;

const ALGAE_COLORS: &[Color] = &[
    Color::Green,
    Color::LightGreen,
    Color::Yellow,
    Color::LightYellow,
    Color::Cyan,
    Color::LightCyan,
    Color::Blue,
    Color::LightBlue,
    Color::Magenta,
    Color::LightMagenta,
    ORANGE,
];

pub const CORAL_A_LINES: &[&str] = &[
    "                      _⎽_",
    "                     || ⎸⎸",
    "                _⎽⎼⎻⎺‾_⎽||__",
    "               //        / ||",
    "               ⎸⎸    X  ╱   ⎸⎸",
    r#"            _--⎺   ⎽⎼⎼⎻⎻⎺    \\"#,
    "           ⎹⎹                ⟍⟍",
    "          ⎹⎹                  ⎸⎸",
    "           ⟍⟍   X             ||",
    "            ||            X   ⎹⎹",
    "           ||                 ||",
    "           ⎸⎸_---‾           //",
    "        _-‾_-          ╱    ⎹⎹",
    "       //  X    __---‾‾     ||",
    "      ⎹⎹⎽⎽⎼⎼⎻⎻⎺⎺‾           ⎼⎽⎺⎻⎼⎽",
    r#"      ||               X         \\"#,
    "      ⎸⎸                          ))",
    "     ((    X      ⎺⎻⎼⎽            ((",
    r#"      ⎺⎻⎼⎽⎺⎻⎼⎽      ⟍          X    \\"#,
    r#"         ⎺⎻⎼⎽⎺⎻      \ X             ||"#,
    "            ||       ⎺⎺⎻⎻⎼⎼⎽⎽_       ⎹⎹",
    "            ⎸⎸               ‾-_ X  ||",
    "        ⎽⎼⎻⎺⎽⎼     X             ‾-_((",
    r#"      ⟋⟋X                           \\"#,
    "   _-‾_-                             ))",
    "  ╱╱                          X     ((",
    " ╱╱           X            _⎽⎽⎼⎼⎻⎻⎺⎺‾ ⟍⟍",
    "((                _⎽⎽⎼⎼⎻⎻⎺⎺‾            ⟍⟍",
    r#" \\                                     \\"#,
    "  ||                                    ||",
    "  ⎸⎸    X           X            X      ⎸⎸",
];

const ALGAE_A: &[&str] = &[
    ". │   │/    /   │",
    r#"\ \ . \\│  │/ /_/"#,
    r#" \_\│  \\_│/_/"#,
    r#"    \\_││"#,
];

const ALGAE_B: &[&str] = &[
    " {º}(^)",
    "(^)(@){^}",
    "{^}(º)(@)(º)",
    "|_(º){^}(^)|",
    "  |_||_||_|",
];

const ALGAE_C: &[&str] = &[".__._.__.", "(;;););;)", "|;;|;|;;|", "|;;|;|;;|"];

const ALGAE_D: &[&str] = &["((()))", " ((()())", "((())))"];

const ALGAE_E: &[&str] = &["  ...  .", " .|||_/.", r#" \\//_/"#, r#"  \\|/"#];

const ALGAE_F: &[&str] = &["  .", ". |/.  .", r#"\_|/|_/."#, r#" \||/_/"#];

const ALGAE_G: &[&str] = &[".__._.__.", "|;;|;|;;|", "|;;|;|;;|"];

const ALGAE_HEIGHT: &[&str] = &[
    r#". \_ | @ |  ."#,
    r#" \  \| | /_/"#,
    r#"  \__\\|//_."#,
    r#"      \||"#,
];

const ALGAE_I: &[&str] = &[" {}{ {{}{ ", "}{}}{{}{}}{", " {{}}{{ {} "];

const ALGAE_K: &[&str] = &[
    " {}{{}}{{}{}}{}",
    r#"    \| | /_/"#,
    r#"      \|//"#,
    r#"      \||"#,
];

const ALGAE_J: &[&str] = &[
    r#"/;\"#,
    r#"|;|   /;\"#,
    r#"|;|/;\|;|"#,
    r#"|;||;||;|"#,
    r#"|;||;||;|"#,
    r#"|;||;||;|"#,
];

pub fn algae_art(idx: usize) -> &'static [&'static str] {
    match idx % ALGAE_COUNT {
        0 => ALGAE_A,
        1 => ALGAE_B,
        2 => ALGAE_C,
        3 => ALGAE_D,
        4 => ALGAE_E,
        5 => ALGAE_F,
        6 => ALGAE_G,
        7 => ALGAE_HEIGHT,
        8 => ALGAE_I,
        9 => ALGAE_K,
        _ => ALGAE_J,
    }
}

pub const FLOOR_ALGAE_A: &[&str] = &[
    "       .    \\│",
    "  .    │     \\│ │",
    r#"  \__ ││      \││..    / /"#,
    r#"     \│         \││  │/_/"#,
    r#"      \\│        \│ │//"#,
    r#"   .    \\│ .     │//"#,
    r#".   \     \\│.   ││/    ."#,
    r#"\__  \│    \\│  │//     │"#,
    r#"   \__\│     \\ //     /  /"#,
    r#"      \\      \│/     /__//"#,
    r#"      /\\│    ││ │___//"#,
    r#"  .__/  \\   /││//"#,
    r#"         \\// │/"#,
    r#"           \\/│/"#,
];

pub struct BCharState {
    pub row: usize,
    pub col: usize,
    pub current: char,
    pub timer: f32,
}

pub struct ToggleState {
    pub row: usize,
    pub col: usize,
    pub flipped: bool,
    pub timer: f32,
}

pub enum AlgaeAnimState {
    None,
    B(Vec<BCharState>),
    D(Vec<ToggleState>),
    HI(Vec<ToggleState>),
    CG {
        shift_timer: f32,
        shift_phase: u8,
        shifted_rows: u8,
        semis: Vec<ToggleState>,
    },
    J {
        wave_phase: f32,
        total_rows: usize,
        semis: Vec<ToggleState>,
    },
    VertWave {
        phase: f32,
    },
}

pub struct CoralAlgaeInstance {
    pub art_row: usize,
    pub art_col: usize,
    pub algae_idx: usize,
    pub color: Color,
    pub mirrored: bool,
    pub anim: AlgaeAnimState,
}

pub struct CoralStructure {
    pub base_x: i32,
    pub mirrored: bool,
    pub algae: Vec<CoralAlgaeInstance>,
}

pub struct FloorAlgae {
    pub base_x: i32,
    pub mirrored: bool,
    pub phase: f32,
}

impl FloorAlgae {
    pub fn new(base_x: i32, mirrored: bool) -> Self {
        Self {
            base_x,
            mirrored,
            phase: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.phase = (self.phase + FLOOR_WAVE_SPEED * dt).rem_euclid(TAU);
    }
}

pub fn mirror_char(ch: char) -> char {
    match ch {
        '/' => '\\',
        '\\' => '/',
        '(' => ')',
        ')' => '(',
        '{' => '}',
        '}' => '{',
        '╱' => '╲',
        '╲' => '╱',
        '⟋' => '⟍',
        '⟍' => '⟋',
        _ => ch,
    }
}

pub fn vert_wave_char(base: char, row: usize, phase: f32) -> char {
    if base == '|' || base == '│' {
        let val = (phase - row as f32 * FLOOR_WAVE_SPREAD).sin();
        if val > FLOOR_WAVE_THRESHOLD {
            return ')';
        } else if val < -FLOOR_WAVE_THRESHOLD {
            return '(';
        }
    }
    base
}

pub fn query_anim_char(anim: &AlgaeAnimState, li: usize, col: usize, base: char) -> char {
    match anim {
        AlgaeAnimState::B(chars) => {
            for bc in chars {
                if bc.row == li && bc.col == col {
                    return bc.current;
                }
            }
            base
        }
        AlgaeAnimState::D(pairs) => {
            for ts in pairs {
                if ts.flipped && ts.row == li {
                    if ts.col == col {
                        return ')';
                    }
                    if ts.col + 1 == col {
                        return '(';
                    }
                }
            }
            base
        }
        AlgaeAnimState::HI(chars) => {
            for ts in chars {
                if ts.row == li && ts.col == col && ts.flipped {
                    return if base == '{' { '}' } else { '{' };
                }
            }
            base
        }
        AlgaeAnimState::CG { semis, .. } => {
            for ts in semis {
                if ts.row == li && ts.col == col && ts.flipped {
                    return ':';
                }
            }
            base
        }
        AlgaeAnimState::J { semis, .. } => {
            for ts in semis {
                if ts.row == li && ts.col == col && ts.flipped {
                    return ':';
                }
            }
            base
        }
        AlgaeAnimState::VertWave { phase } => {
            if base == '|' || base == '│' {
                let val = (*phase - li as f32 * VERT_WAVE_SPREAD).sin();
                if val > VERT_WAVE_THRESHOLD {
                    return ')';
                } else if val < -VERT_WAVE_THRESHOLD {
                    return '(';
                }
            }
            base
        }
        AlgaeAnimState::None => base,
    }
}

pub fn algae_row_shift(anim: &AlgaeAnimState, li: usize) -> i32 {
    match anim {
        AlgaeAnimState::CG {
            shift_phase,
            shifted_rows,
            ..
        } if li < *shifted_rows as usize => match shift_phase & 3 {
            1 => -1,
            3 => 1,
            _ => 0,
        },
        AlgaeAnimState::J {
            wave_phase,
            total_rows,
            ..
        } if li + 1 < *total_rows => {
            let val = (*wave_phase - li as f32 * J_WAVE_SPREAD).sin();
            if val > J_WAVE_THRESHOLD {
                1
            } else if val < -J_WAVE_THRESHOLD {
                -1
            } else {
                0
            }
        }
        _ => 0,
    }
}

impl CoralAlgaeInstance {
    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt) {
        match &mut self.anim {
            AlgaeAnimState::None => {}
            AlgaeAnimState::B(chars) => {
                for bc in chars.iter_mut() {
                    bc.timer -= dt;
                    if bc.timer <= 0.0 {
                        bc.current = next_b_char(bc.current, rng);
                        bc.timer = rng.random_range(ANIM_TIMER_MIN..ANIM_TIMER_MAX);
                    }
                }
            }
            AlgaeAnimState::D(pairs) => {
                tick_toggles(pairs, dt, rng);
            }
            AlgaeAnimState::HI(chars) => {
                tick_toggles(chars, dt, rng);
            }
            AlgaeAnimState::CG {
                shift_timer,
                shift_phase,
                semis,
                ..
            } => {
                *shift_timer -= dt;
                if *shift_timer <= 0.0 {
                    *shift_phase = (*shift_phase + 1) % 4;
                    *shift_timer = rng.random_range(ANIM_TIMER_MIN..ANIM_TIMER_MAX);
                }
                tick_toggles(semis, dt, rng);
            }
            AlgaeAnimState::J {
                wave_phase, semis, ..
            } => {
                *wave_phase = (*wave_phase + J_WAVE_SPEED * dt).rem_euclid(TAU);
                tick_toggles(semis, dt, rng);
            }
            AlgaeAnimState::VertWave { phase } => {
                *phase = (*phase + VERT_WAVE_SPEED * dt).rem_euclid(TAU);
            }
        }
    }
}

impl CoralStructure {
    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt) {
        for algae in &mut self.algae {
            algae.tick(dt, rng);
        }
    }
}

pub fn extend_coral_reef(
    corals: &mut Vec<CoralStructure>,
    floor_algae: &mut Vec<FloorAlgae>,
    to_width: i32,
    rng: &mut impl RngExt,
) {
    let x_positions = scan_x_positions();
    let canvas_w = coral_art_canvas_w() as i32;
    let floor_canvas_w = floor_algae_canvas_w();

    let (mut coral_x, mut prev_coral_end) = if let Some(last) = corals.last() {
        let end = last.base_x + canvas_w;
        (end + rng.random_range(CORAL_GAP_MIN..=CORAL_GAP_MAX), end)
    } else {
        (rng.random_range(CORAL_LEFT_MIN..=CORAL_LEFT_MAX), 0)
    };

    let mut next_floor_origin = {
        let fa_frontier = floor_algae
            .last()
            .map(|fa| {
                fa.base_x + FLOOR_ALGAE_ORIGIN_COL + floor_canvas_w + FLOOR_ALGAE_EXTRA_SPACING_MIN
            })
            .unwrap_or(0);
        let coral_frontier = corals
            .last()
            .map(|c| c.base_x + canvas_w + FLOOR_ALGAE_CORAL_CLEARANCE)
            .unwrap_or(0);
        fa_frontier.max(coral_frontier).max(FLOOR_ALGAE_ORIGIN_COL)
    };

    while coral_x < to_width {
        let coral_end = coral_x + canvas_w;
        let gap_min_origin = prev_coral_end + FLOOR_ALGAE_CORAL_CLEARANCE;
        let gap_max_origin = coral_x - FLOOR_ALGAE_CORAL_CLEARANCE;

        if next_floor_origin < gap_min_origin {
            next_floor_origin = gap_min_origin;
        }
        while next_floor_origin <= gap_max_origin {
            let base_x = next_floor_origin - FLOOR_ALGAE_ORIGIN_COL;
            floor_algae.push(FloorAlgae::new(base_x, rng.random::<bool>()));
            next_floor_origin += floor_canvas_w
                + rng.random_range(FLOOR_ALGAE_EXTRA_SPACING_MIN..=FLOOR_ALGAE_EXTRA_SPACING_MAX);
        }

        let after_clearance = coral_end + FLOOR_ALGAE_CORAL_CLEARANCE;
        if next_floor_origin < after_clearance {
            next_floor_origin = after_clearance;
        }

        let mirrored = rng.random::<bool>();
        let algae = spawn_algae_for_coral(&x_positions, canvas_w as usize, mirrored, rng);
        corals.push(CoralStructure {
            base_x: coral_x,
            mirrored,
            algae,
        });

        prev_coral_end = coral_end;
        coral_x = coral_end + rng.random_range(CORAL_GAP_MIN..=CORAL_GAP_MAX);
    }
}

fn spawn_algae_for_coral(
    x_positions: &[(usize, usize)],
    canvas_w: usize,
    coral_mirrored: bool,
    rng: &mut impl RngExt,
) -> Vec<CoralAlgaeInstance> {
    x_positions
        .iter()
        .map(|&(art_row, art_col)| {
            let effective_col = if coral_mirrored {
                canvas_w.saturating_sub(art_col + 1)
            } else {
                art_col
            };
            let algae_idx = rng.random_range(0..ALGAE_COUNT);
            let algae_mirrored = rng.random::<bool>();
            let anim = init_algae_anim(algae_idx, rng);
            CoralAlgaeInstance {
                art_row,
                art_col: effective_col,
                algae_idx,
                color: ALGAE_COLORS[rng.random_range(0..ALGAE_COLORS.len())],
                mirrored: algae_mirrored,
                anim,
            }
        })
        .collect()
}

fn init_algae_anim(algae_idx: usize, rng: &mut impl RngExt) -> AlgaeAnimState {
    match algae_idx % ALGAE_COUNT {
        0 | 4 | 5 => AlgaeAnimState::VertWave {
            phase: rng.random::<f32>() * TAU,
        },
        1 => AlgaeAnimState::B(scan_b_chars(rng)),
        2 => AlgaeAnimState::CG {
            shift_timer: rng.random_range(ANIM_TIMER_MIN..ANIM_TIMER_MAX),
            shift_phase: 0,
            shifted_rows: 3,
            semis: scan_semis(ALGAE_C, rng),
        },
        3 => AlgaeAnimState::D(scan_d_pairs(rng)),
        6 => AlgaeAnimState::CG {
            shift_timer: rng.random_range(ANIM_TIMER_MIN..ANIM_TIMER_MAX),
            shift_phase: 0,
            shifted_rows: 2,
            semis: scan_semis(ALGAE_G, rng),
        },
        8 => AlgaeAnimState::HI(scan_hi_chars(ALGAE_I, rng)),
        9 => AlgaeAnimState::HI(scan_hi_chars(ALGAE_K, rng)),
        10 => AlgaeAnimState::J {
            wave_phase: rng.random::<f32>() * TAU,
            total_rows: ALGAE_J.len(),
            semis: scan_semis(ALGAE_J, rng),
        },
        _ => AlgaeAnimState::None,
    }
}

fn tick_toggles(states: &mut [ToggleState], dt: f32, rng: &mut impl RngExt) {
    for ts in states.iter_mut() {
        ts.timer -= dt;
        if ts.timer <= 0.0 {
            ts.flipped = !ts.flipped;
            ts.timer = rng.random_range(ANIM_TIMER_MIN..ANIM_TIMER_MAX);
        }
    }
}

fn next_b_char(current: char, rng: &mut impl RngExt) -> char {
    let others: Vec<char> = B_CHARS.iter().filter(|&&c| c != current).copied().collect();
    others[rng.random_range(0..others.len())]
}

fn scan_x_positions() -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    for (row, line) in CORAL_A_LINES.iter().enumerate() {
        let mut col = 0usize;
        for ch in line.chars() {
            if ch == 'X' {
                positions.push((row, col));
            }
            col += UnicodeWidthChar::width(ch).unwrap_or(1);
        }
    }
    positions
}

fn line_display_width(line: &str) -> usize {
    line.chars()
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(1))
        .sum()
}

fn coral_art_canvas_w() -> usize {
    CORAL_A_LINES
        .iter()
        .map(|l| line_display_width(l))
        .max()
        .unwrap_or(0)
}

fn floor_algae_canvas_w() -> i32 {
    FLOOR_ALGAE_A
        .iter()
        .map(|l| line_display_width(l) as i32)
        .max()
        .unwrap_or(0)
}

fn scan_b_chars(rng: &mut impl RngExt) -> Vec<BCharState> {
    let mut states = Vec::new();
    for (row, line) in ALGAE_B.iter().enumerate() {
        let mut col = 0usize;
        for ch in line.chars() {
            if B_CHARS.contains(&ch) {
                states.push(BCharState {
                    row,
                    col,
                    current: ch,
                    timer: rng.random_range(ANIM_TIMER_MIN..ANIM_TIMER_MAX),
                });
            }
            col += UnicodeWidthChar::width(ch).unwrap_or(1);
        }
    }
    states
}

fn scan_d_pairs(rng: &mut impl RngExt) -> Vec<ToggleState> {
    let mut pairs = Vec::new();
    for (row, line) in ALGAE_D.iter().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        let mut col = 0usize;
        let mut i = 0usize;
        while i + 1 < chars.len() {
            let w = UnicodeWidthChar::width(chars[i]).unwrap_or(1);
            if chars[i] == '(' && chars[i + 1] == ')' {
                pairs.push(ToggleState {
                    row,
                    col,
                    flipped: false,
                    timer: rng.random_range(ANIM_TIMER_MIN..ANIM_TIMER_MAX),
                });
                col += w + UnicodeWidthChar::width(chars[i + 1]).unwrap_or(1);
                i += 2;
                continue;
            }
            col += w;
            i += 1;
        }
    }
    pairs
}

fn scan_hi_chars(art: &[&str], rng: &mut impl RngExt) -> Vec<ToggleState> {
    let mut states = Vec::new();
    for (row, line) in art.iter().enumerate() {
        let mut col = 0usize;
        for ch in line.chars() {
            if ch == '{' || ch == '}' {
                states.push(ToggleState {
                    row,
                    col,
                    flipped: false,
                    timer: rng.random_range(ANIM_TIMER_MIN..ANIM_TIMER_MAX),
                });
            }
            col += UnicodeWidthChar::width(ch).unwrap_or(1);
        }
    }
    states
}

fn scan_semis(art: &[&str], rng: &mut impl RngExt) -> Vec<ToggleState> {
    let mut states = Vec::new();
    for (row, line) in art.iter().enumerate() {
        let mut col = 0usize;
        for ch in line.chars() {
            if ch == ';' {
                states.push(ToggleState {
                    row,
                    col,
                    flipped: false,
                    timer: rng.random_range(ANIM_TIMER_MIN..ANIM_TIMER_MAX),
                });
            }
            col += UnicodeWidthChar::width(ch).unwrap_or(1);
        }
    }
    states
}
