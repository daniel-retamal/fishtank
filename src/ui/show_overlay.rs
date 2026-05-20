use rand::RngExt;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthChar;

use crate::entities::fish::{Direction, Fish};
use crate::entities::species::FishSpecies;
use crate::tank::TankKind;
use crate::ui::{
    fields::{self, FieldKind},
    render_fish_segs, table,
};

const BG: Color = Color::Reset;
const FISH_PAD: u16 = 1;
const FISH_INNER_H: u16 = 3;
const MIN_BODY_W: u16 = 38;
const MIN_OVERLAY_H: u16 = 7;
const FIELD_COUNT_MIN: usize = 3;
const FIELD_COUNT_MAX: usize = 6;

pub enum ShowSource {
    FromIndex,
    FromCommand,
}

struct ShowField {
    label: &'static str,
    value: String,
    swatch: Option<Color>,
}

pub struct ShowState {
    pub fish: Fish,
    fields: Vec<ShowField>,
    pub scroll: usize,
    pub source: ShowSource,
}

impl ShowState {
    pub fn new(
        fish: &Fish,
        tank_name: &str,
        tank_kind: TankKind,
        all_names: &[String],
        source: ShowSource,
        all: bool,
        rng: &mut impl RngExt,
    ) -> Self {
        let mut display_fish = fish.clone();
        display_fish.facing = Direction::Right;

        let mut show_fields: Vec<ShowField> = vec![
            ShowField {
                label: "Name",
                value: fish.name.clone(),
                swatch: None,
            },
            ShowField {
                label: "Species",
                value: fish.species.display_name().to_string(),
                swatch: None,
            },
            ShowField {
                label: "Weight",
                value: fields::format_weight(fish.weight_g),
                swatch: None,
            },
            ShowField {
                label: "Fishtank",
                value: tank_name.to_string(),
                swatch: None,
            },
            ShowField {
                label: "Tank Type",
                value: tank_kind.display_name().to_string(),
                swatch: None,
            },
        ];

        if fish.devil_marked {
            show_fields.push(ShowField {
                label: "Marked by the Devil",
                value: "Yes".to_string(),
                swatch: None,
            });
        }

        if all {
            for &kind in FieldKind::all() {
                let fv = fields::gen_field_value(kind, fish, all_names, rng);
                show_fields.push(ShowField {
                    label: kind.header(),
                    value: fv.text,
                    swatch: fv.swatch,
                });
            }
        } else {
            let count = rng.random_range(FIELD_COUNT_MIN..=FIELD_COUNT_MAX);
            let mut avail: Vec<FieldKind> = FieldKind::all().to_vec();
            for _ in 0..count.min(avail.len()) {
                let idx = rng.random_range(0..avail.len());
                let kind = avail.remove(idx);
                let fv = fields::gen_field_value(kind, fish, all_names, rng);
                show_fields.push(ShowField {
                    label: kind.header(),
                    value: fv.text,
                    swatch: fv.swatch,
                });
            }
        }

        if let Some(ref m) = fish.mutant {
            if !m.mitosis_partners.is_empty() {
                show_fields.push(ShowField {
                    label: "Mitosis Partners",
                    value: m.mitosis_partners.join(", "),
                    swatch: None,
                });
            }
            show_fields.push(ShowField {
                label: "Mutation Count",
                value: m.mutation_count.to_string(),
                swatch: None,
            });
            let history = if m.mutation_history.is_empty() {
                "—".to_string()
            } else {
                m.mutation_history.join(", ")
            };
            show_fields.push(ShowField {
                label: "Mutation History",
                value: history,
                swatch: None,
            });
        }

        Self {
            fish: display_fish,
            fields: show_fields,
            scroll: 0,
            source,
        }
    }

    pub fn tick_animation(&mut self, dt: f32) {
        self.fish.tick_animation(dt);
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn scroll_down(&mut self, visible: usize) {
        if self.scroll + visible < self.fields.len() {
            self.scroll += 1;
        }
    }

    pub fn visible_count(overlay_h: u16) -> usize {
        (overlay_h.saturating_sub(3) / 3) as usize
    }

    pub fn overlay_h(&self, area_h: u16, area_w: u16) -> u16 {
        let fish_inner_w = self.fish.display_width as u16 + 2 * FISH_PAD;
        let max_content_w = self
            .fields
            .iter()
            .flat_map(|f| [table::visual_width(f.label), table::visual_width(&f.value)])
            .max()
            .unwrap_or(0);
        let target_right_inner_w = MIN_BODY_W.max((max_content_w + 2) as u16);
        let overlay_w = (fish_inner_w + 3 + target_right_inner_w).min(area_w);
        let right_inner_w = overlay_w.saturating_sub(fish_inner_w + 3);
        let available_content = right_inner_w.saturating_sub(2) as usize;
        let total_field_rows: u16 = self
            .fields
            .iter()
            .map(|f| {
                let lines = if f.swatch.is_some() {
                    1u16
                } else {
                    value_line_count(&f.value, available_content) as u16
                };
                1 + lines + 1
            })
            .sum();
        total_field_rows
            .saturating_sub(1)
            .saturating_add(4)
            .min(area_h)
            .max(MIN_OVERLAY_H)
    }
}

fn hard_break_word(word: &str, max_w: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut chunk = String::new();
    let mut w = 0usize;
    for c in word.chars() {
        let cw = UnicodeWidthChar::width(c).unwrap_or(1);
        if w + cw > max_w && !chunk.is_empty() {
            lines.push(std::mem::take(&mut chunk));
            w = 0;
        }
        chunk.push(c);
        w += cw;
    }
    if !chunk.is_empty() {
        lines.push(chunk);
    }
    lines
}

fn wrap_value(value: &str, max_w: usize) -> Vec<String> {
    if max_w == 0 || value.is_empty() {
        return vec![value.to_string()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_w = 0usize;
    for word in value.split_whitespace() {
        let word_w = table::visual_width(word);
        if current_w == 0 {
            if word_w <= max_w {
                current.push_str(word);
                current_w = word_w;
            } else {
                lines.extend(hard_break_word(word, max_w));
            }
        } else if current_w + 1 + word_w <= max_w {
            current.push(' ');
            current.push_str(word);
            current_w += 1 + word_w;
        } else {
            lines.push(std::mem::take(&mut current));
            current_w = 0;
            if word_w <= max_w {
                current.push_str(word);
                current_w = word_w;
            } else {
                lines.extend(hard_break_word(word, max_w));
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn value_line_count(value: &str, max_w: usize) -> usize {
    wrap_value(value, max_w).len()
}

pub struct ShowOverlay<'a> {
    state: &'a ShowState,
}

impl<'a> ShowOverlay<'a> {
    pub fn new(state: &'a ShowState) -> Self {
        Self { state }
    }
}

impl Widget for ShowOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state;

        let fish_art_w = state.fish.display_width as u16;
        let fish_inner_w = fish_art_w + 2 * FISH_PAD;

        let max_content_w = state
            .fields
            .iter()
            .flat_map(|f| [table::visual_width(f.label), table::visual_width(&f.value)])
            .max()
            .unwrap_or(0);
        let target_right_inner_w = MIN_BODY_W.max((max_content_w + 2) as u16);
        let overlay_w = (fish_inner_w + 3 + target_right_inner_w).min(area.width);
        let right_inner_w = overlay_w.saturating_sub(fish_inner_w + 3);
        let available_content = right_inner_w.saturating_sub(2) as usize;

        if area.height < MIN_OVERLAY_H || available_content == 0 {
            return;
        }

        let total_field_rows: u16 = state
            .fields
            .iter()
            .map(|f| {
                let lines = if f.swatch.is_some() {
                    1u16
                } else {
                    value_line_count(&f.value, available_content) as u16
                };
                1 + lines + 1
            })
            .sum();
        let overlay_h = total_field_rows
            .saturating_sub(1)
            .saturating_add(4)
            .min(area.height)
            .max(MIN_OVERLAY_H);

        let ox = area.x + area.width.saturating_sub(overlay_w) / 2;
        let oy = area.y + (area.height - overlay_h) / 2;
        let right = ox + overlay_w - 1;
        let bottom = oy + overlay_h - 1;
        let sep_x = ox + 1 + fish_inner_w;
        let right_x = sep_x + 2;
        let fish_bottom_y = oy + FISH_INNER_H + 1;
        let field_end_y = oy + overlay_h - 4;

        for dy in 0..=FISH_INNER_H + 1 {
            for dx in 0..overlay_w {
                buf[(ox + dx, oy + dy)].reset();
            }
        }
        for dy in FISH_INNER_H + 2..overlay_h {
            for dx in fish_inner_w + 1..overlay_w {
                buf[(ox + dx, oy + dy)].reset();
            }
        }

        let border_color = match state.fish.species {
            FishSpecies::Goldenfish => Color::LightYellow,
            FishSpecies::Mutantfish => state.fish.color,
            _ => Color::White,
        };
        let border_style = Style::default().fg(border_color).bg(BG);
        let title_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
            .bg(BG);

        buf[(ox, oy)].set_char('┌').set_style(border_style);
        buf[(right, oy)].set_char('┐').set_style(border_style);
        for dx in 1..overlay_w - 1 {
            buf[(ox + dx, oy)].set_char('─').set_style(border_style);
        }
        let title = " FishResource#show ";
        if (title.len() as u16 + 4) < overlay_w {
            buf.set_string(ox + 2, oy, title, title_style);
        }

        for dy in 1..overlay_h - 1 {
            buf[(right, oy + dy)].set_char('│').set_style(border_style);
        }

        for dy in 1..=FISH_INNER_H {
            buf[(ox, oy + dy)].set_char('│').set_style(border_style);
        }

        buf[(ox, fish_bottom_y)]
            .set_char('└')
            .set_style(border_style);
        for dx in 1..=fish_inner_w {
            buf[(ox + dx, fish_bottom_y)]
                .set_char('─')
                .set_style(border_style);
        }
        buf[(sep_x, fish_bottom_y)]
            .set_char('┤')
            .set_style(border_style);

        for dy in 1..overlay_h - 1 {
            let y = oy + dy;
            if y != fish_bottom_y {
                buf[(sep_x, y)].set_char('│').set_style(border_style);
            }
        }

        buf[(sep_x, bottom)].set_char('└').set_style(border_style);
        for x in sep_x + 1..right {
            buf[(x, bottom)].set_char('─').set_style(border_style);
        }
        buf[(right, bottom)].set_char('┘').set_style(border_style);

        let extra = fish_inner_w.saturating_sub(fish_art_w);
        let fish_art_x = ox + 1 + extra / 2;
        let fish_art_y = oy + 1 + FISH_INNER_H / 2;
        let segs = state.fish.segments();
        render_fish_segs(buf, &segs, fish_art_x, fish_art_y, fish_art_w, BG);

        let white_bold = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
            .bg(BG);
        let white = Style::default().fg(Color::White).bg(BG);

        let mut y = oy + 1;
        let mut rendered_count = 0usize;
        for field in state.fields.iter().skip(state.scroll) {
            if y > field_end_y {
                break;
            }
            buf.set_string(
                right_x,
                y,
                table::truncate_str(field.label, available_content),
                white_bold,
            );

            if let Some(swatch) = field.swatch {
                let vy = y + 1;
                if vy <= field_end_y {
                    let sw = 6u16.min(right_inner_w.saturating_sub(2));
                    for dx in 0..sw {
                        buf[(right_x + dx, vy)]
                            .set_char(' ')
                            .set_fg(swatch)
                            .set_bg(swatch);
                    }
                }
                y += 3;
            } else {
                let wrapped = wrap_value(&field.value, available_content);
                for (li, line) in wrapped.iter().enumerate() {
                    let vy = y + 1 + li as u16;
                    if vy > field_end_y {
                        break;
                    }
                    buf.set_string(right_x, vy, line, white);
                }
                y += 1 + wrapped.len() as u16 + 1;
            }
            rendered_count += 1;
        }

        let footer_y = oy + overlay_h - 2;
        let hint_style = Style::default().fg(Color::DarkGray).bg(BG);
        let last_visible = state.scroll + rendered_count;
        let actually_scrollable = state.scroll > 0 || last_visible < state.fields.len();
        let left_footer = if actually_scrollable {
            format!("↑↓ navigate ({}/{})", last_visible, state.fields.len())
        } else {
            String::new()
        };
        let right_footer = match state.source {
            ShowSource::FromIndex => "ESC/q return",
            ShowSource::FromCommand => "ESC/q close",
        };
        let left_w = table::visual_width(&left_footer) as u16;
        let right_w = table::visual_width(right_footer) as u16;
        if left_w + 2 <= right_inner_w {
            buf.set_string(right_x, footer_y, &left_footer, hint_style);
        }
        if right_w + 2 <= right_inner_w {
            let footer_right_x = right_x + right_inner_w - right_w - 2;
            if footer_right_x > right_x + left_w {
                buf.set_string(footer_right_x, footer_y, right_footer, hint_style);
            }
        }
    }
}
