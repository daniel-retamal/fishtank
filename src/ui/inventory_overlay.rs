use rand::RngExt;
use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::colors::{SELECTED_COLOR, SELECTED_TANK_COLOR};
use crate::loot::ConsumableKind;
use crate::ui::{hints::{HINT_CLOSE, HINT_ENTER_CONSUME, HINT_NAV}, scroll_list, table};

pub struct InventoryItem {
    pub name: String,
    pub qty: u32,
    pub is_consumable: bool,
    pub desc: String,
}

pub struct InventoryState {
    pub selected: usize,
    pub scroll: usize,
    pub items: Vec<InventoryItem>,
}

fn item_desc(name: &str, rng: &mut impl RngExt) -> String {
    match name {
        "Coffee" => ConsumableKind::Coffee.description().to_string(),
        "Bait" => ConsumableKind::Bait.description().to_string(),
        "Necronomicon" => {
            "An Image [or Picture] of the Law of the Dead. Image and pre-image. Summons a Gate to Hell, The Helltank. The devil has a lot of cash"
                .to_string()
        }
        "Junk" => {
            if rng.random_range(0..10u32) == 0 {
                "Junk... having 100 would be nice".to_string()
            } else {
                "Junk...".to_string()
            }
        }
        _ => String::new(),
    }
}

impl InventoryState {
    pub fn new(inventory: &HashMap<String, u32>, rng: &mut impl RngExt) -> Option<Self> {
        let mut items: Vec<InventoryItem> = inventory
            .iter()
            .filter(|(_, qty)| **qty > 0)
            .map(|(name, qty)| InventoryItem {
                name: name.clone(),
                qty: *qty,
                is_consumable: matches!(name.as_str(), "Coffee" | "Bait" | "Necronomicon"),
                desc: item_desc(name, rng),
            })
            .collect();
        if items.is_empty() {
            return None;
        }
        items.sort_by_key(|item| item.name.clone());
        Some(Self {
            selected: 0,
            scroll: 0,
            items,
        })
    }

    pub fn update_from(&mut self, inventory: &HashMap<String, u32>, rng: &mut impl RngExt) {
        let old_descs: HashMap<String, String> = self
            .items
            .iter()
            .map(|item| (item.name.clone(), item.desc.clone()))
            .collect();

        let mut items: Vec<InventoryItem> = inventory
            .iter()
            .filter(|(_, qty)| **qty > 0)
            .map(|(name, qty)| {
                let desc = old_descs
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| item_desc(name, rng));
                InventoryItem {
                    name: name.clone(),
                    qty: *qty,
                    is_consumable: matches!(name.as_str(), "Coffee" | "Bait" | "Necronomicon"),
                    desc,
                }
            })
            .collect();

        items.sort_by_key(|item| item.name.clone());
        self.items = items;
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
    }

    pub fn scroll_up(&mut self) {
        scroll_list::scroll_up(&mut self.selected, &mut self.scroll);
    }

    pub fn scroll_down(&mut self, visible: usize) {
        scroll_list::scroll_down(
            &mut self.selected,
            &mut self.scroll,
            self.items.len(),
            visible,
        );
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

const CONS_WIDTH: usize = 11;

fn word_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_w = 0usize;

    for word in text.split_whitespace() {
        let word_w = table::visual_width(word);
        if current_w == 0 {
            let truncated = table::truncate_str(word, width);
            current_w = table::visual_width(&truncated);
            current = truncated;
        } else if current_w + 1 + word_w <= width {
            current.push(' ');
            current.push_str(word);
            current_w += 1 + word_w;
        } else {
            lines.push(current.clone());
            let truncated = table::truncate_str(word, width);
            current_w = table::visual_width(&truncated);
            current = truncated;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

impl Widget for InventoryOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state;
        let n = state.items.len();
        let bg = Color::Reset;

        let item_w = state
            .items
            .iter()
            .map(|item| table::visual_width(&item.name))
            .max()
            .unwrap_or(4)
            .max(table::visual_width("Item"));
        let qty_w = state
            .items
            .iter()
            .map(|item| item.qty.to_string().len())
            .max()
            .unwrap_or(1)
            .max(table::visual_width("Quantity"));

        let avail = (area.width as usize).saturating_sub(2);
        let base_inner = item_w + 1 + qty_w + 1 + CONS_WIDTH;
        let avail_for_desc = avail.saturating_sub(base_inner + 1);
        let show_desc = avail_for_desc >= table::visual_width("Description");
        let desc_w = avail_for_desc;

        let all_item_lines: Vec<Vec<String>> = state
            .items
            .iter()
            .map(|item| {
                if show_desc && desc_w > 0 {
                    let lines = word_wrap(&item.desc, desc_w);
                    if lines.is_empty() {
                        vec![String::new()]
                    } else {
                        lines
                    }
                } else {
                    vec![String::new()]
                }
            })
            .collect();

        let max_data_rows = area.height.saturating_sub(6) as usize;
        let mut data_rows_used = 0usize;
        let mut items_to_show = 0usize;
        for lines in all_item_lines
            .iter()
            .skip(state.scroll)
            .take(n - state.scroll)
        {
            let h = lines.len().max(1);
            if data_rows_used + h > max_data_rows {
                break;
            }
            data_rows_used += h;
            items_to_show += 1;
        }

        let visible_data_rows = data_rows_used.max(1);

        let scrollable = state.scroll > 0 || state.scroll + items_to_show < n;
        let left_hint = if scrollable {
            format!(" ↑↓ scroll ({}/{})", state.selected + 1, n)
        } else {
            HINT_NAV.to_string()
        };
        let consume_hint = HINT_ENTER_CONSUME;
        let right_hint = HINT_CLOSE;
        let left_w = table::visual_width(&left_hint);
        let consume_w = table::visual_width(consume_hint);
        let right_w = table::visual_width(right_hint);
        let footer_min_w = left_w + 2 + consume_w + 2 + right_w;

        let inner_w = if show_desc { avail } else { base_inner }.max(footer_min_w);
        let overlay_w = (inner_w + 2) as u16;
        let overlay_h = (visible_data_rows as u16 + 6).min(area.height);

        let Some(layout) = table::OverlayLayout::centered(area, overlay_w, overlay_h) else {
            return;
        };
        layout.clear_bg(buf, bg);

        let (ox, oy) = (layout.ox, layout.oy);

        layout.draw_border(buf, " Inventory#index ", Color::White, bg);

        let sep1_x = ox + 1 + item_w as u16;
        let sep2_x = sep1_x + 1 + qty_w as u16;
        let sep3_x = sep2_x + 1 + CONS_WIDTH as u16;
        let sep_xs_2 = [sep1_x, sep2_x];
        let sep_xs_3 = [sep1_x, sep2_x, sep3_x];
        let sep_xs: &[u16] = if show_desc { &sep_xs_3 } else { &sep_xs_2 };

        let inner_x = ox + 1;
        draw_header(buf, inner_x, oy + 1, item_w, qty_w, show_desc, desc_w, bg);
        table::draw_box_separator(buf, ox, oy + 2, overlay_w, sep_xs, Color::White, bg);

        let data_start_y = oy + 3;
        let data_end_y = oy + overlay_h.saturating_sub(4);

        let mut current_y = data_start_y;
        for (idx, (item, lines)) in state
            .items
            .iter()
            .zip(all_item_lines.iter())
            .enumerate()
            .skip(state.scroll)
            .take(items_to_show)
        {
            if current_y > data_end_y {
                break;
            }
            draw_row(
                buf,
                item,
                lines,
                inner_x,
                current_y,
                item_w,
                qty_w,
                inner_w,
                show_desc,
                desc_w,
                idx == state.selected,
                bg,
            );
            current_y += lines.len().max(1) as u16;
        }

        let footer_y = oy + overlay_h - 2;
        let hint_style = Style::default().fg(Color::DarkGray).bg(bg);

        buf.set_string(
            inner_x,
            footer_y,
            table::truncate_str(&left_hint, inner_w),
            hint_style,
        );

        let right_x = inner_x + inner_w.saturating_sub(right_w + 1) as u16;
        buf.set_string(right_x, footer_y, right_hint, hint_style);

        let selected_is_consumable = state
            .items
            .get(state.selected)
            .map(|item| item.is_consumable)
            .unwrap_or(false);
        if selected_is_consumable {
            let center_x = inner_x + (left_w + 2) as u16;
            buf.set_string(center_x, footer_y, consume_hint, hint_style);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_header(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    item_w: usize,
    qty_w: usize,
    show_desc: bool,
    desc_w: usize,
    bg: Color,
) {
    let bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(bg);
    let sep = Style::default().fg(Color::White).bg(bg);

    buf.set_string(x, y, table::pad_right("Item", item_w), bold);

    let q_x = x + item_w as u16;
    buf[(q_x, y)].set_char('│').set_style(sep);
    buf.set_string(q_x + 1, y, table::pad_right("Quantity", qty_w), bold);

    let c_x = q_x + 1 + qty_w as u16;
    buf[(c_x, y)].set_char('│').set_style(sep);
    buf.set_string(c_x + 1, y, table::pad_right("Consumable?", CONS_WIDTH), bold);

    if show_desc {
        let d_x = c_x + 1 + CONS_WIDTH as u16;
        buf[(d_x, y)].set_char('│').set_style(sep);
        buf.set_string(d_x + 1, y, table::truncate_str("Description", desc_w), bold);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_row(
    buf: &mut Buffer,
    item: &InventoryItem,
    desc_lines: &[String],
    x: u16,
    start_y: u16,
    item_w: usize,
    qty_w: usize,
    total_inner_w: usize,
    show_desc: bool,
    desc_w: usize,
    selected: bool,
    base_bg: Color,
) {
    let item_h = desc_lines.len().max(1);
    let row_bg = if selected { SELECTED_COLOR } else { base_bg };
    let fg = if selected { Color::Black } else { SELECTED_TANK_COLOR };
    let text_style = Style::default().fg(fg).bg(row_bg);
    let sep_style = Style::default().fg(Color::White).bg(row_bg);

    for dy in 0..item_h as u16 {
        for dx in 0..total_inner_w as u16 {
            buf[(x + dx, start_y + dy)].set_bg(row_bg);
        }
    }

    let q_x = x + item_w as u16;
    let c_x = q_x + 1 + qty_w as u16;
    let d_x = c_x + 1 + CONS_WIDTH as u16;

    for dy in 0..item_h as u16 {
        let py = start_y + dy;
        buf[(q_x, py)].set_char('│').set_style(sep_style);
        buf[(c_x, py)].set_char('│').set_style(sep_style);
        if show_desc {
            buf[(d_x, py)].set_char('│').set_style(sep_style);
        }
    }

    let center_y = start_y + (item_h.saturating_sub(1) / 2) as u16;
    buf.set_string(
        x,
        center_y,
        table::pad_right(&item.name, item_w),
        text_style,
    );
    buf.set_string(
        q_x + 1,
        center_y,
        table::pad_right(&item.qty.to_string(), qty_w),
        text_style,
    );
    let cons_val = if item.is_consumable { "Yes" } else { "No" };
    buf.set_string(
        c_x + 1,
        center_y,
        table::pad_right(cons_val, CONS_WIDTH),
        text_style,
    );

    if show_desc {
        for (i, line) in desc_lines.iter().enumerate() {
            let line_y = start_y + i as u16;
            buf.set_string(
                d_x + 1,
                line_y,
                table::truncate_str(line, desc_w),
                text_style,
            );
        }
    }
}
