use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

use crate::loot::ConsumableKind;
use crate::tank::ActiveConsumable;

#[allow(clippy::too_many_arguments)]
pub fn height(
    show_stats: bool,
    width: u16,
    active_consumables: &[ActiveConsumable],
    money: u32,
    food_supply: u32,
    fish_count: usize,
    fish_capacity: usize,
    tank_name: &str,
    devils_luck: u32,
) -> u16 {
    if !show_stats {
        return 3;
    }
    let has_statuses = !active_consumables.is_empty() || devils_luck > 0;
    if has_statuses
        && !inline_fits(
            width,
            active_consumables,
            money,
            food_supply,
            fish_count,
            fish_capacity,
            tank_name,
            devils_luck,
        )
    {
        5
    } else {
        4
    }
}

pub struct CommandBar<'a> {
    pub input: &'a str,
    pub cursor_pos: usize,
    pub cursor_visible: bool,
    pub ghost: &'a str,
    pub fish_count: usize,
    pub fish_capacity: usize,
    pub food_supply: u32,
    pub money: u32,
    pub show_stats: bool,
    pub active_consumables: &'a [ActiveConsumable],
    pub tank_name: &'a str,
    pub devils_luck: u32,
}

fn format_metric(n: u32) -> String {
    if n >= 1_000_000_000 {
        format!(
            "{}.{}B",
            n / 1_000_000_000,
            (n % 1_000_000_000) / 100_000_000
        )
    } else if n >= 1_000_000 {
        format!("{}.{}M", n / 1_000_000, (n % 1_000_000) / 100_000)
    } else if n >= 1_000 {
        format!("{}.{}k", n / 1_000, (n % 1_000) / 100)
    } else {
        n.to_string()
    }
}

fn format_mm_ss(secs: f32) -> String {
    let total = secs.ceil() as u32;
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn consumables_total_len(active_consumables: &[ActiveConsumable], devils_luck: u32) -> usize {
    let mut total = 0;
    for (i, ac) in active_consumables.iter().enumerate() {
        if i > 0 {
            total += 2;
        }
        let name_len = match ac.kind {
            ConsumableKind::Coffee => 6,
            ConsumableKind::Bait => 4,
        };
        total += name_len + 1 + crate::names::to_roman(ac.stacks).len() + 2 + 5;
    }
    if devils_luck > 0 {
        if !active_consumables.is_empty() {
            total += 2;
        }
        total += "devil's luck ".len() + crate::names::to_roman(devils_luck).len();
    }
    total
}

fn stats_str(money: u32, food_supply: u32, fish_count: usize, fish_capacity: usize) -> String {
    format!(
        "cash: {}  food: {}  fishes: {}/{}",
        format_metric(money),
        format_metric(food_supply),
        fish_count,
        fish_capacity,
    )
}

#[allow(clippy::too_many_arguments)]
fn inline_fits(
    width: u16,
    active_consumables: &[ActiveConsumable],
    money: u32,
    food_supply: u32,
    fish_count: usize,
    fish_capacity: usize,
    tank_name: &str,
    devils_luck: u32,
) -> bool {
    let has_statuses = !active_consumables.is_empty() || devils_luck > 0;
    if !has_statuses {
        return true;
    }
    let title_len = tank_name.len();
    let cons_len = consumables_total_len(active_consumables, devils_luck);
    let s_len = stats_str(money, food_supply, fish_count, fish_capacity)
        .chars()
        .count();
    title_len + 2 + cons_len + 2 + s_len <= width as usize
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
        let display_col = UnicodeWidthStr::width(&self.input[..self.cursor_pos]) as u16;
        let cursor_col = base_x + display_col;
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
            let ghost_x = base_x + UnicodeWidthStr::width(self.input) as u16;
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
            buf.set_string(area.x, stats_row, self.tank_name, white);

            let has_statuses = !self.active_consumables.is_empty() || self.devils_luck > 0;
            let stats = stats_str(
                self.money,
                self.food_supply,
                self.fish_count,
                self.fish_capacity,
            );
            let stats_width = stats.chars().count() as u16;
            let stats_x = area.right().saturating_sub(stats_width);
            if stats_width <= area.width {
                buf.set_string(stats_x, stats_row, &stats, white);
            }

            if has_statuses {
                let fits_inline = inline_fits(
                    area.width,
                    self.active_consumables,
                    self.money,
                    self.food_supply,
                    self.fish_count,
                    self.fish_capacity,
                    self.tank_name,
                    self.devils_luck,
                );
                let cons_row = if fits_inline { stats_row } else { area.y + 4 };
                let cons_total =
                    consumables_total_len(self.active_consumables, self.devils_luck) as u16;
                let start_x = if fits_inline {
                    stats_x.saturating_sub(cons_total + 2)
                } else {
                    area.x
                };
                let mut x = start_x;
                for (i, ac) in self.active_consumables.iter().enumerate() {
                    if i > 0 {
                        x += 2;
                    }
                    let name = match ac.kind {
                        ConsumableKind::Coffee => "coffee",
                        ConsumableKind::Bait => "bait",
                    };
                    let text = format!(
                        "{} {}: {}",
                        name,
                        crate::names::to_roman(ac.stacks),
                        format_mm_ss(ac.time_remaining)
                    );
                    let text_w = text.chars().count() as u16;
                    if x + text_w <= area.right() {
                        buf.set_string(x, cons_row, &text, white);
                        x += text_w;
                    }
                }
                if self.devils_luck > 0 {
                    if !self.active_consumables.is_empty() {
                        x += 2;
                    }
                    let dl_text =
                        format!("devil's luck {}", crate::names::to_roman(self.devils_luck));
                    let dl_w = dl_text.chars().count() as u16;
                    if x + dl_w <= area.right() {
                        buf.set_string(x, cons_row, &dl_text, white);
                    }
                }
            }
        }
    }
}
