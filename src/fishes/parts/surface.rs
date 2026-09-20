use super::buffer::{Buffer, WORD_BITS};

pub const DOT_ROWS: usize = 4;
const CELL_DOTS_WIDE: usize = 2;
const BLANK_BRAILLE: u32 = 0x2800;
const BRAILLE_DOTS: [[u8; CELL_DOTS_WIDE]; DOT_ROWS] =
    [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
const QUADRANT_DOTS: [[u8; CELL_DOTS_WIDE]; 2] = [[0x1, 0x2], [0x4, 0x8]];
const QUADRANTS: [char; 16] = [
    ' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█',
];
const QUADRANTS_NAME: &str = "quadrants";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Glyphs {
    Braille,
    Quadrants,
}

impl Glyphs {
    pub fn parse(text: &str) -> Self {
        if text.eq_ignore_ascii_case(QUADRANTS_NAME) {
            return Glyphs::Quadrants;
        }
        Glyphs::Braille
    }

    fn dots(self) -> &'static [[u8; CELL_DOTS_WIDE]] {
        match self {
            Glyphs::Braille => &BRAILLE_DOTS,
            Glyphs::Quadrants => &QUADRANT_DOTS,
        }
    }

    fn glyph(self, bits: u8) -> char {
        match self {
            Glyphs::Braille => {
                char::from_u32(BLANK_BRAILLE + u32::from(bits)).unwrap_or(QUADRANTS[0])
            }
            Glyphs::Quadrants => QUADRANTS[usize::from(bits)],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Surface {
    pub width: usize,
}

impl Surface {
    pub fn dots(self) -> usize {
        self.width * DOT_ROWS
    }

    pub fn bytes(self) -> usize {
        self.dots().div_ceil(WORD_BITS)
    }

    pub fn cells(self) -> usize {
        self.width.div_ceil(CELL_DOTS_WIDE)
    }

    pub fn plot(self, dot: usize, high: bool, buffer: &mut Buffer) {
        if dot < self.dots() {
            buffer.store_bit(dot, high);
        }
    }

    pub fn blit(self, address: usize, byte: u8, buffer: &mut Buffer) {
        if address < self.bytes() {
            buffer.store(address, byte);
        }
    }

    pub fn lit(self, x: usize, y: usize, buffer: &Buffer) -> bool {
        x < self.width && y < DOT_ROWS && buffer.load_bit(y * self.width + x)
    }

    pub fn is_dark(self, buffer: &Buffer) -> bool {
        (0..self.dots()).all(|dot| !buffer.load_bit(dot))
    }

    pub fn draw(self, glyphs: Glyphs, buffer: &Buffer) -> Option<Vec<String>> {
        if self.is_dark(buffer) {
            return None;
        }
        let table = glyphs.dots();
        let tall = table.len();
        let rows = (0..DOT_ROWS / tall)
            .map(|row| {
                (0..self.cells())
                    .map(|cell| glyphs.glyph(self.cell_bits(table, cell, row * tall, buffer)))
                    .collect()
            })
            .collect();
        Some(rows)
    }

    fn cell_bits(
        self,
        table: &[[u8; CELL_DOTS_WIDE]],
        cell: usize,
        top: usize,
        buffer: &Buffer,
    ) -> u8 {
        let mut bits = 0;
        for (dy, row) in table.iter().enumerate() {
            for (dx, &bit) in row.iter().enumerate() {
                if self.lit(cell * CELL_DOTS_WIDE + dx, top + dy, buffer) {
                    bits |= bit;
                }
            }
        }
        bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EIGHT_WIDE: Surface = Surface { width: 8 };

    fn plotted(surface: Surface, dots: &[(usize, usize)]) -> Buffer {
        let mut buffer = Buffer::new();
        for &(x, y) in dots {
            surface.plot(y * surface.width + x, true, &mut buffer);
        }
        buffer
    }

    #[test]
    fn a_surface_is_one_row_of_cells_four_dots_tall_and_holds_a_byte_per_cell() {
        assert_eq!(EIGHT_WIDE.dots(), 32);
        assert_eq!(EIGHT_WIDE.cells(), 4);
        assert_eq!(EIGHT_WIDE.bytes(), 4, "eight dots per cell, one byte");
    }

    #[test]
    fn a_dark_surface_draws_nothing() {
        assert_eq!(EIGHT_WIDE.draw(Glyphs::Braille, &Buffer::new()), None);
        let mut buffer = Buffer::new();
        EIGHT_WIDE.plot(3, true, &mut buffer);
        EIGHT_WIDE.plot(3, false, &mut buffer);
        assert_eq!(EIGHT_WIDE.draw(Glyphs::Braille, &buffer), None);
    }

    #[test]
    fn every_dot_of_a_braille_cell_is_the_unicode_bit_for_that_dot() {
        let one_cell = Surface { width: 2 };
        let numbered = [
            ((0, 0), '⠁'),
            ((0, 1), '⠂'),
            ((0, 2), '⠄'),
            ((1, 0), '⠈'),
            ((1, 1), '⠐'),
            ((1, 2), '⠠'),
            ((0, 3), '⡀'),
            ((1, 3), '⢀'),
        ];
        for (dot, glyph) in numbered {
            assert_eq!(
                one_cell.draw(Glyphs::Braille, &plotted(one_cell, &[dot])),
                Some(vec![glyph.to_string()]),
                "dot {dot:?}"
            );
        }
    }

    #[test]
    fn a_known_two_by_four_block_encodes_exactly() {
        let one_cell = Surface { width: 2 };
        let diagonal = plotted(one_cell, &[(0, 0), (1, 1), (0, 2), (1, 3)]);
        assert_eq!(
            one_cell.draw(Glyphs::Braille, &diagonal),
            Some(vec![
                char::from_u32(0x2800 + 0x01 + 0x10 + 0x04 + 0x80)
                    .unwrap()
                    .to_string()
            ])
        );
        let full = plotted(
            one_cell,
            &[
                (0, 0),
                (1, 0),
                (0, 1),
                (1, 1),
                (0, 2),
                (1, 2),
                (0, 3),
                (1, 3),
            ],
        );
        assert_eq!(
            one_cell.draw(Glyphs::Braille, &full),
            Some(vec!["⣿".to_string()])
        );
    }

    #[test]
    fn a_dot_is_addressed_across_then_down_and_the_picture_is_laid_out_the_same_way() {
        let buffer = plotted(EIGHT_WIDE, &[(0, 0), (7, 3)]);
        assert!(buffer.load_bit(0));
        assert!(buffer.load_bit(31), "the last dot of the last row");
        assert_eq!(
            EIGHT_WIDE.draw(Glyphs::Braille, &buffer),
            Some(vec!["⠁⠀⠀⢀".to_string()])
        );
    }

    #[test]
    fn a_byte_is_eight_dots_of_one_row_lowest_bit_leftmost() {
        let mut buffer = Buffer::new();
        EIGHT_WIDE.blit(1, 0b0000_0011, &mut buffer);
        assert!(EIGHT_WIDE.lit(0, 1, &buffer) && EIGHT_WIDE.lit(1, 1, &buffer));
        assert!(!EIGHT_WIDE.lit(2, 1, &buffer));
        assert!(!EIGHT_WIDE.lit(0, 0, &buffer), "byte 1 is the second row");
        assert_eq!(
            EIGHT_WIDE.draw(Glyphs::Braille, &buffer),
            Some(vec!["⠒⠀⠀⠀".to_string()])
        );
    }

    #[test]
    fn a_write_past_the_surface_lands_nowhere() {
        let mut buffer = Buffer::new();
        EIGHT_WIDE.plot(EIGHT_WIDE.dots(), true, &mut buffer);
        EIGHT_WIDE.blit(EIGHT_WIDE.bytes(), 0xFF, &mut buffer);
        assert_eq!(buffer, Buffer::new());
    }

    #[test]
    fn quadrants_draw_the_same_dots_as_solid_blocks_two_rows_tall() {
        let one_cell = Surface { width: 2 };
        let buffer = plotted(one_cell, &[(0, 0), (1, 1), (0, 2), (0, 3)]);
        assert_eq!(
            one_cell.draw(Glyphs::Quadrants, &buffer),
            Some(vec!["▚".to_string(), "▌".to_string()])
        );
        let full = plotted(
            EIGHT_WIDE,
            &(0..8)
                .flat_map(|x| (0..DOT_ROWS).map(move |y| (x, y)))
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            EIGHT_WIDE.draw(Glyphs::Quadrants, &full),
            Some(vec!["████".to_string(), "████".to_string()])
        );
    }

    #[test]
    fn a_glyph_style_is_read_by_name_and_anything_else_is_braille() {
        assert_eq!(Glyphs::parse("Quadrants"), Glyphs::Quadrants);
        assert_eq!(Glyphs::parse("braille"), Glyphs::Braille);
        assert_eq!(Glyphs::parse("sixels"), Glyphs::Braille);
    }

    #[test]
    fn an_odd_width_draws_its_last_cell_one_dot_wide() {
        let three = Surface { width: 3 };
        assert_eq!(three.cells(), 2);
        let buffer = plotted(three, &[(2, 0), (2, 3)]);
        assert_eq!(
            three.draw(Glyphs::Braille, &buffer),
            Some(vec!["⠀⡁".to_string()])
        );
    }
}
