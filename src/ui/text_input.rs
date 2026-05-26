use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    style::{Color, Style},
};

use crate::ui::input_action::InputAction;
use crate::ui::table::truncate_str;

pub struct TextInput {
    pub value: String,
    pub cursor: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
        }
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
            KeyCode::Home => { self.cursor = 0; }
            KeyCode::End => { self.cursor = self.value.len(); }
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
            InputAction::Home => { self.cursor = 0; }
            InputAction::End => { self.cursor = self.value.len(); }
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

pub fn draw_text_cursor(
    buf: &mut Buffer,
    input: &TextInput,
    cursor_vis: bool,
    x: u16,
    y: u16,
    w: u16,
    bg: Color,
) {
    if w < 3 {
        return;
    }

    let s_white = Style::default().fg(Color::White).bg(bg);

    buf[(x, y)].set_char('>').set_style(s_white);
    buf[(x + 1, y)].set_char(' ').set_style(s_white);

    let base_x = x + 2;
    let cp = input.cursor;
    let text = input.as_str();
    let at_end = cp == text.len();
    let cursor_col = base_x + cp as u16;

    if cp > 0 && base_x < x + w {
        let avail = (x + w - base_x) as usize;
        buf.set_string(base_x, y, truncate_str(&text[..cp], avail), s_white);
    }

    if cursor_col < x + w {
        let ch = if at_end {
            ' '
        } else {
            text[cp..].chars().next().unwrap_or(' ')
        };
        if cursor_vis {
            buf[(cursor_col, y)]
                .set_char(ch)
                .set_fg(Color::Black)
                .set_bg(Color::White);
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
