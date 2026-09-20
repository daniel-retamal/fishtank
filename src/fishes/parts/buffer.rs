pub const BLANK_CELL: u8 = b' ';
pub const WORD_BITS: usize = u8::BITS as usize;
const UNWRITTEN: u8 = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
}

impl Grid {
    pub fn cells(self) -> usize {
        self.width * self.height
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Buffer {
    bytes: Vec<u8>,
    cursor: usize,
    pulse: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn latch(&mut self, line: &str) {
        self.bytes = line.as_bytes().to_vec();
        self.cursor = 0;
    }

    pub fn advance(&mut self) {
        if self.is_ready() {
            self.cursor += 1;
        }
    }

    pub fn shown(&self) -> u8 {
        self.cursor
            .checked_sub(1)
            .and_then(|last| self.bytes.get(last))
            .copied()
            .unwrap_or(0)
    }

    pub fn is_ready(&self) -> bool {
        self.cursor < self.bytes.len()
    }

    pub fn is_done(&self) -> bool {
        !self.bytes.is_empty() && !self.is_ready()
    }

    pub fn put(&mut self, byte: u8, grid: Grid) {
        if self.bytes.len() != grid.cells() {
            self.blank(grid);
        }
        if self.cursor >= grid.cells() {
            self.scroll(grid);
        }
        self.bytes[self.cursor] = byte;
        self.cursor += 1;
    }

    pub fn blank(&mut self, grid: Grid) {
        self.bytes = vec![BLANK_CELL; grid.cells()];
        self.cursor = 0;
    }

    fn scroll(&mut self, grid: Grid) {
        self.bytes.drain(..grid.width);
        self.bytes
            .extend(std::iter::repeat_n(BLANK_CELL, grid.width));
        self.cursor -= grid.width;
    }

    pub fn store(&mut self, address: usize, word: u8) {
        if self.bytes.len() <= address {
            self.bytes.resize(address + 1, UNWRITTEN);
        }
        self.bytes[address] = word;
    }

    pub fn load(&self, address: usize) -> u8 {
        self.bytes.get(address).copied().unwrap_or(UNWRITTEN)
    }

    pub fn store_bit(&mut self, index: usize, high: bool) {
        let address = index / WORD_BITS;
        let mask = 1u8 << (index % WORD_BITS);
        let word = self.load(address);
        self.store(address, if high { word | mask } else { word & !mask });
    }

    pub fn load_bit(&self, index: usize) -> bool {
        self.load(index / WORD_BITS) >> (index % WORD_BITS) & 1 == 1
    }

    pub fn set_line(&mut self, line: usize, high: bool) {
        self.store(line, u8::from(high));
    }

    pub fn line(&self, line: usize) -> bool {
        self.load(line) != UNWRITTEN
    }

    pub fn raise(&mut self) {
        self.pulse = true;
    }

    pub fn pulsed(&self) -> bool {
        self.pulse
    }

    pub fn settle(&mut self) {
        self.pulse = false;
    }

    pub fn rows(&self, grid: Grid) -> Option<Vec<&[u8]>> {
        if self.bytes.len() != grid.cells() || self.bytes.iter().all(|&cell| cell == BLANK_CELL) {
            return None;
        }
        Some(self.bytes.chunks(grid.width).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clocked_out(buffer: &mut Buffer, strobes: usize) -> Vec<u8> {
        (0..strobes)
            .map(|_| {
                buffer.advance();
                buffer.shown()
            })
            .collect()
    }

    #[test]
    fn an_empty_buffer_shows_nothing_and_is_neither_ready_nor_done() {
        let buffer = Buffer::new();
        assert_eq!(buffer.shown(), 0);
        assert!(!buffer.is_ready());
        assert!(!buffer.is_done(), "nothing was ever read to its end");
    }

    #[test]
    fn a_latched_line_is_ready_and_shows_nothing_until_the_first_advance() {
        let mut buffer = Buffer::new();
        buffer.latch("hi");
        assert!(buffer.is_ready());
        assert!(!buffer.is_done());
        assert_eq!(buffer.shown(), 0, "no character has been clocked out yet");
    }

    #[test]
    fn each_advance_shows_the_next_byte_of_the_line() {
        let mut buffer = Buffer::new();
        buffer.latch("6 + 4");
        assert_eq!(
            clocked_out(&mut buffer, 5),
            vec![0x36, 0x20, 0x2B, 0x20, 0x34]
        );
        assert!(!buffer.is_ready());
        assert!(buffer.is_done());
    }

    #[test]
    fn advancing_past_the_end_holds_the_last_byte() {
        let mut buffer = Buffer::new();
        buffer.latch("ok");
        assert_eq!(clocked_out(&mut buffer, 4), vec![b'o', b'k', b'k', b'k']);
        assert!(buffer.is_done());
    }

    #[test]
    fn a_new_line_replaces_the_one_being_read_and_starts_again() {
        let mut buffer = Buffer::new();
        buffer.latch("abc");
        buffer.advance();
        buffer.latch("z");
        assert!(buffer.is_ready());
        assert_eq!(buffer.shown(), 0);
        assert_eq!(clocked_out(&mut buffer, 1), vec![b'z']);
    }

    const PANEL: Grid = Grid {
        width: 4,
        height: 2,
    };

    fn typed(text: &str) -> Buffer {
        let mut buffer = Buffer::new();
        for &byte in text.as_bytes() {
            buffer.put(byte, PANEL);
        }
        buffer
    }

    fn shown(buffer: &Buffer) -> Option<Vec<String>> {
        buffer.rows(PANEL).map(|rows| {
            rows.iter()
                .map(|row| String::from_utf8_lossy(row).into_owned())
                .collect()
        })
    }

    #[test]
    fn a_panel_nobody_wrote_to_shows_nothing() {
        assert_eq!(shown(&Buffer::new()), None);
    }

    #[test]
    fn each_write_lands_at_the_cursor_and_the_cursor_advances() {
        assert_eq!(
            shown(&typed("ab")),
            Some(vec!["ab  ".to_string(), "    ".to_string()])
        );
    }

    #[test]
    fn the_cursor_wraps_to_the_next_line_at_the_end_of_one() {
        assert_eq!(
            shown(&typed("abcde")),
            Some(vec!["abcd".to_string(), "e   ".to_string()])
        );
    }

    #[test]
    fn a_full_panel_shows_every_cell_and_scrolls_only_when_one_more_arrives() {
        assert_eq!(
            shown(&typed("abcdefgh")),
            Some(vec!["abcd".to_string(), "efgh".to_string()]),
            "the last cell is written without pushing a line away"
        );
        assert_eq!(
            shown(&typed("abcdefghi")),
            Some(vec!["efgh".to_string(), "i   ".to_string()]),
            "the ninth byte scrolls the panel up a line, like a terminal"
        );
    }

    #[test]
    fn blanking_empties_every_cell_and_homes_the_cursor() {
        let mut buffer = typed("abcdef");
        buffer.blank(PANEL);
        assert_eq!(shown(&buffer), None, "a blank panel shows nothing");
        buffer.put(b'z', PANEL);
        assert_eq!(shown(&buffer).expect("z was written")[0], "z   ");
    }

    #[test]
    fn a_panel_of_spaces_is_a_blank_panel() {
        assert_eq!(shown(&typed("    ")), None);
    }

    #[test]
    fn a_panel_resized_to_another_number_of_cells_starts_blank() {
        let buffer = typed("abc");
        let wider = Grid {
            width: 5,
            height: 1,
        };
        assert_eq!(buffer.rows(wider), None, "the old cells are not relaid");
        let mut buffer = buffer;
        buffer.put(b'x', wider);
        let rows = buffer.rows(wider).expect("x was written");
        assert_eq!(rows, vec![b"x    ".as_slice()]);
    }

    #[test]
    fn a_panel_reshaped_over_the_same_cells_relays_them_like_display_ram() {
        let one_line = Grid {
            width: 8,
            height: 1,
        };
        assert_eq!(
            typed("abcde").rows(one_line),
            Some(vec![b"abcde   ".as_slice()]),
            "the cells are one run of memory; the shape only says where lines break"
        );
    }

    #[test]
    fn a_line_nobody_raised_reads_low_and_each_line_holds_on_its_own() {
        let mut buffer = Buffer::new();
        assert!(!buffer.line(3));
        buffer.set_line(3, true);
        buffer.set_line(0, true);
        assert!(buffer.line(3) && buffer.line(0));
        assert!(!buffer.line(1), "raising one line raises no other");
        buffer.set_line(3, false);
        assert!(!buffer.line(3));
        assert!(buffer.line(0), "lowering one line leaves the rest held");
    }

    #[test]
    fn a_word_nobody_stored_loads_zero() {
        let mut buffer = Buffer::new();
        assert_eq!(buffer.load(0), 0);
        assert_eq!(buffer.load(255), 0);
        buffer.store(200, b'@');
        assert_eq!(
            buffer.load(199),
            0,
            "storing far away writes nothing on the way"
        );
        assert_eq!(buffer.load(201), 0);
    }

    #[test]
    fn a_stored_word_loads_back_until_it_is_overwritten() {
        let mut buffer = Buffer::new();
        buffer.store(7, 42);
        buffer.store(3, 9);
        assert_eq!((buffer.load(7), buffer.load(3)), (42, 9));
        buffer.store(7, 1);
        assert_eq!(buffer.load(7), 1);
        assert_eq!(buffer.load(3), 9, "one address is one word");
    }

    #[test]
    fn a_bit_lives_in_the_word_that_holds_it_lowest_bit_first() {
        let mut buffer = Buffer::new();
        buffer.store_bit(0, true);
        buffer.store_bit(9, true);
        assert_eq!(
            buffer.load(0),
            0b0000_0001,
            "bit 0 is the word's lowest bit"
        );
        assert_eq!(
            buffer.load(1),
            0b0000_0010,
            "bit 9 is the second bit of word 1"
        );
        assert!(buffer.load_bit(9) && !buffer.load_bit(8) && !buffer.load_bit(10));
        buffer.store_bit(9, false);
        assert_eq!(buffer.load(1), 0, "clearing a bit leaves its neighbours");
        assert!(buffer.load_bit(0));
        assert!(!buffer.load_bit(2000), "a bit nobody stored reads low");
    }

    #[test]
    fn storing_a_word_sets_the_eight_bits_it_covers() {
        let mut buffer = Buffer::new();
        buffer.store(2, 0b1000_0001);
        let lit: Vec<usize> = (0..32).filter(|&bit| buffer.load_bit(bit)).collect();
        assert_eq!(lit, vec![16, 23]);
    }

    #[test]
    fn a_character_beyond_ascii_is_forwarded_as_the_bytes_that_spell_it() {
        let mut buffer = Buffer::new();
        buffer.latch("é");
        assert_eq!(clocked_out(&mut buffer, 2), "é".as_bytes().to_vec());
    }
}
