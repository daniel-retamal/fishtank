use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier, Style},
};
use unicode_width::UnicodeWidthChar;

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

pub fn draw_box_border(
    buf: &mut Buffer,
    ox: u16,
    oy: u16,
    w: u16,
    h: u16,
    title: &str,
    fg: Color,
    bg: Color,
) {
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

