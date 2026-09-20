use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    style::{Color, Style},
};

use crate::colors::{BLACK, WHITE};
use crate::ui::input_action::InputAction;
use crate::ui::table::{scrolled_to_fit, truncate_str, visual_width};

pub struct TextInput {
    pub value: String,
    pub cursor: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
        }
    }

    pub fn with_value(value: String) -> Self {
        let cursor = value.len();
        Self { value, cursor }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            KeyCode::Backspace if self.cursor > 0 => {
                let prev = prev_char_boundary(&self.value, self.cursor);
                self.value.drain(prev..self.cursor);
                self.cursor = prev;
            }
            KeyCode::Delete if self.cursor < self.value.len() => {
                let next = next_char_boundary(&self.value, self.cursor);
                self.value.drain(self.cursor..next);
            }
            KeyCode::Left if self.cursor > 0 => {
                self.cursor = prev_char_boundary(&self.value, self.cursor);
            }
            KeyCode::Right if self.cursor < self.value.len() => {
                self.cursor = next_char_boundary(&self.value, self.cursor);
            }
            KeyCode::Home => {
                self.cursor = 0;
            }
            KeyCode::End => {
                self.cursor = self.value.len();
            }
            _ => {}
        }
    }

    pub fn handle_action(&mut self, action: &InputAction) {
        match action {
            InputAction::Char(c) => {
                self.value.insert(self.cursor, *c);
                self.cursor += c.len_utf8();
            }
            InputAction::Backspace if self.cursor > 0 => {
                let prev = prev_char_boundary(&self.value, self.cursor);
                self.value.drain(prev..self.cursor);
                self.cursor = prev;
            }
            InputAction::Delete if self.cursor < self.value.len() => {
                let next = next_char_boundary(&self.value, self.cursor);
                self.value.drain(self.cursor..next);
            }
            InputAction::Left if self.cursor > 0 => {
                self.cursor = prev_char_boundary(&self.value, self.cursor);
            }
            InputAction::Right if self.cursor < self.value.len() => {
                self.cursor = next_char_boundary(&self.value, self.cursor);
            }
            InputAction::Home => {
                self.cursor = 0;
            }
            InputAction::End => {
                self.cursor = self.value.len();
            }
            _ => {}
        }
    }
}

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos.saturating_sub(1);
    while p > 0 && !s.is_char_boundary(p) {
        p -= 1;
    }
    p
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos + 1;
    while p < s.len() && !s.is_char_boundary(p) {
        p += 1;
    }
    p
}

const PROMPT: &str = "> ";
const CURSOR_CELLS: usize = 1;
const MIN_FIELD_W: u16 = PROMPT.len() as u16 + CURSOR_CELLS as u16;

pub fn draw_text_cursor(
    buf: &mut Buffer,
    input: &TextInput,
    cursor_vis: bool,
    x: u16,
    y: u16,
    w: u16,
    bg: Color,
) {
    if w < MIN_FIELD_W {
        return;
    }

    let s_white = Style::default().fg(WHITE).bg(bg);

    buf.set_string(x, y, PROMPT, s_white);

    let base_x = x + PROMPT.len() as u16;
    let cp = input.cursor;
    let text = input.as_str();
    let at_end = cp == text.len();
    let room = (x + w - base_x) as usize;
    let before = scrolled_to_fit(&text[..cp], room.saturating_sub(CURSOR_CELLS));
    buf.set_string(base_x, y, before, s_white);
    let cursor_col = base_x + visual_width(before) as u16;

    if cursor_col < x + w {
        let ch = if at_end {
            ' '
        } else {
            text[cp..].chars().next().unwrap_or(' ')
        };
        if cursor_vis {
            buf[(cursor_col, y)]
                .set_char(ch)
                .set_fg(BLACK)
                .set_bg(WHITE);
        } else if !at_end {
            buf[(cursor_col, y)].set_char(ch).set_style(s_white);
        }
    }

    if !at_end {
        let char_len = text[cp..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        let after = &text[cp + char_len..];
        let after_col = cursor_col + 1;
        if !after.is_empty() && after_col < x + w {
            buf.set_string(
                after_col,
                y,
                truncate_str(after, (x + w - after_col) as usize),
                s_white,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    const FIELD_W: u16 = 12;
    const LONG_LINE: &str = "/clone {cheapest mutantfish}";

    fn drawn(input: &TextInput) -> String {
        let area = Rect::new(0, 0, FIELD_W, 1);
        let mut buf = Buffer::empty(area);
        draw_text_cursor(&mut buf, input, true, 0, 0, FIELD_W, Color::Reset);
        (0..FIELD_W)
            .map(|x| buf[(x, 0)].symbol().to_string())
            .collect()
    }

    #[test]
    fn a_line_longer_than_its_field_scrolls_to_keep_the_cursor_in_view() {
        let input = TextInput::with_value(LONG_LINE.to_string());
        let row = drawn(&input);
        assert!(
            row.starts_with("> ") && row.trim_end().ends_with("tfish}"),
            "the end of the line is what you are typing: {row:?}"
        );
    }

    #[test]
    fn a_short_line_is_drawn_from_its_start() {
        let input = TextInput::with_value("/feed 1".to_string());
        assert!(drawn(&input).starts_with("> /feed 1"));
    }
}
