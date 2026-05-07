use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub fn height(show_stats: bool) -> u16 {
    if show_stats { 4 } else { 3 }
}

pub struct CommandBar<'a> {
    pub input: &'a str,
    pub cursor_pos: usize,
    pub cursor_visible: bool,
    pub ghost: &'a str,
    pub fish_count: usize,
    pub food_supply: u32,
    pub money: u32,
    pub show_stats: bool,
}

impl Widget for CommandBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let sep = "─".repeat(area.width as usize);
        let sep_style = Style::default().fg(Color::DarkGray);
        let white = Style::default().fg(Color::White);
        let gray = Style::default().fg(Color::DarkGray);

        buf.set_string(area.x, area.y, &sep, sep_style);

        let row = area.y + 1;
        buf.set_string(area.x, row, "> ", white);

        let base_x = area.x + 2;
        let cursor_col = base_x + self.cursor_pos as u16;
        let at_end = self.cursor_pos == self.input.len();

        if self.cursor_pos > 0 && base_x < area.right() {
            let clip = (area.right() - base_x) as usize;
            buf.set_string(base_x, row, &self.input[..self.cursor_pos.min(clip)], white);
        }

        if cursor_col < area.right() {
            let ch = if at_end {
                self.ghost.chars().next().unwrap_or(' ')
            } else {
                self.input[self.cursor_pos..].chars().next().unwrap_or(' ')
            };
            if self.cursor_visible {
                buf[(cursor_col, row)]
                    .set_char(ch)
                    .set_fg(Color::Black)
                    .set_bg(Color::White);
            } else if at_end {
                buf[(cursor_col, row)].set_char(ch).set_fg(Color::DarkGray);
            } else {
                buf[(cursor_col, row)].set_char(ch).set_fg(Color::White);
            }
        }

        if at_end {
            let ghost_first_len = self.ghost.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            let ghost_rest = &self.ghost[ghost_first_len..];
            let after_x = cursor_col + 1;
            if !ghost_rest.is_empty() && after_x < area.right() {
                let clip = (area.right() - after_x) as usize;
                buf.set_string(
                    after_x,
                    row,
                    &ghost_rest[..ghost_rest.len().min(clip)],
                    gray,
                );
            }
        } else {
            let char_len = self.input[self.cursor_pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            let after = &self.input[self.cursor_pos + char_len..];
            let after_x = cursor_col + 1;
            if !after.is_empty() && after_x < area.right() {
                let clip = (area.right() - after_x) as usize;
                buf.set_string(after_x, row, &after[..after.len().min(clip)], white);
            }
            let ghost_x = base_x + self.input.len() as u16;
            if !self.ghost.is_empty() && ghost_x < area.right() {
                let clip = (area.right() - ghost_x) as usize;
                buf.set_string(
                    ghost_x,
                    row,
                    &self.ghost[..self.ghost.len().min(clip)],
                    gray,
                );
            }
        }

        buf.set_string(area.x, area.y + 2, &sep, sep_style);

        if self.show_stats {
            let stats_row = area.y + 3;
            buf.set_string(area.x, stats_row, "Fishtank I", white);

            let stats = format!(
                "cash: {}  food: {}  fishes: {}/∞",
                self.money, self.food_supply, self.fish_count
            );
            let stats_width = stats.chars().count() as u16;
            if stats_width <= area.width {
                let stats_x = area.right().saturating_sub(stats_width);
                buf.set_string(stats_x, stats_row, &stats, white);
            }
        }
    }
}
