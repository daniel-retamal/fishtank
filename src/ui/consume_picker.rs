use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::colors::{BLACK, DARK_GRAY, STEEL, WHITE};
use crate::loot::MilkVariant;
use crate::ui::{
    hints::{HINT_CLOSE, HINT_ENTER_CONSUME, HINT_NAV, HINT_SCROLL},
    scroll_list, table,
};

pub struct ConsumePickerEntry {
    pub fish_name: String,
    pub species_display: String,
    pub tank_name: String,
    pub tank_idx: usize,
    pub fish_idx: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConsumePickerSource {
    FromInventory,
    FromCommand,
}

pub struct ConsumePickerState {
    pub milk: MilkVariant,
    pub item_name: String,
    pub remaining: u32,
    pub entries: Vec<ConsumePickerEntry>,
    pub selected: usize,
    pub scroll: usize,
    pub source: ConsumePickerSource,
}

impl ConsumePickerState {
    pub fn new(
        milk: MilkVariant,
        item_name: String,
        remaining: u32,
        entries: Vec<ConsumePickerEntry>,
        source: ConsumePickerSource,
    ) -> Option<Self> {
        if entries.is_empty() {
            return None;
        }
        Some(Self {
            milk,
            item_name,
            remaining,
            entries,
            selected: 0,
            scroll: 0,
            source,
        })
    }

    pub fn scroll_up(&mut self) {
        scroll_list::scroll_up(&mut self.selected, &mut self.scroll);
    }

    pub fn scroll_down(&mut self, visible: usize) {
        scroll_list::scroll_down(
            &mut self.selected,
            &mut self.scroll,
            self.entries.len(),
            visible,
        );
    }

    pub fn refresh_after_consume(
        &mut self,
        remaining: u32,
        entries_fn: impl Fn() -> Vec<ConsumePickerEntry>,
    ) {
        self.remaining = remaining;
        let prev_name = self.entries.get(self.selected).map(|e| e.fish_name.clone());
        self.entries = entries_fn();
        if let Some(name) = prev_name
            && let Some(pos) = self.entries.iter().position(|e| e.fish_name == name)
        {
            self.selected = pos;
            return;
        }
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
    }
}

const FRAME_ROWS: u16 = 6;

pub struct ConsumePickerOverlay<'a> {
    pub state: &'a ConsumePickerState,
}

impl ConsumePickerOverlay<'_> {
    pub fn visible_rows(area_h: u16) -> usize {
        area_h.saturating_sub(FRAME_ROWS) as usize
    }
}

impl Widget for ConsumePickerOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state;
        let n = state.entries.len();
        let bg = Color::Reset;

        let name_w = state
            .entries
            .iter()
            .map(|e| table::visual_width(&e.fish_name))
            .max()
            .unwrap_or(4)
            .max(table::visual_width("Name"));
        let species_w = state
            .entries
            .iter()
            .map(|e| table::visual_width(&e.species_display))
            .max()
            .unwrap_or(7)
            .max(table::visual_width("Species"));
        let tank_w = state
            .entries
            .iter()
            .map(|e| table::visual_width(&e.tank_name))
            .max()
            .unwrap_or(8)
            .max(table::visual_width("Fishtank"));
        let content_inner_w = name_w + 1 + species_w + 1 + tank_w;

        let visible_data_rows = n
            .min(area.height.saturating_sub(FRAME_ROWS) as usize)
            .max(1);
        let scrollable = n > visible_data_rows;

        let close_hint = HINT_CLOSE;
        let consume_hint = HINT_ENTER_CONSUME;
        let close_w = table::visual_width(close_hint);
        let consume_w = table::visual_width(consume_hint);
        let nav_max_w = if scrollable {
            table::visual_width(&format!(" {} ({}/{})", HINT_SCROLL, n, n))
        } else {
            table::visual_width(&format!(" {}", HINT_NAV))
        };
        let footer_inner_w = nav_max_w + 2 + consume_w + 2 + close_w + 1;

        let title = format!(" {} to the fishes ({}) ", state.item_name, state.remaining);
        let title_w = table::visual_width(&title) + 2;

        let inner_w = content_inner_w.max(footer_inner_w).max(title_w);
        let overlay_w = (inner_w + 2) as u16;
        let overlay_h = ((visible_data_rows + FRAME_ROWS as usize) as u16).min(area.height);

        let Some(layout) = table::OverlayLayout::centered(area, overlay_w, overlay_h) else {
            return;
        };
        layout.clear_bg(buf, bg);
        layout.draw_border(buf, &title, WHITE, bg);

        let (ox, oy) = (layout.ox, layout.oy);
        let inner_x = layout.inner_x();
        let sep_x = inner_x + name_w as u16;
        let sep2_x = sep_x + 1 + species_w as u16;
        let cols = Columns {
            name_w,
            species_w,
            tank_w,
            sep_x,
            sep2_x,
            inner_w,
        };

        draw_header(buf, inner_x, oy + 1, &cols, bg);
        table::draw_box_separator(buf, ox, oy + 2, overlay_w, &[sep_x, sep2_x], WHITE, bg);

        let data_start_y = oy + 3;
        let data_end_y = oy + overlay_h.saturating_sub(3);

        for row_y in data_start_y..data_end_y {
            let idx = state.scroll + (row_y - data_start_y) as usize;
            if idx >= n {
                break;
            }
            let selected = idx == state.selected;
            draw_row(
                buf,
                &state.entries[idx],
                inner_x,
                row_y,
                &cols,
                selected,
                bg,
            );
        }

        let nav_hint = if scrollable {
            format!(" {} ({}/{})", HINT_SCROLL, state.selected + 1, n)
        } else {
            format!(" {}", HINT_NAV)
        };

        let footer_y = oy + overlay_h - 2;
        let hint_style = Style::default().fg(DARK_GRAY).bg(bg);

        let close_x = inner_x + inner_w as u16 - 1 - close_w as u16;
        let center_x = inner_x + (inner_w as u16 - consume_w as u16) / 2;

        buf.set_string(inner_x, footer_y, &nav_hint, hint_style);
        buf.set_string(center_x, footer_y, consume_hint, hint_style);
        buf.set_string(close_x, footer_y, close_hint, hint_style);
    }
}

struct Columns {
    name_w: usize,
    species_w: usize,
    tank_w: usize,
    sep_x: u16,
    sep2_x: u16,
    inner_w: usize,
}

fn draw_header(buf: &mut Buffer, x: u16, y: u16, cols: &Columns, bg: Color) {
    let bold = Style::default()
        .fg(WHITE)
        .add_modifier(Modifier::BOLD)
        .bg(bg);
    let sep = Style::default().fg(WHITE).bg(bg);

    for dx in 0..cols.inner_w as u16 {
        buf[(x + dx, y)].set_bg(bg);
    }

    buf.set_string(x, y, center_text("Name", cols.name_w), bold);
    buf[(cols.sep_x, y)].set_char('│').set_style(sep);
    buf.set_string(
        cols.sep_x + 1,
        y,
        center_text("Species", cols.species_w),
        bold,
    );
    buf[(cols.sep2_x, y)].set_char('│').set_style(sep);
    buf.set_string(
        cols.sep2_x + 1,
        y,
        center_text("Fishtank", cols.tank_w),
        bold,
    );
}

fn draw_row(
    buf: &mut Buffer,
    entry: &ConsumePickerEntry,
    x: u16,
    y: u16,
    cols: &Columns,
    selected: bool,
    base_bg: Color,
) {
    let row_bg = if selected { WHITE } else { base_bg };
    let fg = if selected { BLACK } else { STEEL };
    let text_style = Style::default().fg(fg).bg(row_bg);
    let sep_style = Style::default().fg(WHITE).bg(row_bg);

    for dx in 0..cols.inner_w as u16 {
        buf[(x + dx, y)].set_bg(row_bg);
    }

    buf.set_string(x, y, center_text(&entry.fish_name, cols.name_w), text_style);
    buf[(cols.sep_x, y)].set_char('│').set_style(sep_style);
    buf.set_string(
        cols.sep_x + 1,
        y,
        center_text(&entry.species_display, cols.species_w),
        text_style,
    );
    buf[(cols.sep2_x, y)].set_char('│').set_style(sep_style);
    buf.set_string(
        cols.sep2_x + 1,
        y,
        center_text(&entry.tank_name, cols.tank_w),
        text_style,
    );
}

fn center_text(text: &str, width: usize) -> String {
    let trunc = table::truncate_str(text, width);
    let vw = table::visual_width(&trunc);
    let total_pad = width.saturating_sub(vw);
    let left = total_pad / 2;
    let right = total_pad - left;
    format!("{}{}{}", " ".repeat(left), trunc, " ".repeat(right))
}
