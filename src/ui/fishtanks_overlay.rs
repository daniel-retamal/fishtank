use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::colors::{BLACK, DARK_GRAY, STEEL, WHITE};
use crate::tank::{Tank, TankKind};
use crate::ui::{
    hints::{HINT_CLOSE, HINT_ENTER_SWITCH, HINT_NAV, HINT_SCROLL},
    scroll_list, table,
};

struct FishtanksEntry {
    name: String,
    kind: TankKind,
    fish_count: usize,
    capacity: usize,
}

pub struct FishtanksState {
    entries: Vec<FishtanksEntry>,
    pub selected: usize,
    scroll: usize,
    pub current_tank: usize,
}

impl FishtanksState {
    pub fn new(tanks: &[Tank], current_tank: usize, visible_rows: usize) -> Self {
        let entries = tanks
            .iter()
            .map(|t| FishtanksEntry {
                name: t.name.clone(),
                kind: t.kind,
                fish_count: t.fish.len(),
                capacity: t.capacity(),
            })
            .collect();
        let scroll = if current_tank >= visible_rows {
            current_tank + 1 - visible_rows
        } else {
            0
        };
        Self {
            entries,
            selected: current_tank,
            scroll,
            current_tank,
        }
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
}

pub struct FishtanksOverlay<'a> {
    state: &'a FishtanksState,
}

impl<'a> FishtanksOverlay<'a> {
    pub fn new(state: &'a FishtanksState) -> Self {
        Self { state }
    }
}

impl Widget for FishtanksOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state;
        let n = state.entries.len();
        let bg = Color::Reset;

        let name_w = state
            .entries
            .iter()
            .map(|e| table::visual_width(&e.name))
            .max()
            .unwrap_or(4)
            .max(table::visual_width("Name"));
        let type_w = state
            .entries
            .iter()
            .map(|e| table::visual_width(e.kind.display_name()))
            .max()
            .unwrap_or(4)
            .max(table::visual_width("Type"));
        let fishes_w = state
            .entries
            .iter()
            .map(|e| table::visual_width(&format!("{}/{}", e.fish_count, e.capacity)))
            .max()
            .unwrap_or(0)
            .max(table::visual_width("Fishes"));
        let content_inner_w = name_w + 1 + type_w + 1 + fishes_w;

        let visible_data_rows = n.min(area.height.saturating_sub(6) as usize).max(1);
        let scrollable = n > visible_data_rows;

        let enter_hint = HINT_ENTER_SWITCH;
        let close_hint = HINT_CLOSE;
        let enter_w = table::visual_width(enter_hint);
        let close_w = table::visual_width(close_hint);
        let nav_max_w = if scrollable {
            table::visual_width(&format!(" {} ({}/{})", HINT_SCROLL, n, n))
        } else {
            table::visual_width(&format!(" {}", HINT_NAV))
        };
        let footer_inner_w = nav_max_w + 2 + enter_w + 2 + close_w + 1;

        let inner_w = content_inner_w.max(footer_inner_w);
        let overlay_w = (inner_w + 2) as u16;
        let overlay_h = ((visible_data_rows + 6) as u16).min(area.height);

        let Some(layout) = table::OverlayLayout::centered(area, overlay_w, overlay_h) else {
            return;
        };
        layout.clear_bg(buf, bg);
        layout.draw_border(buf, " Fishtank#index ", WHITE, bg);

        let (ox, oy) = (layout.ox, layout.oy);
        let inner_x = layout.inner_x();
        let sep_x = inner_x + name_w as u16;
        let sep2_x = sep_x + 1 + type_w as u16;
        let cols = Columns {
            name_w,
            type_w,
            fishes_w,
            sep_x,
            sep2_x,
            total_inner_w: inner_w,
        };

        draw_header(buf, inner_x, oy + 1, &cols, bg);
        table::draw_box_separator(buf, ox, oy + 2, overlay_w, &[sep_x, sep2_x], WHITE, bg);

        let data_start_y = oy + 3;
        let data_end_y = oy + overlay_h.saturating_sub(4);

        for row_y in data_start_y..=data_end_y {
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
        let enter_x = close_x - 2 - enter_w as u16;

        buf.set_string(inner_x, footer_y, &nav_hint, hint_style);
        if state.selected != state.current_tank {
            buf.set_string(enter_x, footer_y, enter_hint, hint_style);
        }
        buf.set_string(close_x, footer_y, close_hint, hint_style);
    }
}

struct Columns {
    name_w: usize,
    type_w: usize,
    fishes_w: usize,
    sep_x: u16,
    sep2_x: u16,
    total_inner_w: usize,
}

fn draw_header(buf: &mut Buffer, x: u16, y: u16, cols: &Columns, bg: Color) {
    let bold = Style::default()
        .fg(WHITE)
        .add_modifier(Modifier::BOLD)
        .bg(bg);
    let sep = Style::default().fg(WHITE).bg(bg);

    buf.set_string(x, y, table::pad_right("Name", cols.name_w), bold);
    buf[(cols.sep_x, y)].set_char('│').set_style(sep);
    buf.set_string(
        cols.sep_x + 1,
        y,
        table::pad_right("Type", cols.type_w),
        bold,
    );
    buf[(cols.sep2_x, y)].set_char('│').set_style(sep);
    buf.set_string(
        cols.sep2_x + 1,
        y,
        table::pad_right("Fishes", cols.fishes_w),
        bold,
    );
}

fn draw_row(
    buf: &mut Buffer,
    entry: &FishtanksEntry,
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

    for dx in 0..cols.total_inner_w as u16 {
        buf[(x + dx, y)].set_bg(row_bg);
    }

    buf.set_string(x, y, table::pad_right(&entry.name, cols.name_w), text_style);
    buf[(cols.sep_x, y)].set_char('│').set_style(sep_style);
    buf.set_string(
        cols.sep_x + 1,
        y,
        table::pad_right(entry.kind.display_name(), cols.type_w),
        text_style,
    );
    buf[(cols.sep2_x, y)].set_char('│').set_style(sep_style);
    let count_str = format!("{}/{}", entry.fish_count, entry.capacity);
    buf.set_string(
        cols.sep2_x + 1,
        y,
        table::pad_right(&count_str, cols.fishes_w),
        text_style,
    );
}
