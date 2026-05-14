use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::loot::ConsumableKind;
use crate::tank::ActiveConsumable;

pub fn height(
    show_stats: bool,
    width: u16,
    active_consumables: &[ActiveConsumable],
    money: u32,
    food_supply: u32,
    fish_count: usize,
) -> u16 {
    if !show_stats {
        return 3;
    }
    if !active_consumables.is_empty()
        && !inline_fits(width, active_consumables, money, food_supply, fish_count)
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
    pub food_supply: u32,
    pub money: u32,
    pub show_stats: bool,
    pub active_consumables: &'a [ActiveConsumable],
}

fn format_metric(n: u32) -> String {
    if n >= 1_000_000_000 {
        format!("{}.{}B", n / 1_000_000_000, (n % 1_000_000_000) / 100_000_000)
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

fn to_roman_bar(mut n: u32) -> String {
    const VALS: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut s = String::new();
    for &(val, sym) in VALS {
        while n >= val {
            s.push_str(sym);
            n -= val;
        }
    }
    s
}

fn consumables_total_len(active_consumables: &[ActiveConsumable]) -> usize {
    let mut total = 0;
    for (i, ac) in active_consumables.iter().enumerate() {
        if i > 0 {
            total += 2;
        }
        let name_len = match ac.kind {
            ConsumableKind::Coffee => 6,
            ConsumableKind::Bait => 4,
        };
        total += name_len + 1 + to_roman_bar(ac.stacks).len() + 2 + 5;
    }
    total
}

fn stats_str(money: u32, food_supply: u32, fish_count: usize) -> String {
    format!(
        "cash: {}  food: {}  fishes: {}/∞",
        format_metric(money),
        format_metric(food_supply),
        fish_count
    )
}

fn inline_fits(
    width: u16,
    active_consumables: &[ActiveConsumable],
    money: u32,
    food_supply: u32,
    fish_count: usize,
) -> bool {
    if active_consumables.is_empty() {
        return true;
    }
    let title_len: usize = 10;
    let cons_len = consumables_total_len(active_consumables);
    let s_len = stats_str(money, food_supply, fish_count).chars().count();
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
        let cursor_col = base_x + self.cursor_pos as u16;
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
            let ghost_x = base_x + self.input.len() as u16;
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
            buf.set_string(area.x, stats_row, "Fishtank I", white);

            let has_consumables = !self.active_consumables.is_empty();
            let stats = stats_str(self.money, self.food_supply, self.fish_count);
            let stats_width = stats.chars().count() as u16;
            let stats_x = area.right().saturating_sub(stats_width);
            if stats_width <= area.width {
                buf.set_string(stats_x, stats_row, &stats, white);
            }

            if has_consumables {
                let fits_inline = inline_fits(
                    area.width,
                    self.active_consumables,
                    self.money,
                    self.food_supply,
                    self.fish_count,
                );
                let cons_row = if fits_inline { stats_row } else { area.y + 4 };
                let cons_total = consumables_total_len(self.active_consumables) as u16;
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
                        to_roman_bar(ac.stacks),
                        format_mm_ss(ac.time_remaining)
                    );
                    let text_w = text.chars().count() as u16;
                    if x + text_w <= area.right() {
                        buf.set_string(x, cons_row, &text, white);
                        x += text_w;
                    }
                }
            }
        }
    }
}
