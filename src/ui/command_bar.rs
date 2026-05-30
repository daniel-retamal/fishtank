use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::colors::{BLACK, DARK_GRAY, WHITE};
use unicode_width::UnicodeWidthStr;

use crate::consumable::ActiveMilkStatus;
use crate::tank::ActiveConsumable;

pub struct StatsBar<'a> {
    pub active_consumables: &'a [ActiveConsumable],
    pub active_statuses: &'a [ActiveMilkStatus],
    pub cash: u32,
    pub food_supply: u32,
    pub fish_count: usize,
    pub fish_capacity: usize,
    pub tank_name: &'a str,
    pub devils_luck: u32,
}

pub fn height(show_stats: bool, width: u16, stats: &StatsBar) -> u16 {
    if !show_stats {
        return 3;
    }
    let has_statuses = !stats.active_consumables.is_empty()
        || !stats.active_statuses.is_empty()
        || stats.devils_luck > 0;
    if has_statuses && !inline_fits(width, stats) {
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
    pub cash: u32,
    pub show_stats: bool,
    pub active_consumables: &'a [ActiveConsumable],
    pub active_statuses: &'a [ActiveMilkStatus],
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

fn consumables_total_len(
    active_consumables: &[ActiveConsumable],
    active_statuses: &[ActiveMilkStatus],
    devils_luck: u32,
) -> usize {
    let mut total = 0;
    let mut items_seen = 0usize;
    for ac in active_consumables.iter() {
        if items_seen > 0 {
            total += 2;
        }
        let Some(label) = ac.kind.active_label() else {
            continue;
        };
        total += label.len() + 1 + crate::names::to_roman(ac.stacks).len() + 2 + 5;
        items_seen += 1;
    }
    for s in active_statuses.iter() {
        if items_seen > 0 {
            total += 2;
        }
        total += s.kind.display_name().len() + 1 + crate::names::to_roman(s.stacks).len() + 2 + 5;
        items_seen += 1;
    }
    if devils_luck > 0 {
        if items_seen > 0 {
            total += 2;
        }
        total += "devil's luck ".len() + crate::names::to_roman(devils_luck).len();
    }
    total
}

fn stats_str(cash: u32, food_supply: u32, fish_count: usize, fish_capacity: usize) -> String {
    format!(
        "cash: {}  food: {}  fishes: {}/{}",
        format_metric(cash),
        format_metric(food_supply),
        fish_count,
        fish_capacity,
    )
}

fn inline_fits(width: u16, stats: &StatsBar) -> bool {
    let has_statuses = !stats.active_consumables.is_empty()
        || !stats.active_statuses.is_empty()
        || stats.devils_luck > 0;
    if !has_statuses {
        return true;
    }
    let title_len = stats.tank_name.len();
    let cons_len = consumables_total_len(
        stats.active_consumables,
        stats.active_statuses,
        stats.devils_luck,
    );
    let s_len = stats_str(
        stats.cash,
        stats.food_supply,
        stats.fish_count,
        stats.fish_capacity,
    )
    .chars()
    .count();
    title_len + 2 + cons_len + 2 + s_len <= width as usize
}

impl Widget for CommandBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let sep = "─".repeat(area.width as usize);
        let sep_style = Style::default().fg(DARK_GRAY);
        let white = Style::default().fg(WHITE);
        let gray = Style::default().fg(DARK_GRAY);

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
                    .set_fg(BLACK)
                    .set_bg(WHITE);
            } else if at_end {
                buf[(cursor_col, row)].set_char(ch).set_fg(DARK_GRAY);
            } else {
                buf[(cursor_col, row)].set_char(ch).set_fg(WHITE);
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

            let stats = stats_str(
                self.cash,
                self.food_supply,
                self.fish_count,
                self.fish_capacity,
            );
            let stats_width = stats.chars().count() as u16;
            let stats_x = area.right().saturating_sub(stats_width);
            if stats_width <= area.width {
                buf.set_string(stats_x, stats_row, &stats, white);
            }

            let has_statuses = !self.active_consumables.is_empty()
                || !self.active_statuses.is_empty()
                || self.devils_luck > 0;
            if has_statuses {
                let fits_inline = inline_fits(
                    area.width,
                    &StatsBar {
                        active_consumables: self.active_consumables,
                        active_statuses: self.active_statuses,
                        cash: self.cash,
                        food_supply: self.food_supply,
                        fish_count: self.fish_count,
                        fish_capacity: self.fish_capacity,
                        tank_name: self.tank_name,
                        devils_luck: self.devils_luck,
                    },
                );
                let cons_row = if fits_inline { stats_row } else { area.y + 4 };
                let cons_total = consumables_total_len(
                    self.active_consumables,
                    self.active_statuses,
                    self.devils_luck,
                ) as u16;
                let start_x = if fits_inline {
                    stats_x.saturating_sub(cons_total + 2)
                } else {
                    area.x
                };
                let mut x = start_x;
                let mut items_drawn = 0usize;
                for ac in self.active_consumables.iter() {
                    if items_drawn > 0 {
                        x += 2;
                    }
                    let Some(name) = ac.kind.active_label() else {
                        continue;
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
                    items_drawn += 1;
                }
                for s in self.active_statuses.iter() {
                    if items_drawn > 0 {
                        x += 2;
                    }
                    let text = format!(
                        "{} {}: {}",
                        s.kind.display_name(),
                        crate::names::to_roman(s.stacks),
                        format_mm_ss(s.time_remaining)
                    );
                    let text_w = text.chars().count() as u16;
                    if x + text_w <= area.right() {
                        buf.set_string(x, cons_row, &text, white);
                        x += text_w;
                    }
                    items_drawn += 1;
                }
                if self.devils_luck > 0 {
                    if items_drawn > 0 {
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
