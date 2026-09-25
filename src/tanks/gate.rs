use rand::RngExt;
use ratatui::style::Color;

pub const BARS_PER_BLOCK: usize = 16;
pub const GATE_BAR_W: i32 = 3;
pub const GATE_BLOCK_W: i32 = 7;
pub const GATE_BAR_PIPE_ROWS: usize = 7;
pub const GATE_BLOCK_ROWS: usize = 18;
pub const GATE_FLOOR_CHAR: char = '~';
pub const GATE_BAR_PIPE: &str = " | ";
pub const GATE_BLOCK_MARK_COL: usize = 2;

const BAR_HOLE_MASK_LEN: usize = 512;
const BLOCK_MARK_MASK_LEN: usize = 32;
const BAR_PIPE_HOLE_DENOM: usize = 24;
const BLOCK_MARK_WEIGHTS: [(char, usize); 3] = [('=', 6), ('-', 3), (' ', 8)];

pub const GATE_BAR_TILE: &[&str] = &[
    " ! ", "_I_", "-|-", "_|_", "-|-", " | ", " | ", " | ", " | ", " | ", " | ", " | ", "_|_",
    "-|-", "_|_", "-|-", "-|-",
];

pub const GATE_BLOCK_TILE: &[&str] = &[
    " ,___, ", " |=  | ", "_|=  |_", "-|-  |-", "_|   |_", "-|   |-", " |=  | ", " |   | ",
    " |-  | ", " |   | ", " |=  | ", " |   | ", " |   | ", "_|   |_", "-|=  |-", "_|   |_",
    "-|=  |-", "-|-  |-",
];

pub type BarHoles = [bool; GATE_BAR_PIPE_ROWS];
pub type BlockMarks = [char; GATE_BLOCK_ROWS];

#[derive(Clone, Copy)]
pub struct GateStyle {
    pub bars: Color,
    pub floor: Color,
}

pub struct Gate {
    pub style: GateStyle,
    bar_holes: Vec<BarHoles>,
    block_marks: Vec<BlockMarks>,
}

impl Gate {
    pub fn ruined(style: GateStyle, rng: &mut impl RngExt) -> Self {
        let bar_holes = (0..BAR_HOLE_MASK_LEN)
            .map(|_| std::array::from_fn(|_| rng.random_range(0..BAR_PIPE_HOLE_DENOM) == 0))
            .collect();
        Self {
            bar_holes,
            ..Self::whole(style, rng)
        }
    }

    pub fn whole(style: GateStyle, rng: &mut impl RngExt) -> Self {
        let block_marks = (0..BLOCK_MARK_MASK_LEN)
            .map(|_| std::array::from_fn(|row| block_mark(row, rng)))
            .collect();
        Self {
            style,
            bar_holes: Vec::new(),
            block_marks,
        }
    }

    pub fn bar_holes(&self, bar: usize) -> Option<&BarHoles> {
        if self.bar_holes.is_empty() {
            return None;
        }
        Some(&self.bar_holes[bar % self.bar_holes.len()])
    }

    pub fn block_marks(&self, block: usize) -> &BlockMarks {
        &self.block_marks[block % self.block_marks.len()]
    }

    pub fn height() -> i32 {
        GATE_BLOCK_TILE.len().max(GATE_BAR_TILE.len()) as i32 + 1
    }
}

fn block_mark(row: usize, rng: &mut impl RngExt) -> char {
    let drawn = GATE_BLOCK_TILE[row]
        .chars()
        .nth(GATE_BLOCK_MARK_COL)
        .unwrap_or(' ');
    if !BLOCK_MARK_WEIGHTS.iter().any(|&(mark, _)| mark == drawn) {
        return drawn;
    }
    let total: usize = BLOCK_MARK_WEIGHTS.iter().map(|&(_, weight)| weight).sum();
    let mut roll = rng.random_range(0..total);
    for &(mark, weight) in &BLOCK_MARK_WEIGHTS {
        if roll < weight {
            return mark;
        }
        roll -= weight;
    }
    drawn
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colors::{GRAY, GREEN_DARK};

    const STYLE: GateStyle = GateStyle {
        bars: GRAY,
        floor: GREEN_DARK,
    };

    #[test]
    fn a_whole_gate_has_no_holes_and_a_ruined_one_does() {
        let mut rng = rand::rng();
        let whole = Gate::whole(STYLE, &mut rng);
        assert!((0..BAR_HOLE_MASK_LEN).all(|bar| whole.bar_holes(bar).is_none()));
        let ruined = Gate::ruined(STYLE, &mut rng);
        assert!(
            (0..BAR_HOLE_MASK_LEN)
                .filter_map(|bar| ruined.bar_holes(bar))
                .any(|holes| holes.iter().any(|&hole| hole)),
            "a ruined gate is missing some of its bars"
        );
    }

    #[test]
    fn a_block_only_rewrites_the_marks_it_was_drawn_with() {
        let gate = Gate::whole(STYLE, &mut rand::rng());
        for block in 0..BLOCK_MARK_MASK_LEN {
            for (row, &mark) in gate.block_marks(block).iter().enumerate() {
                let drawn = GATE_BLOCK_TILE[row]
                    .chars()
                    .nth(GATE_BLOCK_MARK_COL)
                    .unwrap();
                if ['=', '-', ' '].contains(&drawn) {
                    assert!(['=', '-', ' '].contains(&mark));
                } else {
                    assert_eq!(mark, drawn);
                }
            }
        }
    }
}
