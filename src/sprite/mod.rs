use ratatui::style::Color;

use crate::entities::glistening::{GlisteningMode, color_for_glisten, derive_glistening_palette};

pub const TRANSPARENT: char = '\0';

pub type Cell = (char, Color);
pub type Grid = Vec<Vec<Cell>>;

pub fn mirror_char(ch: char) -> char {
    match ch {
        '/' => '\\',
        '\\' => '/',
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
