use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthChar;

pub struct InventoryState {
    pub selected: usize,
    pub scroll: usize,
    pub items: Vec<(String, u32)>,
}

impl InventoryState {
    pub fn new(inventory: &HashMap<String, u32>) -> Option<Self> {
        let mut items: Vec<(String, u32)> = inventory
            .iter()
            .filter(|(_, qty)| **qty > 0)
            .map(|(name, qty)| (name.clone(), *qty))
            .collect();
        if items.is_empty() {
            return None;
        }
        items.sort_by_key(|(n, _)| n.clone());
        Some(Self {
            selected: 0,
            scroll: 0,
            items,
        })
    }

    pub fn scroll_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll {
                self.scroll = self.selected;
            }
        }
    }

    pub fn scroll_down(&mut self, visible: usize) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
            if self.selected >= self.scroll + visible {
                self.scroll = self.selected + 1 - visible;
            }
        }
    }
}

pub struct InventoryOverlay<'a> {
    state: &'a InventoryState,
}

impl<'a> InventoryOverlay<'a> {
    pub fn new(state: &'a InventoryState) -> Self {
        Self { state }
    }
}

impl Widget for InventoryOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state;
        let n = state.items.len();
        let bg = Color::Reset;

        let item_w = state
            .items
            .iter()
            .map(|(name, _)| visual_width(name))
            .max()
            .unwrap_or(4)
            .max(visual_width("Item"));
        let qty_w = state
            .items
            .iter()
            .map(|(_, q)| q.to_string().len())
            .max()
            .unwrap_or(1)
            .max(visual_width("Quantity"));

        let visible_data_rows = n.min(area.height.saturating_sub(6) as usize).max(1);
        let overlay_h = (visible_data_rows as u16 + 6).min(area.height);

        let scrollable = n > visible_data_rows;
        let left_hint = if scrollable {
            format!(" ↑↓ scroll ({}/{})", state.selected + 1, n)
        } else {
            " ↑↓ navigate".to_string()
        };
        let right_text = "ESC/q close";
        let right_w = visual_width(right_text);
        let left_w = visual_width(&left_hint);
        let footer_min_w = left_w + 2 + right_w;

        let inner_w = (item_w + 1 + qty_w).max(footer_min_w);
        let overlay_w = (inner_w + 2) as u16;

        if area.width < overlay_w || area.height < overlay_h {
            return;
        }

        let ox = area.x + (area.width - overlay_w) / 2;
        let oy = area.y + (area.height - overlay_h) / 2;

        for dy in 0..overlay_h {
            for dx in 0..overlay_w {
                let px = ox + dx;
                let py = oy + dy;
                if px < area.right() && py < area.bottom() {
                    buf[(px, py)].reset();
                    buf[(px, py)].set_bg(bg);
                }
            }
        }

        draw_border(buf, ox, oy, overlay_w, overlay_h, bg);

        let inner_x = ox + 1;
        draw_header(buf, inner_x, oy + 1, item_w, qty_w, bg);
        draw_separator(buf, ox, oy + 2, overlay_w, item_w, bg);

        let data_start_y = oy + 3;
        let data_end_y = oy + overlay_h - 3;

        for row_y in data_start_y..=data_end_y {
            let idx = state.scroll + (row_y - data_start_y) as usize;
            if idx >= n {
                break;
            }
            let selected = idx == state.selected;
            draw_row(
                buf,
                &state.items[idx],
                inner_x,
                row_y,
                item_w,
                qty_w,
                inner_w,
                selected,
                bg,
            );
        }

        let footer_y = oy + overlay_h - 2;
        let hint_style = Style::default().fg(Color::DarkGray).bg(bg);

        buf.set_string(
            inner_x,
            footer_y,
            truncate_str(&left_hint, inner_w),
            hint_style,
        );
        if left_w + 2 + right_w <= inner_w {
            buf.set_string(
                inner_x + (inner_w - right_w) as u16,
                footer_y,
                right_text,
                hint_style,
            );
        }
    }
}

fn draw_border(buf: &mut Buffer, ox: u16, oy: u16, w: u16, h: u16, bg: Color) {
    let style = Style::default().fg(Color::White).bg(bg);
    let title_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(bg);
    let right = ox + w - 1;
    let bottom = oy + h - 1;

    buf[(ox, oy)].set_char('┌').set_style(style);
    buf[(right, oy)].set_char('┐').set_style(style);
    buf[(ox, bottom)].set_char('└').set_style(style);
    buf[(right, bottom)].set_char('┘').set_style(style);

    for dx in 1..w - 1 {
        buf[(ox + dx, oy)].set_char('─').set_style(style);
        buf[(ox + dx, bottom)].set_char('─').set_style(style);
    }
    for dy in 1..h - 1 {
        buf[(ox, oy + dy)].set_char('│').set_style(style);
        buf[(right, oy + dy)].set_char('│').set_style(style);
    }

    let title = " Inventory ";
    if (title.len() as u16 + 4) < w {
        buf.set_string(ox + 2, oy, title, title_style);
    }
}

fn draw_header(buf: &mut Buffer, x: u16, y: u16, item_w: usize, qty_w: usize, bg: Color) {
    let style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(bg);
    let sep_style = Style::default().fg(Color::White).bg(bg);

    buf.set_string(x, y, pad_right("Item", item_w), style);
    buf[(x + item_w as u16, y)]
        .set_char('│')
        .set_style(sep_style);
    buf.set_string(x + item_w as u16 + 1, y, pad_right("Qty", qty_w), style);
}

fn draw_separator(buf: &mut Buffer, ox: u16, sep_y: u16, w: u16, item_w: usize, bg: Color) {
    let style = Style::default().fg(Color::White).bg(bg);

    buf[(ox, sep_y)].set_char('├').set_style(style);
    buf[(ox + w - 1, sep_y)].set_char('┤').set_style(style);
    for dx in 1..w - 1 {
        buf[(ox + dx, sep_y)].set_char('─').set_style(style);
    }
    let col_sep_x = ox + 1 + item_w as u16;
    if col_sep_x > ox && col_sep_x < ox + w - 1 {
        buf[(col_sep_x, sep_y)].set_char('┼').set_style(style);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_row(
    buf: &mut Buffer,
    item: &(String, u32),
    x: u16,
    y: u16,
    item_w: usize,
    qty_w: usize,
    total_inner_w: usize,
    selected: bool,
    base_bg: Color,
) {
    let sel_bg = Color::Rgb(230, 228, 220);
    let row_bg = if selected { sel_bg } else { base_bg };
    let fg = if selected {
        Color::Black
    } else {
        Color::Rgb(180, 190, 210)
    };
    let text_style = Style::default().fg(fg).bg(row_bg);
    let sep_style = Style::default().fg(Color::White).bg(row_bg);

    for dx in 0..total_inner_w as u16 {
        buf[(x + dx, y)].set_bg(row_bg);
    }

    buf.set_string(x, y, pad_right(&item.0, item_w), text_style);
    buf[(x + item_w as u16, y)]
        .set_char('│')
        .set_style(sep_style);
    buf.set_string(
        x + item_w as u16 + 1,
        y,
        pad_right(&item.1.to_string(), qty_w),
        text_style,
    );
}

fn truncate_str(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(1);
        if w + cw > width {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out
}

fn pad_right(s: &str, width: usize) -> String {
    let vw = visual_width(s);
    if vw >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - vw))
    }
}

fn visual_width(s: &str) -> usize {
    s.chars()
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(1))
        .sum()
}
