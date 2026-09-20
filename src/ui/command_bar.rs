use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::colors::{BLACK, DARK_GRAY, WHITE};
use unicode_width::UnicodeWidthStr;

use crate::consumable::ActiveMilkStatus;
use crate::names::to_roman;
use crate::tank::ActiveConsumable;
use crate::ui::hint_bar::HintBar;
use crate::ui::table::{ellipsize, visual_width};

const RULE_ROWS: u16 = 2;
const EDITOR_ROWS: u16 = 1;
const ITEM_GAP: &str = "  ";
const TINY_GAP: &str = " ";
const THOUSAND: u32 = 1_000;
const MILLION: u32 = 1_000_000;
const BILLION: u32 = 1_000_000_000;
const SECS_PER_MINUTE: u32 = 60;
const NAME_KEPT_W: usize = 16;
const TINY_FOOD: &str = "•";
const TINY_FISH: &str = "><>";
const WORD_JOINER: char = '-';

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Density {
    Full,
    Short,
    Tiny,
}

impl Density {
    const ALL: [Density; 3] = [Density::Full, Density::Short, Density::Tiny];

    fn gap(self) -> &'static str {
        match self {
            Density::Tiny => TINY_GAP,
            Density::Full | Density::Short => ITEM_GAP,
        }
    }
}

pub struct StatsBar<'a> {
    pub active_consumables: &'a [ActiveConsumable],
    pub active_statuses: &'a [ActiveMilkStatus],
    pub cash: u32,
    pub food_supply: u32,
    pub fish_count: usize,
    pub fish_capacity: usize,
    pub tank_name: &'a str,
    pub devils_luck: u32,
    pub cajetans_grace: u32,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct StatusRow {
    pub left: String,
    pub right: String,
}

impl StatsBar<'_> {
    fn stats(&self, density: Density) -> String {
        let (cash, food, fish) = match density {
            Density::Full => (
                format!("cash: {}", metric(self.cash)),
                format!("food: {}", metric(self.food_supply)),
                format!("fishes: {}/{}", self.fish_count, self.fish_capacity),
            ),
            Density::Short => (
                format!("${}", metric(self.cash)),
                format!("food {}", metric(self.food_supply)),
                format!("fish {}/{}", self.fish_count, self.fish_capacity),
            ),
            Density::Tiny => (
                format!("${}", rounded_metric(self.cash)),
                format!("{TINY_FOOD}{}", rounded_metric(self.food_supply)),
                format!("{TINY_FISH}{}/{}", self.fish_count, self.fish_capacity),
            ),
        };
        [cash, food, fish].join(density.gap())
    }

    fn statuses(&self, density: Density) -> Vec<String> {
        let mut items = Vec::new();
        for active in self.active_consumables {
            let Some(label) = active.kind.active_label() else {
                continue;
            };
            items.push(timed(label, active.stacks, active.time_remaining, density));
        }
        for status in self.active_statuses {
            items.push(timed(
                status.kind.display_name(),
                status.stacks,
                status.time_remaining,
                density,
            ));
        }
        if self.devils_luck > 0 {
            items.push(stacked("devil's luck", self.devils_luck, density));
        }
        if self.cajetans_grace > 0 {
            items.push(stacked("cajetan's grace", self.cajetans_grace, density));
        }
        items
    }

    pub fn rows(&self, width: u16) -> Vec<StatusRow> {
        let width = width as usize;
        let name_w = visual_width(self.tank_name).min(NAME_KEPT_W);
        let density = Density::ALL
            .into_iter()
            .find(|&density| name_w + ITEM_GAP.len() + visual_width(&self.stats(density)) <= width)
            .unwrap_or(Density::Tiny);
        let stats = self.stats(density);
        let statuses = self.statuses(density);
        let gap = density.gap();
        let joined = statuses.join(gap);
        let name_room = width.saturating_sub(visual_width(&stats) + ITEM_GAP.len());
        if name_room == 0 {
            let mut rows = vec![StatusRow {
                left: ellipsize(self.tank_name, width),
                right: String::new(),
            }];
            rows.extend(flow(
                stats.split(gap).map(str::to_string).collect(),
                width,
                gap,
            ));
            rows.extend(flow(statuses, width, gap));
            return rows;
        }
        let full_name_w = visual_width(self.tank_name);
        let inline = !joined.is_empty()
            && full_name_w
                + ITEM_GAP.len()
                + visual_width(&joined)
                + ITEM_GAP.len()
                + visual_width(&stats)
                <= width;
        let mut first = StatusRow {
            left: ellipsize(self.tank_name, name_room),
            right: stats,
        };
        if inline {
            first.left = format!("{}{ITEM_GAP}{joined}", self.tank_name);
            return vec![first];
        }
        let mut rows = vec![first];
        rows.extend(flow(statuses, width, gap));
        rows
    }
}

fn flow(items: Vec<String>, width: usize, gap: &str) -> Vec<StatusRow> {
    let mut rows: Vec<StatusRow> = Vec::new();
    let mut current = String::new();
    for item in items {
        let item = ellipsize(&item, width);
        let needed = if current.is_empty() {
            visual_width(&item)
        } else {
            visual_width(&current) + gap.len() + visual_width(&item)
        };
        if !current.is_empty() && needed > width {
            rows.push(StatusRow {
                left: std::mem::take(&mut current),
                right: String::new(),
            });
        }
        if !current.is_empty() {
            current.push_str(gap);
        }
        current.push_str(&item);
    }
    if !current.is_empty() {
        rows.push(StatusRow {
            left: current,
            right: String::new(),
        });
    }
    rows
}

fn initials(label: &str) -> String {
    label
        .split(|ch: char| ch.is_whitespace() || ch == WORD_JOINER)
        .filter_map(|word| word.chars().find(|ch| ch.is_alphabetic()))
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

fn timed(label: &str, stacks: u32, secs: f32, density: Density) -> String {
    let total = secs.ceil().max(0.0) as u32;
    let (minutes, seconds) = (total / SECS_PER_MINUTE, total % SECS_PER_MINUTE);
    match density {
        Density::Full => format!("{label} {}: {minutes:02}:{seconds:02}", to_roman(stacks)),
        Density::Short => format!("{label} {} {minutes}:{seconds:02}", to_roman(stacks)),
        Density::Tiny if total < SECS_PER_MINUTE => format!("{}{stacks} {total}s", initials(label)),
        Density::Tiny => format!(
            "{}{stacks} {}m",
            initials(label),
            total.div_ceil(SECS_PER_MINUTE)
        ),
    }
}

fn stacked(label: &str, stacks: u32, density: Density) -> String {
    match density {
        Density::Full => format!("{label} {}", to_roman(stacks)),
        Density::Short => format!("{label} {stacks}"),
        Density::Tiny => format!("{}{stacks}", initials(label)),
    }
}

fn metric(n: u32) -> String {
    if n >= BILLION {
        format!("{}.{}B", n / BILLION, (n % BILLION) / (BILLION / 10))
    } else if n >= MILLION {
        format!("{}.{}M", n / MILLION, (n % MILLION) / (MILLION / 10))
    } else if n >= THOUSAND {
        format!("{}.{}k", n / THOUSAND, (n % THOUSAND) / (THOUSAND / 10))
    } else {
        n.to_string()
    }
}

fn rounded_metric(n: u32) -> String {
    for (unit, suffix) in [(BILLION, "B"), (MILLION, "M"), (THOUSAND, "k")] {
        if n < unit {
            continue;
        }
        let tenths = (n as u64 * 10 + unit as u64 / 2) / unit as u64;
        if tenths < 100 && !tenths.is_multiple_of(10) {
            return format!("{}.{}{suffix}", tenths / 10, tenths % 10);
        }
        return format!("{}{suffix}", (tenths + 5) / 10);
    }
    n.to_string()
}

fn prompt_rows(width: u16, console: Option<&HintBar>) -> u16 {
    console.map_or(EDITOR_ROWS, |bar| bar.height(width))
}

pub fn height(show_stats: bool, width: u16, stats: &StatsBar, console: Option<&HintBar>) -> u16 {
    let prompt = RULE_ROWS + prompt_rows(width, console);
    if !show_stats {
        return prompt;
    }
    prompt + stats.rows(width).len() as u16
}

pub struct CommandBar<'a> {
    pub input: &'a str,
    pub cursor_pos: usize,
    pub cursor_visible: bool,
    pub ghost: &'a str,
    pub show_stats: bool,
    pub stats: StatsBar<'a>,
    pub console: Option<HintBar>,
}

impl CommandBar<'_> {
    fn render_editor(&self, area: Rect, buf: &mut Buffer) {
        let white = Style::default().fg(WHITE);
        let gray = Style::default().fg(DARK_GRAY);
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
    }
}

impl Widget for CommandBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let sep = "─".repeat(area.width as usize);
        let sep_style = Style::default().fg(DARK_GRAY);
        let white = Style::default().fg(WHITE);

        buf.set_string(area.x, area.y, &sep, sep_style);

        let prompt = prompt_rows(area.width, self.console.as_ref());
        match &self.console {
            Some(bar) => bar.draw(
                buf,
                Rect::new(area.x, area.y + 1, area.width, prompt).intersection(area),
                Color::Reset,
            ),
            None => self.render_editor(area, buf),
        }

        let lower_rule = area.y + 1 + prompt;
        if lower_rule < area.bottom() {
            buf.set_string(area.x, lower_rule, &sep, sep_style);
        }

        if !self.show_stats {
            return;
        }
        for (index, status) in self.stats.rows(area.width).iter().enumerate() {
            let y = lower_rule + 1 + index as u16;
            if y >= area.bottom() {
                return;
            }
            buf.set_stringn(area.x, y, &status.left, area.width as usize, white);
            let right_w = visual_width(&status.right) as u16;
            if right_w > 0 {
                let x = area.right().saturating_sub(right_w);
                buf.set_stringn(x, y, &status.right, area.width as usize, white);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(tank_name: &str) -> StatsBar<'_> {
        StatsBar {
            active_consumables: &[],
            active_statuses: &[],
            cash: 40_000,
            food_supply: 1_250,
            fish_count: 3,
            fish_capacity: 50,
            tank_name,
            devils_luck: 2,
            cajetans_grace: 0,
        }
    }

    #[test]
    fn a_wide_bar_says_everything_in_words_on_one_row() {
        let rows = bar("Fishtank").rows(100);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].right, "cash: 40.0k  food: 1.2k  fishes: 3/50");
        assert!(rows[0].left.contains("devil's luck II"), "{rows:?}");
    }

    #[test]
    fn a_narrower_bar_switches_to_symbols_before_it_hides_anything() {
        let rows = bar("Fishtank").rows(40);
        assert_eq!(rows[0].left, "Fishtank");
        assert_eq!(rows[0].right, "$40.0k  food 1.2k  fish 3/50");
    }

    #[test]
    fn a_tiny_bar_rounds_its_numbers_and_keeps_the_name() {
        let rows = bar("Fishtank").rows(26);
        assert_eq!(rows[0].right, "$40k •1.3k ><>3/50");
        assert!(rows[0].left.starts_with("Fis"), "{rows:?}");
        assert!(
            rows.iter().any(|row| row.left.contains("DL2")),
            "the status wraps to its own row as initials: {rows:?}"
        );
    }

    #[test]
    fn a_bar_narrower_than_its_stats_stacks_them_instead_of_dropping_them() {
        let rows = bar("Fishtank").rows(12);
        let text: String = rows
            .iter()
            .map(|row| format!("{}{}", row.left, row.right))
            .collect();
        assert!(text.contains("$40k"), "{rows:?}");
        assert!(text.contains("><>3/50"), "{rows:?}");
    }

    #[test]
    fn rounding_keeps_one_decimal_only_while_it_says_something() {
        assert_eq!(rounded_metric(999), "999");
        assert_eq!(rounded_metric(1_250), "1.3k");
        assert_eq!(rounded_metric(40_000), "40k");
        assert_eq!(rounded_metric(2_000_000), "2M");
    }

    #[test]
    fn a_status_timer_is_minutes_in_tiny_form() {
        assert_eq!(timed("caffeinated", 2, 299.0, Density::Tiny), "C2 5m");
        assert_eq!(timed("caffeinated", 2, 42.0, Density::Tiny), "C2 42s");
        assert_eq!(
            timed("caffeinated", 2, 299.0, Density::Full),
            "caffeinated II: 04:59"
        );
    }
}
