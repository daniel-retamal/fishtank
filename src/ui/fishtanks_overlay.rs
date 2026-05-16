use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::tank::{TANK_CAPACITY, Tank};
use crate::ui::{scroll_list, table};

struct FishtanksEntry {
    name: String,
    fish_count: usize,
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
                fish_count: t.fish.len(),
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
        let count_str_max = format!("{}/{}", TANK_CAPACITY, TANK_CAPACITY);
        let fishes_w = table::visual_width(&count_str_max).max(table::visual_width("Fishes"));
        let content_inner_w = name_w + 1 + fishes_w;

        let visible_data_rows = n.min(area.height.saturating_sub(6) as usize).max(1);
        let scrollable = n > visible_data_rows;

        let enter_hint = "ENTER switch";
        let close_hint = "ESC/q close";
        let enter_w = table::visual_width(enter_hint);
        let close_w = table::visual_width(close_hint);
        let nav_max_w = if scrollable {
            table::visual_width(&format!(" \u{2191}\u{2193} scroll ({}/{})", n, n))
        } else {
            table::visual_width(" \u{2191}\u{2193} navigate")
        };
        let footer_inner_w = nav_max_w + 2 + enter_w + 2 + close_w + 1;

        let inner_w = content_inner_w.max(footer_inner_w);
        let overlay_w = (inner_w + 2) as u16;
        let overlay_h = ((visible_data_rows + 6) as u16).min(area.height);

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

        table::draw_box_border(
            buf,
            ox,
            oy,
            overlay_w,
            overlay_h,
            " Fishtank#index ",
            Color::White,
            bg,
        );

        let inner_x = ox + 1;
        let sep_x = inner_x + name_w as u16;

        draw_header(buf, inner_x, oy + 1, name_w, fishes_w, sep_x, bg);
        table::draw_box_separator(buf, ox, oy + 2, overlay_w, &[sep_x], Color::White, bg);

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
                name_w,
                fishes_w,
                sep_x,
                inner_w,
                selected,
                bg,
            );
        }

        let nav_hint = if scrollable {
            format!(" \u{2191}\u{2193} scroll ({}/{})", state.selected + 1, n)
        } else {
            " \u{2191}\u{2193} navigate".to_string()
        };

        let footer_y = oy + overlay_h - 2;
        let hint_style = Style::default().fg(Color::DarkGray).bg(bg);

        let close_x = inner_x + inner_w as u16 - 1 - close_w as u16;
        let enter_x = close_x - 2 - enter_w as u16;

        buf.set_string(inner_x, footer_y, &nav_hint, hint_style);
        if state.selected != state.current_tank {
            buf.set_string(enter_x, footer_y, enter_hint, hint_style);
        }
        buf.set_string(close_x, footer_y, close_hint, hint_style);
    }
}

fn draw_header(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    name_w: usize,
    fishes_w: usize,
    sep_x: u16,
    bg: Color,
) {
    let bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(bg);
    let sep = Style::default().fg(Color::White).bg(bg);

    buf.set_string(x, y, table::pad_right("Name", name_w), bold);
    buf[(sep_x, y)].set_char('│').set_style(sep);
    buf.set_string(sep_x + 1, y, table::pad_right("Fishes", fishes_w), bold);
}

#[allow(clippy::too_many_arguments)]
fn draw_row(
    buf: &mut Buffer,
    entry: &FishtanksEntry,
    x: u16,
    y: u16,
    name_w: usize,
    fishes_w: usize,
    sep_x: u16,
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

    buf[(sep_x, y)].set_char('│').set_style(sep_style);
    buf.set_string(x, y, table::pad_right(&entry.name, name_w), text_style);

    let count_str = format!("{}/{}", entry.fish_count, TANK_CAPACITY);
    buf.set_string(
        sep_x + 1,
        y,
        table::pad_right(&count_str, fishes_w),
        text_style,
    );
}
