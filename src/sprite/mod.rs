use rand::{RngExt, SeedableRng, rngs::SmallRng};
use ratatui::style::Color;

use crate::entities::components::sway_x_offset;
use crate::entities::glistening::{GlisteningMode, color_for_glisten, derive_glistening_palette};
use crate::util::even_indices;

pub const TRANSPARENT: char = '\0';

pub const EAR_LEFT: char = 'Ɛ';
pub const EAR_RIGHT: char = '3';

const FOOT_QUOTE: char = '"';
const FOOT_CARET: char = '^';
const APPENDAGE_EDGE_INSET: usize = 1;
const FOOT_QUOTE_STRIDE: usize = 2;
const FOOT_CARET_WIDTH: usize = 2;
const FOOT_CARET_UNIT: usize = 3;

const TENTACLE_UPRIGHT: char = '|';
const TENTACLE_LEAN_RIGHT: char = ')';
const TENTACLE_LEAN_LEFT: char = '(';
const SPIKE_GLYPH: char = '¦';
const WING_LEFT_GLYPH: char = '/';
const TENTACLE_STRIDE: usize = 3;
const TENTACLE_WAVE_SPREAD: f32 = 0.5;
const TENTACLE_SWAY_AMOUNT: f32 = 2.0;
const TENTACLE_PHASE_STEP: f32 = 1.3;
const TENTACLE_RELAX_HEIGHT: usize = 3;
const STRAND_PRIME: u64 = 0x9E37_79B9_7F4A_7C15;

pub type Cell = (char, Color);
pub type Grid = Vec<Vec<Cell>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FeetStyle {
    Quote,
    Caret,
}

#[derive(Clone, Copy)]
pub struct Feet {
    pub style: FeetStyle,
    pub color: Option<Color>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExtensionVariant {
    Tentacle,
    Spike,
    Wing,
}

impl ExtensionVariant {
    pub fn start_length(self) -> usize {
        match self {
            ExtensionVariant::Tentacle => 2,
            ExtensionVariant::Spike => 1,
            ExtensionVariant::Wing => 1,
        }
    }

    pub fn sways(self) -> bool {
        matches!(self, ExtensionVariant::Tentacle)
    }
}

#[derive(Clone, Copy)]
pub struct BodyExtension {
    pub variant: ExtensionVariant,
    pub length: usize,
    pub seed: u64,
}

pub fn painted_span(body: &[Cell]) -> Option<(usize, usize)> {
    let painted = |c: char| c != ' ' && c != TRANSPARENT;
    let first = body.iter().position(|&(c, _)| painted(c))?;
    let last = body.iter().rposition(|&(c, _)| painted(c)).unwrap();
    let lo = first + APPENDAGE_EDGE_INSET;
    let hi = last.saturating_sub(APPENDAGE_EDGE_INSET);
    if hi < lo { None } else { Some((lo, hi)) }
}

fn wing_glyph(facing_left: bool, top: bool) -> char {
    let base = if facing_left {
        WING_LEFT_GLYPH
    } else {
        mirror_char(WING_LEFT_GLYPH)
    };
    if top { base } else { mirror_char(base) }
}

fn strand_key(col: usize, top: bool) -> u64 {
    ((col as u64) << 1) | top as u64
}

fn strand_length(seed: u64, key: u64, max_len: usize) -> usize {
    if max_len <= 1 {
        return max_len;
    }
    let mut rng = SmallRng::seed_from_u64(seed ^ key.wrapping_mul(STRAND_PRIME));
    let mut len = 1;
    while len < max_len && rng.random::<bool>() {
        len += 1;
    }
    len
}

fn strand_reaches(ext: BodyExtension, col: usize, top: bool, depth: usize) -> bool {
    depth < strand_length(ext.seed, strand_key(col, top), ext.length)
}

fn tentacle_glyph(offset: i32) -> char {
    if offset > 0 {
        TENTACLE_LEAN_RIGHT
    } else if offset < 0 {
        TENTACLE_LEAN_LEFT
    } else {
        TENTACLE_UPRIGHT
    }
}

#[allow(clippy::too_many_arguments)]
pub fn extension_row(
    body: &[Cell],
    span: (usize, usize),
    ext: BodyExtension,
    depth: usize,
    top: bool,
    facing_left: bool,
    phase: f32,
    max_tentacles: Option<usize>,
) -> Vec<Cell> {
    let mut row = vec![(TRANSPARENT, Color::Reset); body.len()];
    let (lo, hi) = span;
    if hi >= body.len() || lo > hi {
        return row;
    }
    let color_at = |col: usize| body[col].1;
    match ext.variant {
        ExtensionVariant::Tentacle => {
            let region = hi - lo + 1;
            let mut count = region.div_ceil(TENTACLE_STRIDE);
            if let Some(max) = max_tentacles {
                count = count.min(max);
            }
            for (strand, slot) in even_indices(region, count).into_iter().enumerate() {
                let anchor = lo + slot;
                if !strand_reaches(ext, anchor, top, depth) {
                    continue;
                }
                let strand_phase = phase + strand as f32 * TENTACLE_PHASE_STEP;
                let offset = if ext.length <= 1 {
                    0
                } else {
                    sway_x_offset(
                        strand_phase,
                        depth,
                        ext.length.max(TENTACLE_RELAX_HEIGHT),
                        TENTACLE_WAVE_SPREAD,
                        TENTACLE_SWAY_AMOUNT,
                    )
                };
                let col = anchor as i32 + offset;
                if col >= 0 && (col as usize) < row.len() {
                    row[col as usize] = (tentacle_glyph(offset), color_at(anchor));
                }
            }
        }
        ExtensionVariant::Spike => {
            for (col, cell) in row.iter_mut().enumerate().take(hi + 1).skip(lo) {
                if strand_reaches(ext, col, top, depth) {
                    *cell = (SPIKE_GLYPH, body[col].1);
                }
            }
        }
        ExtensionVariant::Wing => {
            let glyph = wing_glyph(facing_left, top);
            let dir: i32 = if facing_left { 1 } else { -1 };
            for col in lo..=hi {
                if !strand_reaches(ext, col, top, depth) {
                    continue;
                }
                let drawn = col as i32 + dir * depth as i32;
                if drawn >= 0 && (drawn as usize) < row.len() {
                    row[drawn as usize] = (glyph, color_at(col));
                }
            }
        }
    }
    row
}

pub fn feet_row(body: &[Cell], span: (usize, usize), feet: Feet) -> Vec<Cell> {
    let mut row = vec![(TRANSPARENT, Color::Reset); body.len()];
    let (lo, hi) = span;
    if hi >= body.len() || lo > hi {
        return row;
    }
    let region = hi - lo + 1;
    let color_at = |col: usize| feet.color.unwrap_or(body[col].1);
    match feet.style {
        FeetStyle::Quote => {
            let count = region.div_ceil(FOOT_QUOTE_STRIDE);
            for slot in even_indices(region, count) {
                let col = lo + slot;
                row[col] = (FOOT_QUOTE, color_at(col));
            }
        }
        FeetStyle::Caret => {
            let pairs = (region + 1) / FOOT_CARET_UNIT;
            if pairs == 0 {
                return row;
            }
            let used = pairs * FOOT_CARET_UNIT - 1;
            let left_pad = (region - used) / 2;
            for pair in 0..pairs {
                let start = lo + left_pad + pair * FOOT_CARET_UNIT;
                for (col, cell) in row
                    .iter_mut()
                    .enumerate()
                    .skip(start)
                    .take(FOOT_CARET_WIDTH)
                {
                    *cell = (FOOT_CARET, color_at(col));
                }
            }
        }
    }
    row
}

pub fn ear_glyph(facing_left: bool) -> char {
    if facing_left { EAR_LEFT } else { EAR_RIGHT }
}

pub fn mirror_char(ch: char) -> char {
    match ch {
        '/' => '\\',
        '\\' => '/',
        EAR_LEFT => EAR_RIGHT,
        EAR_RIGHT => EAR_LEFT,
        '(' => ')',
        ')' => '(',
        '{' => '}',
        '}' => '{',
        '[' => ']',
        ']' => '[',
        '`' => '\'',
        '\'' => '`',
        '╱' => '╲',
        '╲' => '╱',
        '⟋' => '⟍',
        '⟍' => '⟋',
        other => other,
    }
}

pub fn mirror_grid(grid: &[Vec<Cell>]) -> Grid {
    let width = grid.iter().map(|row| row.len()).max().unwrap_or(0);
    grid.iter()
        .map(|row| {
            (0..width)
                .rev()
                .map(|i| {
                    row.get(i)
                        .map(|&(ch, color)| (mirror_char(ch), color))
                        .unwrap_or((TRANSPARENT, Color::Reset))
                })
                .collect()
        })
        .collect()
}

pub fn opaque_line(line: &str, fill: Color, color_fn: impl Fn(char, usize) -> Color) -> Vec<Cell> {
    let chars: Vec<char> = line.chars().collect();
    let first = chars.iter().position(|&c| c != ' ');
    let last = chars.iter().rposition(|&c| c != ' ');
    chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            if c == ' ' {
                match (first, last) {
                    (Some(f), Some(l)) if i > f && i < l => (' ', fill),
                    _ => (TRANSPARENT, fill),
                }
            } else {
                (c, color_fn(c, i))
            }
        })
        .collect()
}

pub fn apply_glisten(
    rows: &mut [Vec<Cell>],
    phase: f32,
    mode: GlisteningMode,
    base: Color,
    peak: Color,
) {
    let (b, mid, p_default) = derive_glistening_palette(base);
    let p = if peak == Color::Reset {
        p_default
    } else {
        peak
    };
    let total = rows.len();
    for (row_i, row) in rows.iter_mut().enumerate() {
        let c = color_for_glisten(mode, phase, row_i, total, b, mid, p);
        for cell in row.iter_mut() {
            if cell.0 != ' ' && cell.0 != TRANSPARENT {
                cell.1 = c;
            }
        }
    }
}
