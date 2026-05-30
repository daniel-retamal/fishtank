use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::colors::DARK_GRAY;
use unicode_width::UnicodeWidthChar;

pub struct OverlayLayout {
    pub ox: u16,
    pub oy: u16,
    pub w: u16,
    pub h: u16,
}

impl OverlayLayout {
    pub fn centered(area: Rect, w: u16, h: u16) -> Option<Self> {
        if area.width < w || area.height < h {
            return None;
        }
        Some(Self {
            ox: area.x + (area.width - w) / 2,
            oy: area.y + (area.height - h) / 2,
            w,
            h,
        })
    }

    pub fn clear_bg(&self, buf: &mut Buffer, bg: Color) {
        for dy in 0..self.h {
            for dx in 0..self.w {
                buf[(self.ox + dx, self.oy + dy)].reset();
                buf[(self.ox + dx, self.oy + dy)].set_bg(bg);
            }
        }
    }

    pub fn draw_border(&self, buf: &mut Buffer, title: &str, fg: Color, bg: Color) {
        draw_box_border(
            buf,
            Rect {
                x: self.ox,
                y: self.oy,
                width: self.w,
                height: self.h,
            },
            title,
            fg,
            bg,
        );
    }

    pub fn inner_x(&self) -> u16 {
        self.ox + 1
    }

    pub fn inner_y(&self) -> u16 {
        self.oy + 1
    }

    pub fn inner_w(&self) -> u16 {
        self.w.saturating_sub(2)
    }
}

pub fn visual_width(s: &str) -> usize {
    s.chars()
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(1))
        .sum()
}

pub fn pad_right(s: &str, width: usize) -> String {
    let vw = visual_width(s);
    if vw >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - vw))
    }
}

pub fn truncate_str(s: &str, max_width: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(1);
        if w + cw > max_width {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out
}

pub fn draw_box_border(buf: &mut Buffer, area: Rect, title: &str, fg: Color, bg: Color) {
    let (ox, oy, w, h) = (area.x, area.y, area.width, area.height);
    if w < 2 || h < 2 {
        return;
    }
    let s = Style::default().fg(fg).bg(bg);
    let ts = Style::default().fg(fg).add_modifier(Modifier::BOLD).bg(bg);
    let right = ox + w - 1;
    let bottom = oy + h - 1;

    buf[(ox, oy)].set_char('┌').set_style(s);
    buf[(right, oy)].set_char('┐').set_style(s);
    buf[(ox, bottom)].set_char('└').set_style(s);
    buf[(right, bottom)].set_char('┘').set_style(s);

    for dx in 1..w - 1 {
        buf[(ox + dx, oy)].set_char('─').set_style(s);
        buf[(ox + dx, bottom)].set_char('─').set_style(s);
    }
    for dy in 1..h - 1 {
        buf[(ox, oy + dy)].set_char('│').set_style(s);
        buf[(right, oy + dy)].set_char('│').set_style(s);
    }

    if !title.is_empty() && (title.len() as u16 + 4) < w {
        buf.set_string(ox + 2, oy, title, ts);
    }
}

pub fn draw_hint_bar(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    inner_w: u16,
    left: &str,
    right: &str,
    bg: Color,
) {
    let s = Style::default().fg(DARK_GRAY).bg(bg);
    buf.set_string(x, y, truncate_str(left, inner_w as usize), s);
    let rw = visual_width(right) as u16;
    let lw = visual_width(left) as u16;
    if lw + rw + 2 <= inner_w {
        buf.set_string(x + inner_w - rw, y, right, s);
    }
}

pub fn draw_box_separator(
    buf: &mut Buffer,
    ox: u16,
    sep_y: u16,
    w: u16,
    col_xs: &[u16],
    fg: Color,
    bg: Color,
) {
    let style = Style::default().fg(fg).bg(bg);

    buf[(ox, sep_y)].set_char('├').set_style(style);
    buf[(ox + w - 1, sep_y)].set_char('┤').set_style(style);
    for dx in 1..w - 1 {
        buf[(ox + dx, sep_y)].set_char('─').set_style(style);
    }
    for &cx in col_xs {
        if cx > ox && cx < ox + w - 1 {
            buf[(cx, sep_y)].set_char('┼').set_style(style);
        }
    }
}
